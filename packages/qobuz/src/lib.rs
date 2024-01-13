#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "db")]
pub mod db;

use std::{collections::HashMap, str::Utf8Error};

use base64::{engine::general_purpose, Engine as _};
use moosicbox_core::sqlite::models::AsModelResult;
use moosicbox_json_utils::{ParseError, ToNestedValue, ToValue};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{AsRefStr, EnumString};
use thiserror::Error;
use url::form_urlencoded;

static AUTH_HEADER_NAME: &str = "x-user-auth-token";

#[derive(Debug, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum QobuzDeviceType {
    Browser,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct QobuzImage {
    pub small: String,
    pub thumbnail: String,
    pub large: String,
}

impl AsModelResult<QobuzImage, ParseError> for Value {
    fn as_model(&self) -> Result<QobuzImage, ParseError> {
        Ok(QobuzImage {
            small: self.to_value("small")?,
            thumbnail: self.to_value("thumbnail")?,
            large: self.to_value("large")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct QobuzGenre {
    pub id: u64,
    pub name: String,
    pub slug: String,
}

impl AsModelResult<QobuzGenre, ParseError> for Value {
    fn as_model(&self) -> Result<QobuzGenre, ParseError> {
        Ok(QobuzGenre {
            id: self.to_value("id")?,
            name: self.to_value("name")?,
            slug: self.to_value("slug")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct QobuzAlbum {
    pub id: u64,
    pub artist: String,
    pub artist_id: u64,
    pub maximum_bit_depth: u16,
    pub image: QobuzImage,
    pub title: String,
    pub qobuz_id: u64,
    pub released_at: u64,
    pub duration: u32,
    pub parental_warning: bool,
    pub popularity: u32,
    pub tracks_count: u32,
    pub genre: QobuzGenre,
    pub maximum_channel_count: u16,
    pub maximum_sampling_rate: f32,
}

impl QobuzAlbum {
    pub fn cover_url(&self, size: u16) -> String {
        let cover_path = self.image.large.replace('-', "/");
        format!("https://resources.qobuz.com/images/{cover_path}/{size}x{size}.jpg")
    }
}

impl AsModelResult<QobuzAlbum, ParseError> for Value {
    fn as_model(&self) -> Result<QobuzAlbum, ParseError> {
        Ok(QobuzAlbum {
            id: self.to_value("id")?,
            artist: self.to_nested_value(&["artist", "name"])?,
            artist_id: self.to_nested_value(&["artist", "id"])?,
            maximum_bit_depth: self.to_value("maximum_bit_depth")?,
            image: self.to_value::<&Value>("image")?.as_model()?,
            title: self.to_value("title")?,
            qobuz_id: self.to_value("qobuz_id")?,
            released_at: self.to_value("released_at")?,
            duration: self.to_value("duration")?,
            parental_warning: self.to_value("parental_warning")?,
            popularity: self.to_value("popularity")?,
            tracks_count: self.to_value("tracks_count")?,
            genre: self.to_value::<&Value>("genre")?.as_model()?,
            maximum_channel_count: self.to_value("maximum_channel_count")?,
            maximum_sampling_rate: self.to_value("maximum_sampling_rate")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct QobuzTrack {
    pub id: u64,
    pub track_number: u32,
    pub album_id: u64,
    pub artist_id: u64,
    pub audio_quality: String,
    pub copyright: Option<String>,
    pub duration: u32,
    pub parental_warning: bool,
    pub isrc: String,
    pub popularity: u32,
    pub title: String,
}

impl AsModelResult<QobuzTrack, ParseError> for Value {
    fn as_model(&self) -> Result<QobuzTrack, ParseError> {
        Ok(QobuzTrack {
            id: self.to_value("id")?,
            track_number: self.to_value("track_number")?,
            album_id: self.to_nested_value(&["album", "id"])?,
            artist_id: self.to_nested_value(&["artist", "id"])?,
            audio_quality: self.to_value("audio_quality")?,
            copyright: self.to_value("copyright")?,
            duration: self.to_value("duration")?,
            parental_warning: self.to_value("parental_warning")?,
            isrc: self.to_value("isrc")?,
            popularity: self.to_value("popularity")?,
            title: self.to_value("title")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct QobuzArtist {
    pub id: u64,
    pub picture: Option<String>,
    pub popularity: u32,
    pub name: String,
}

impl QobuzArtist {
    pub fn picture_url(&self, size: u16) -> Option<String> {
        self.picture.as_ref().map(|picture| {
            let picture_path = picture.replace('-', "/");
            format!("https://resources.qobuz.com/images/{picture_path}/{size}x{size}.jpg")
        })
    }
}

impl AsModelResult<QobuzArtist, ParseError> for Value {
    fn as_model(&self) -> Result<QobuzArtist, ParseError> {
        Ok(QobuzArtist {
            id: self.get("id").unwrap().as_u64().unwrap(),
            picture: self
                .get("picture")
                .unwrap()
                .as_str()
                .map(|pic| pic.to_string()),
            popularity: self.get("popularity").unwrap().as_u64().unwrap() as u32,
            name: self.to_value("name")?,
        })
    }
}

trait ToUrl {
    fn to_url(&self) -> String;
}

enum QobuzApiEndpoint {
    Login,
    Bundle,
    FavoriteAlbums,
    AlbumTracks,
}

static QOBUZ_PLAY_API_BASE_URL: &str = "https://play.qobuz.com";
static QOBUZ_API_BASE_URL: &str = "https://www.qobuz.com/api.json/0.2";

impl ToUrl for QobuzApiEndpoint {
    fn to_url(&self) -> String {
        match self {
            Self::Login => {
                format!("{QOBUZ_PLAY_API_BASE_URL}/login")
            }
            Self::Bundle => format!("{QOBUZ_PLAY_API_BASE_URL}/resources/:bundleVersion/bundle.js"),
            Self::FavoriteAlbums => format!("{QOBUZ_API_BASE_URL}/favorite/getUserFavorites"),
            Self::AlbumTracks => format!("{QOBUZ_API_BASE_URL}/album/get"),
        }
    }
}

fn replace_all(value: &str, params: &[(&str, &str)]) -> String {
    let mut string = value.to_string();

    for (key, value) in params {
        string = string.replace(key, value);
    }

    string
}

fn attach_query_string(value: &str, query: &[(&str, &str)]) -> String {
    let mut query_string = form_urlencoded::Serializer::new(String::new());

    for (key, value) in query {
        query_string.append_pair(key, value);
    }

    format!("{}?{}", value, &query_string.finish())
}

#[macro_export]
macro_rules! qobuz_api_endpoint {
    ($name:ident $(,)?) => {
        QobuzApiEndpoint::$name.to_url()
    };

    ($name:ident, $params:expr) => {
        replace_all(&qobuz_api_endpoint!($name), $params)
    };

    ($name:ident, $params:expr, $query:expr) => {
        attach_query_string(&qobuz_api_endpoint!($name, $params), $query)
    };
}

#[derive(Debug, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum QobuzAlbumOrder {
    Date,
}

#[derive(Debug, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum QobuzAlbumOrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Error)]
pub enum QobuzFavoriteAlbumsError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "db")]
    #[error(transparent)]
    Db(#[from] moosicbox_core::sqlite::db::DbError),
    #[error("No access token available")]
    NoAccessTokenAvailable,
    #[error("No user ID available")]
    NoUserIdAvailable,
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[allow(clippy::too_many_arguments)]
pub async fn favorite_albums(
    #[cfg(feature = "db")] db: &moosicbox_core::app::Db,
    offset: Option<u32>,
    limit: Option<u32>,
    access_token: Option<String>,
) -> Result<(Vec<QobuzAlbum>, u32), QobuzFavoriteAlbumsError> {
    #[cfg(feature = "db")]
    let access_token = {
        match access_token.clone() {
            Some(access_token) => access_token,
            _ => {
                let config = db::get_qobuz_config(&db.library.lock().unwrap().inner)?
                    .ok_or(QobuzFavoriteAlbumsError::NoAccessTokenAvailable)?;
                access_token.unwrap_or(config.access_token)
            }
        }
    };

    #[cfg(not(feature = "db"))]
    let access_token = access_token.ok_or(QobuzFavoriteAlbumsError::NoAccessTokenAvailable)?;

    let url = qobuz_api_endpoint!(
        FavoriteAlbums,
        &[],
        &[
            ("offset", &offset.unwrap_or(0).to_string()),
            ("limit", &limit.unwrap_or(100).to_string()),
            ("type", "albums"),
        ]
    );

    let value: Value = reqwest::Client::new()
        .get(url)
        .header(AUTH_HEADER_NAME, format!("Bearer {}", access_token))
        .send()
        .await?
        .json()
        .await?;

    let items = value
        .get("albums")
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_model())
        .collect::<Result<Vec<_>, _>>()?;

    let count = value.get("totalNumberOfItems").unwrap().as_u64().unwrap() as u32;

    Ok((items, count))
}

#[derive(Debug, Error)]
pub enum QobuzAlbumTracksError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "db")]
    #[error(transparent)]
    Db(#[from] moosicbox_core::sqlite::db::DbError),
    #[error("No access token available")]
    NoAccessTokenAvailable,
    #[error("Request failed: {0:?}")]
    RequestFailed(String),
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[allow(clippy::too_many_arguments)]
pub async fn album_tracks(
    #[cfg(feature = "db")] db: &moosicbox_core::app::Db,
    album_id: &str,
    offset: Option<u32>,
    limit: Option<u32>,
    access_token: Option<String>,
) -> Result<(Vec<QobuzTrack>, u32), QobuzAlbumTracksError> {
    #[cfg(feature = "db")]
    let access_token = match access_token {
        Some(access_token) => access_token,
        _ => {
            let config = db::get_qobuz_config(&db.library.lock().as_ref().unwrap().inner)?
                .ok_or(QobuzAlbumTracksError::NoAccessTokenAvailable)?;

            access_token.unwrap_or(config.access_token)
        }
    };

    #[cfg(not(feature = "db"))]
    let access_token = access_token.ok_or(QobuzAlbumTracksError::NoAccessTokenAvailable)?;

    let url = qobuz_api_endpoint!(
        AlbumTracks,
        &[
            ("album_id", album_id),
            ("offset", &offset.unwrap_or(0).to_string()),
            ("limit", &limit.unwrap_or(100).to_string()),
        ]
    );

    let value: Value = reqwest::Client::new()
        .get(url)
        .header(AUTH_HEADER_NAME, format!("Bearer {}", access_token))
        .send()
        .await?
        .json()
        .await?;

    let items = match value.get("tracks").unwrap().get("items") {
        Some(items) => items
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_model())
            .collect::<Result<Vec<_>, _>>()?,
        None => {
            return Err(QobuzAlbumTracksError::RequestFailed(format!("{value:?}")));
        }
    };

    let count = value.get("totalNumberOfItems").unwrap().as_u64().unwrap() as u32;

    Ok((items, count))
}

#[derive(Debug, Error)]
pub enum QobuzFetchLoginSourceError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "db")]
    #[error(transparent)]
    Db(#[from] moosicbox_core::sqlite::db::DbError),
}

#[allow(unused)]
async fn fetch_login_source() -> Result<String, QobuzFetchLoginSourceError> {
    let url = qobuz_api_endpoint!(Login);

    Ok(reqwest::Client::new().get(url).send().await?.text().await?)
}

#[allow(unused)]
async fn search_bundle_version(login_source: &str) -> Option<String> {
    static BUNDLE_ID_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(
            r#"<script src="/resources/(\d+\.\d+\.\d+-[a-z]\d{3})/bundle\.js"></script>"#,
        )
        .unwrap()
    });

    if let Some(caps) = BUNDLE_ID_REGEX.captures(login_source) {
        if let Some(version) = caps.get(1) {
            let version = version.as_str();
            log::debug!("Found version={version}");
            return Some(version.to_string());
        }
    }

    None
}

#[derive(Debug, Error)]
pub enum QobuzFetchBundleSourceError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "db")]
    #[error(transparent)]
    Db(#[from] moosicbox_core::sqlite::db::DbError),
}

#[allow(unused)]
async fn fetch_bundle_source(bundle_version: &str) -> Result<String, QobuzFetchBundleSourceError> {
    let url = qobuz_api_endpoint!(Bundle, &[(":bundleVersion", bundle_version)]);

    Ok(reqwest::Client::new().get(url).send().await?.text().await?)
}

#[derive(Debug, Error)]
pub enum QobuzFetchAppSecretsError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
    #[cfg(feature = "db")]
    #[error(transparent)]
    Db(#[from] moosicbox_core::sqlite::db::DbError),
    #[error("No App ID found in output")]
    NoAppId,
    #[error("No seed and timezone found in output")]
    NoSeedAndTimezone,
    #[error("No info and extras found in output")]
    NoInfoAndExtras,
    #[error("No matching info for timezone")]
    NoMatchingInfoForTimezone,
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

fn capitalize(value: &str) -> String {
    let mut v: Vec<char> = value.chars().collect();
    v[0] = v[0].to_uppercase().next().unwrap();
    v.into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppConfig {
    pub(crate) app_id: String,
    pub(crate) secrets: HashMap<String, String>,
}

#[allow(unused)]
pub(crate) async fn search_app_config(
    bundle: &str,
) -> Result<AppConfig, QobuzFetchAppSecretsError> {
    static APP_ID_REGEX: Lazy<regex::Regex> =
        Lazy::new(|| regex::Regex::new(r#"production:\{api:\{appId:"([^"]+)""#).unwrap());

    let app_id = if let Some(caps) = APP_ID_REGEX.captures(bundle) {
        if let Some(app_id) = caps.get(1) {
            let app_id = app_id.as_str();
            log::debug!("Found app_id={app_id}");
            app_id.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoAppId);
        }
    } else {
        return Err(QobuzFetchAppSecretsError::NoAppId);
    };

    let mut seed_timezones = vec![];

    static SEED_AND_TIMEZONE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(r#"[a-z]\.initialSeed\("([\w=]+)",window\.utimezone\.(.+?)\)"#).unwrap()
    });

    for caps in SEED_AND_TIMEZONE_REGEX.captures_iter(bundle) {
        let seed = if let Some(seed) = caps.get(1) {
            let seed = seed.as_str();
            log::debug!("Found seed={seed}");
            seed.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoSeedAndTimezone);
        };
        let timezone = if let Some(timezone) = caps.get(2) {
            let timezone = timezone.as_str();
            log::debug!("Found timezone={timezone}");
            timezone.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoSeedAndTimezone);
        };

        seed_timezones.push((seed, timezone));
    }

    if seed_timezones.is_empty() {
        return Err(QobuzFetchAppSecretsError::NoSeedAndTimezone);
    };

    let mut name_info_extras = vec![];

    static INFO_AND_EXTRAS_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(r#"name:"\w+/([^"]+)",info:"([\w=]+)",extras:"([\w=]+)""#).unwrap()
    });

    for caps in INFO_AND_EXTRAS_REGEX.captures_iter(bundle) {
        let name = if let Some(name) = caps.get(1) {
            let name = name.as_str();
            log::debug!("Found name={name}");
            name.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoInfoAndExtras);
        };
        let info = if let Some(info) = caps.get(2) {
            let info = info.as_str();
            log::debug!("Found info={info}");
            info.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoInfoAndExtras);
        };
        let extras = if let Some(extras) = caps.get(3) {
            let extras = extras.as_str();
            log::debug!("Found extras={extras}");
            extras.to_string()
        } else {
            return Err(QobuzFetchAppSecretsError::NoInfoAndExtras);
        };

        name_info_extras.push((name, info, extras));
    }

    if name_info_extras.is_empty() {
        return Err(QobuzFetchAppSecretsError::NoInfoAndExtras);
    };

    let mut secrets = HashMap::new();

    log::trace!("seed_timezones={:?}", &seed_timezones);
    for (seed, timezone) in seed_timezones {
        log::trace!("name_info_extras={:?}", &name_info_extras);
        let (_, info, _) = name_info_extras
            .iter()
            .find(|(name, _, _)| name.starts_with(&capitalize(&timezone)))
            .ok_or(QobuzFetchAppSecretsError::NoMatchingInfoForTimezone)
            .expect("No matching name for timezone");

        let secret_base64 = format!("{seed}{info}");
        let secret_base64 = &secret_base64[..44];
        let secret = general_purpose::STANDARD.decode(secret_base64)?;
        let secret = std::str::from_utf8(&secret)?.to_string();

        secrets.insert(timezone, secret);
    }

    Ok(AppConfig { app_id, secrets })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::*;

    static TEST_LOGIN_SOURCE: &str = r#"</script>
        <script src="/resources/7.1.3-b011/bundle.js"></script>
        </body>"#;
    static TEST_BUNDLE_SOURCE: &str = r#"s,extra:o},production:{api:{appId:"123456789",appSecret{var e=window.__ENVIRONMENT__;return"recette"===e?d.initialSeed("YjBiMGIwYmQzYWRiMzNmY2Q2YTc0MD",window.utimezone.london):"integration"===e?d.initialSeed("MjBiMGIwYmQzYWRiMzNmY2Q2YTc0MD",window.utimezone.algier):d.initialSeed("MzBiMGIwYmQzYWRiMzNmY2Q2YTc0MD",window.utimezone.berlin)},d.string{offset:"GMT",name:"Europe/Dublin",info:"XXXXX",extras:"XXXXX"},{offset:"GMT",name:"Europe/Lisbon"},{offset:"GMT",name:"Europe/London",info:"VmMjU1NTU1NTU=YjBiMGIwYmQzMzMz",extras:"MzMzMzMzMzMzMDVmMjU4OTA1NTU="},{offset:"UTC",name:"UTC"},{offset:"GMT+01:00",name:"Africa/Algiers",info:"VmMjU1NTU1NTI=YjBiMGIwYmQzMzMz",extras:"MzMzMzMzMzMzMDVmMjU4OTA1NTU="},{offset:"GMT+01:00",name:"Africa/Windhoek"},{offset:"GMT+01:00",name:"Atlantic/Azores"},{offset:"GMT+01:00",name:"Atlantic/Stanley"},{offset:"GMT+01:00",name:"Europe/Amsterdam"},{offset:"GMT+01:00",name:"Europe/Paris",info:"XXXXX",extras:"XXXXX"},{offset:"GMT+01:00",name:"Europe/Belgrade"},{offset:"GMT+01:00",name:"Europe/Brussels"},{offset:"GMT+02:00",name:"Africa/Cairo"},{offset:"GMT+02:00",name:"Africa/Blantyre"},{offset:"GMT+02:00",name:"Asia/Beirut"},{offset:"GMT+02:00",name:"Asia/Damascus"},{offset:"GMT+02:00",name:"Asia/Gaza"},{offset:"GMT+02:00",name:"Asia/Jerusalem"},{offset:"GMT+02:00",name:"Europe/Berlin",info:"VmMjU1NTU1NTM=YjBiMGIwYmQzMzMz",extras:"MzMzMzMzMzMzMDVmMjU4OTA1NTU="},{offset:"GMT+03:00",name:"Africa/Addis_Ababa"},{offset:"GMT+03:00",name:"Asia/Riyadh89"},{offset:"GMT+03:00",name:"Europe/Minsk"},{offset:"GMT+03:30""#;

    #[tokio::test]
    async fn test_search_bundle_version() {
        let version = search_bundle_version(TEST_LOGIN_SOURCE)
            .await
            .expect("Failed to search_bundle_version");

        assert_eq!(version, "7.1.3-b011");
    }

    #[tokio::test]
    async fn test_search_app_config() {
        let secrets = search_app_config(TEST_BUNDLE_SOURCE)
            .await
            .expect("Failed to search_app_config");

        assert_eq!(
            secrets,
            AppConfig {
                app_id: "123456789".to_string(),
                secrets: HashMap::from([
                    (
                        "london".to_string(),
                        "b0b0b0bd3adb33fcd6a7405f25555555".to_string()
                    ),
                    (
                        "algier".to_string(),
                        "20b0b0bd3adb33fcd6a7405f25555552".to_string()
                    ),
                    (
                        "berlin".to_string(),
                        "30b0b0bd3adb33fcd6a7405f25555553".to_string()
                    )
                ])
            }
        );
    }
}
