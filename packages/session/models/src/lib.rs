#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::sync::LazyLock;

use moosicbox_audio_zone_models::{ApiPlayer, Player};
use moosicbox_core::{
    sqlite::models::{ApiSource, AsModelResult, ToApi},
    types::PlaybackQuality,
};
use moosicbox_database::{AsId, DatabaseValue};
use moosicbox_json_utils::{database::ToValue as _, ParseError, ToValueType};
use moosicbox_library_models::{ApiLibraryTrack, ApiTrack};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SetSessionAudioZone {
    pub session_id: u64,
    pub audio_zone_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateSession {
    pub name: String,
    pub audio_zone_id: Option<u64>,
    pub playlist: CreateSessionPlaylist,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionPlaylist {
    pub tracks: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, AsRefStr, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PlaybackTarget {
    #[serde(rename_all = "camelCase")]
    AudioZone { audio_zone_id: u64 },
    #[serde(rename_all = "camelCase")]
    ConnectionOutput {
        connection_id: String,
        output_id: String,
    },
}

const DEFAULT_AUDIO_ZONE: PlaybackTarget = PlaybackTarget::AudioZone { audio_zone_id: 0 };
static DEFAULT_CONNECTION_OUTPUT: LazyLock<PlaybackTarget> =
    LazyLock::new(|| PlaybackTarget::ConnectionOutput {
        connection_id: String::new(),
        output_id: String::new(),
    });

impl PlaybackTarget {
    pub fn default_from_str(r#type: &str) -> Option<Self> {
        if DEFAULT_AUDIO_ZONE.as_ref() == r#type {
            Some(DEFAULT_AUDIO_ZONE)
        } else if DEFAULT_CONNECTION_OUTPUT.as_ref() == r#type {
            Some(DEFAULT_CONNECTION_OUTPUT.clone())
        } else {
            None
        }
    }
}

impl Default for PlaybackTarget {
    fn default() -> Self {
        Self::AudioZone { audio_zone_id: 0 }
    }
}

impl From<ApiPlaybackTarget> for PlaybackTarget {
    fn from(value: ApiPlaybackTarget) -> Self {
        match value {
            ApiPlaybackTarget::AudioZone { audio_zone_id } => Self::AudioZone { audio_zone_id },
            ApiPlaybackTarget::ConnectionOutput {
                connection_id,
                output_id,
            } => Self::ConnectionOutput {
                connection_id,
                output_id,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSession {
    pub session_id: u64,
    pub profile: String,
    pub playback_target: PlaybackTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seek: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist: Option<UpdateSessionPlaylist>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<PlaybackQuality>,
}

impl UpdateSession {
    #[must_use]
    pub const fn playback_updated(&self) -> bool {
        self.play.is_some()
            || self.stop.is_some()
            || self.active.is_some()
            || self.playing.is_some()
            || self.position.is_some()
            || self.volume.is_some()
            || self.seek.is_some()
            || self.playlist.is_some()
    }
}

impl From<ApiUpdateSession> for UpdateSession {
    fn from(value: ApiUpdateSession) -> Self {
        Self {
            session_id: value.session_id,
            profile: value.profile,
            playback_target: value.playback_target.into(),
            play: value.play,
            stop: value.stop,
            name: value.name,
            active: value.active,
            playing: value.playing,
            position: value.position,
            seek: value.seek,
            volume: value.volume,
            playlist: value.playlist.map(Into::into),
            quality: value.quality,
        }
    }
}

impl From<UpdateSession> for ApiUpdateSession {
    fn from(value: UpdateSession) -> Self {
        Self {
            session_id: value.session_id,
            profile: value.profile,
            playback_target: value.playback_target.into(),
            play: value.play,
            stop: value.stop,
            name: value.name,
            active: value.active,
            playing: value.playing,
            position: value.position,
            seek: value.seek,
            volume: value.volume,
            playlist: value.playlist.as_ref().map(ToApi::to_api),
            quality: value.quality,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionPlaylist {
    pub session_playlist_id: u64,
    pub tracks: Vec<UpdateSessionPlaylistTrack>,
}

impl From<ApiUpdateSessionPlaylist> for UpdateSessionPlaylist {
    fn from(value: ApiUpdateSessionPlaylist) -> Self {
        Self {
            session_playlist_id: value.session_playlist_id,
            tracks: value.tracks.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionPlaylistTrack {
    pub id: String,
    pub r#type: ApiSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl From<ApiTrack> for UpdateSessionPlaylistTrack {
    fn from(value: ApiTrack) -> Self {
        Self {
            id: value.track_id().to_string(),
            r#type: value.api_source(),
            data: value.data().map(|x| x.to_string()),
        }
    }
}

impl From<ApiUpdateSessionPlaylistTrack> for UpdateSessionPlaylistTrack {
    fn from(value: ApiUpdateSessionPlaylistTrack) -> Self {
        Self {
            id: value.id,
            r#type: value.r#type,
            data: value.data,
        }
    }
}

impl From<UpdateSessionPlaylistTrack> for SessionPlaylistTrack {
    fn from(value: UpdateSessionPlaylistTrack) -> Self {
        Self {
            id: value.id,
            r#type: value.r#type,
            data: value.data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApiUpdateSessionPlaylistTrack {
    pub id: String,
    pub r#type: ApiSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl ToApi<ApiUpdateSessionPlaylistTrack> for UpdateSessionPlaylistTrack {
    fn to_api(self) -> ApiUpdateSessionPlaylistTrack {
        ApiUpdateSessionPlaylistTrack {
            id: self.id,
            r#type: self.r#type,
            data: self.data,
        }
    }
}

impl ToApi<ApiUpdateSessionPlaylist> for UpdateSessionPlaylist {
    fn to_api(self) -> ApiUpdateSessionPlaylist {
        ApiUpdateSessionPlaylist {
            session_playlist_id: self.session_playlist_id,
            tracks: self
                .tracks
                .into_iter()
                .map(From::<UpdateSessionPlaylistTrack>::from)
                .map(|track: SessionPlaylistTrack| track.to_api())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, AsRefStr, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiPlaybackTarget {
    #[serde(rename_all = "camelCase")]
    AudioZone { audio_zone_id: u64 },
    #[serde(rename_all = "camelCase")]
    ConnectionOutput {
        connection_id: String,
        output_id: String,
    },
}

impl Default for ApiPlaybackTarget {
    fn default() -> Self {
        Self::AudioZone { audio_zone_id: 0 }
    }
}

impl From<PlaybackTarget> for ApiPlaybackTarget {
    fn from(value: PlaybackTarget) -> Self {
        match value {
            PlaybackTarget::AudioZone { audio_zone_id } => Self::AudioZone { audio_zone_id },
            PlaybackTarget::ConnectionOutput {
                connection_id,
                output_id,
            } => Self::ConnectionOutput {
                connection_id,
                output_id,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiUpdateSession {
    pub session_id: u64,
    pub profile: String,
    pub playback_target: ApiPlaybackTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seek: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist: Option<ApiUpdateSessionPlaylist>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<PlaybackQuality>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiUpdateSessionPlaylist {
    pub session_playlist_id: u64,
    pub tracks: Vec<ApiTrack>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSession {
    pub session_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub playing: bool,
    pub position: Option<u16>,
    pub seek: Option<u64>,
    pub volume: Option<f64>,
    pub playback_target: Option<PlaybackTarget>,
    pub playlist: SessionPlaylist,
}

impl ToValueType<Session> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<Session, ParseError> {
        let playback_target_type: Option<String> = self.to_value("playback_target")?;
        let playback_target_type =
            playback_target_type.and_then(|x| PlaybackTarget::default_from_str(&x));

        Ok(Session {
            id: self.to_value("id")?,
            name: self.to_value("name")?,
            active: self.to_value("active")?,
            playing: self.to_value("playing")?,
            position: self.to_value("position")?,
            seek: self.to_value("seek")?,
            volume: self.to_value("volume")?,
            playback_target: match playback_target_type {
                Some(PlaybackTarget::AudioZone { .. }) => Some(PlaybackTarget::AudioZone {
                    audio_zone_id: self.to_value("audio_zone_id")?,
                }),
                Some(PlaybackTarget::ConnectionOutput { .. }) => {
                    Some(PlaybackTarget::ConnectionOutput {
                        connection_id: self.to_value("connection_id")?,
                        output_id: self.to_value("output_id")?,
                    })
                }
                None => None,
            },
            ..Default::default()
        })
    }
}

impl AsId for Session {
    fn as_id(&self) -> DatabaseValue {
        #[allow(clippy::cast_possible_wrap)]
        DatabaseValue::Number(self.id as i64)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiSession {
    pub session_id: u64,
    pub name: String,
    pub active: bool,
    pub playing: bool,
    pub position: Option<u16>,
    pub seek: Option<u64>,
    pub volume: Option<f64>,
    pub playback_target: Option<PlaybackTarget>,
    pub playlist: ApiSessionPlaylist,
}

impl ToApi<ApiSession> for Session {
    fn to_api(self) -> ApiSession {
        ApiSession {
            session_id: self.id,
            name: self.name,
            active: self.active,
            playing: self.playing,
            position: self.position,
            seek: self.seek,
            volume: self.volume,
            playback_target: self.playback_target,
            playlist: self.playlist.to_api(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionPlaylist {
    pub id: u64,
    pub tracks: Vec<ApiTrack>,
}

impl ToValueType<SessionPlaylist> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<SessionPlaylist, ParseError> {
        Ok(SessionPlaylist {
            id: self.to_value("id")?,
            ..Default::default()
        })
    }
}

#[derive(Debug)]
pub struct SessionPlaylistTracks(pub Vec<SessionPlaylistTrack>);

impl AsId for SessionPlaylist {
    fn as_id(&self) -> DatabaseValue {
        #[allow(clippy::cast_possible_wrap)]
        DatabaseValue::Number(self.id as i64)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionPlaylistTrack {
    pub id: String,
    pub r#type: ApiSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl ToApi<ApiTrack> for SessionPlaylistTrack {
    #[allow(clippy::too_many_lines)]
    fn to_api(self) -> ApiTrack {
        match self.r#type {
            ApiSource::Library => {
                let id = self.id.parse::<u64>().expect("Invalid Library Track ID");
                ApiTrack::Library {
                    track_id: id,
                    data: self
                        .data
                        .and_then(|x| {
                            log::debug!("Mapping track data string: {x}");
                            serde_json::from_str(&x)
                                .map_err(|e| log::warn!("Failed to parse track json: {e:?}"))
                                .ok()
                        })
                        .unwrap_or_else(|| ApiLibraryTrack {
                            track_id: id,
                            ..Default::default()
                        }),
                }
            }
            #[cfg(feature = "tidal")]
            ApiSource::Tidal => {
                let id = self.id.parse::<u64>().expect("Invalid Tidal Track ID");
                match &self.data {
                    Some(data) => ApiTrack::Tidal {
                        track_id: id,
                        data: serde_json::from_str(data)
                            .expect("Failed to parse UpdateSessionPlaylistTrack data"),
                    },
                    None => ApiTrack::Tidal {
                        track_id: id,
                        data: self
                            .data
                            .and_then(|x| {
                                log::debug!("Mapping track data string: {x}");
                                serde_json::from_str(&x)
                                    .map_err(|e| log::warn!("Failed to parse track json: {e:?}"))
                                    .ok()
                            })
                            .unwrap_or_else(|| {
                                serde_json::json!({
                                    "id": id,
                                    "type": self.r#type,
                                })
                            }),
                    },
                }
            }
            #[cfg(feature = "qobuz")]
            ApiSource::Qobuz => {
                let id = self.id.parse::<u64>().expect("Invalid Qobuz Track ID");
                match &self.data {
                    Some(data) => ApiTrack::Qobuz {
                        track_id: id,
                        data: serde_json::from_str(data)
                            .expect("Failed to parse UpdateSessionPlaylistTrack data"),
                    },
                    None => ApiTrack::Qobuz {
                        track_id: id,
                        data: self
                            .data
                            .and_then(|x| {
                                log::debug!("Mapping track data string: {x}");
                                serde_json::from_str(&x)
                                    .map_err(|e| log::warn!("Failed to parse track json: {e:?}"))
                                    .ok()
                            })
                            .unwrap_or_else(|| {
                                serde_json::json!({
                                    "id": id,
                                    "type": self.r#type,
                                })
                            }),
                    },
                }
            }
            #[cfg(feature = "yt")]
            ApiSource::Yt => match &self.data {
                Some(data) => ApiTrack::Yt {
                    track_id: self.id,
                    data: serde_json::from_str(data)
                        .expect("Failed to parse UpdateSessionPlaylistTrack data"),
                },
                None => ApiTrack::Yt {
                    track_id: self.id.clone(),
                    data: self
                        .data
                        .and_then(|x| {
                            log::debug!("Mapping track data string: {x}");
                            serde_json::from_str(&x)
                                .map_err(|e| log::warn!("Failed to parse track json: {e:?}"))
                                .ok()
                        })
                        .unwrap_or_else(|| {
                            serde_json::json!({
                                "id": self.id,
                                "type": self.r#type,
                            })
                        }),
                },
            },
        }
    }
}

impl ToValueType<SessionPlaylistTrack> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<SessionPlaylistTrack, ParseError> {
        Ok(SessionPlaylistTrack {
            id: self.to_value("track_id")?,
            r#type: self.to_value("type")?,
            data: self.to_value("data")?,
        })
    }
}

impl AsModelResult<SessionPlaylistTrack, ParseError> for &moosicbox_database::Row {
    fn as_model(&self) -> Result<SessionPlaylistTrack, ParseError> {
        Ok(SessionPlaylistTrack {
            id: self.to_value("track_id")?,
            r#type: self.to_value("type")?,
            data: self.to_value("data")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiSessionPlaylistTrack {
    pub id: String,
    pub r#type: ApiSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl ToApi<ApiSessionPlaylistTrack> for SessionPlaylistTrack {
    fn to_api(self) -> ApiSessionPlaylistTrack {
        ApiSessionPlaylistTrack {
            id: self.id,
            r#type: self.r#type,
            data: self.data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiSessionPlaylist {
    pub session_playlist_id: u64,
    pub tracks: Vec<ApiTrack>,
}

impl ToApi<ApiSessionPlaylist> for SessionPlaylist {
    fn to_api(self) -> ApiSessionPlaylist {
        ApiSessionPlaylist {
            session_playlist_id: self.id,
            tracks: self.tracks,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegisterConnection {
    pub connection_id: String,
    pub name: String,
    pub players: Vec<RegisterPlayer>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub created: String,
    pub updated: String,
    pub players: Vec<Player>,
}

impl ToValueType<Connection> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<Connection, ParseError> {
        Ok(Connection {
            id: self.to_value("id")?,
            name: self.to_value("name")?,
            created: self.to_value("created")?,
            updated: self.to_value("updated")?,
            ..Default::default()
        })
    }
}

impl AsId for Connection {
    fn as_id(&self) -> DatabaseValue {
        DatabaseValue::String(self.id.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiConnection {
    pub connection_id: String,
    pub name: String,
    pub alive: bool,
    pub players: Vec<ApiPlayer>,
}

impl ToApi<ApiConnection> for Connection {
    fn to_api(self) -> ApiConnection {
        ApiConnection {
            connection_id: self.id,
            name: self.name,
            alive: false,
            players: self.players.iter().map(ToApi::to_api).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterPlayer {
    pub audio_output_id: String,
    pub name: String,
}
