use std::str::FromStr as _;

use moosicbox_core::sqlite::db::{get_album_version_qualities, DbError};
use moosicbox_core::{
    sqlite::models::{AlbumSource, ApiSource, ApiSources, TrackApiSource},
    types::AudioFormat,
};
use moosicbox_database::{AsId, Database, DatabaseValue};
use moosicbox_json_utils::{database::ToValue as _, MissingValue, ParseError, ToValueType};

use crate::{sort_album_versions, LibraryAlbum, LibraryAlbumType, LibraryArtist, LibraryTrack};

impl AsId for LibraryTrack {
    fn as_id(&self) -> DatabaseValue {
        DatabaseValue::Number(self.id.try_into().unwrap())
    }
}

impl moosicbox_core::sqlite::models::AsModel<LibraryArtist> for &moosicbox_database::Row {
    fn as_model(&self) -> LibraryArtist {
        moosicbox_core::sqlite::models::AsModelResult::as_model(self).unwrap()
    }
}

impl ToValueType<LibraryArtist> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<LibraryArtist, ParseError> {
        Ok(LibraryArtist {
            id: self.to_value("id")?,
            title: self.to_value("title")?,
            cover: self.to_value("cover")?,
            tidal_id: self.to_value("tidal_id")?,
            qobuz_id: self.to_value("qobuz_id")?,
            yt_id: self.to_value("yt_id")?,
        })
    }
}

impl moosicbox_core::sqlite::models::AsModelResult<LibraryArtist, ParseError>
    for &moosicbox_database::Row
{
    fn as_model(&self) -> Result<LibraryArtist, ParseError> {
        Ok(LibraryArtist {
            id: self.to_value("id")?,
            title: self.to_value("title")?,
            cover: self.to_value("cover")?,
            tidal_id: self.to_value("tidal_id")?,
            qobuz_id: self.to_value("qobuz_id")?,
            yt_id: self.to_value("yt_id")?,
        })
    }
}

impl AsId for LibraryArtist {
    fn as_id(&self) -> DatabaseValue {
        DatabaseValue::Number(self.id.try_into().unwrap())
    }
}

impl MissingValue<LibraryAlbumType> for &moosicbox_database::Row {}
impl ToValueType<LibraryAlbumType> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<LibraryAlbumType, ParseError> {
        self.get("album_type")
            .ok_or_else(|| ParseError::MissingValue("album_type".into()))?
            .to_value_type()
    }
}
impl ToValueType<LibraryAlbumType> for DatabaseValue {
    fn to_value_type(self) -> Result<LibraryAlbumType, ParseError> {
        LibraryAlbumType::from_str(
            self.as_str()
                .ok_or_else(|| ParseError::ConvertType("AlbumType".into()))?,
        )
        .map_err(|_| ParseError::ConvertType("AlbumType".into()))
    }
}

impl moosicbox_core::sqlite::models::AsModel<LibraryAlbum> for &moosicbox_database::Row {
    fn as_model(&self) -> LibraryAlbum {
        moosicbox_core::sqlite::models::AsModelResult::as_model(self).unwrap()
    }
}

impl MissingValue<LibraryAlbum> for &moosicbox_database::Row {}
impl ToValueType<LibraryAlbum> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<LibraryAlbum, ParseError> {
        #[cfg(any(feature = "tidal", feature = "qobuz", feature = "yt"))]
        use moosicbox_core::sqlite::models::Id;

        let album_type: Option<LibraryAlbumType> = self.to_value("album_type")?;

        #[cfg(feature = "tidal")]
        let tidal_id: Option<Id> = self.to_value("tidal_id")?;
        #[cfg(feature = "tidal")]
        let tidal_artist_id: Option<Id> = self.to_value("tidal_artist_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_id: Option<Id> = self.to_value("qobuz_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_artist_id: Option<Id> = self.to_value("qobuz_artist_id")?;
        #[cfg(feature = "yt")]
        let yt_id: Option<Id> = self.to_value("yt_id")?;
        #[cfg(feature = "yt")]
        let yt_artist_id: Option<Id> = self.to_value("yt_artist_id")?;

        let id = self.to_value("id")?;
        let artist_id = self.to_value("artist_id")?;

        Ok(LibraryAlbum {
            id,
            artist: self.to_value("artist").unwrap_or_default(),
            artist_id,
            title: self.to_value("title")?,
            album_type: album_type.unwrap_or_default(),
            date_released: self.to_value("date_released")?,
            date_added: self.to_value("date_added")?,
            artwork: self.to_value("artwork")?,
            directory: self.to_value("directory")?,
            source: AlbumSource::Local,
            blur: self.to_value("blur")?,
            versions: vec![],
            album_sources: {
                #[allow(unused_mut)]
                let mut sources = ApiSources::default().with_source(ApiSource::Library, id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_id);
                }

                sources
            },
            artist_sources: {
                #[allow(unused_mut)]
                let mut sources =
                    ApiSources::default().with_source(ApiSource::Library, artist_id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_artist_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_artist_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_artist_id);
                }

                sources
            },
        })
    }
}

impl moosicbox_core::sqlite::models::AsModelResult<LibraryAlbum, ParseError>
    for &moosicbox_database::Row
{
    fn as_model(&self) -> Result<LibraryAlbum, ParseError> {
        #[cfg(any(feature = "tidal", feature = "qobuz", feature = "yt"))]
        use moosicbox_core::sqlite::models::Id;

        let album_type: Option<LibraryAlbumType> = self.to_value("album_type")?;

        #[cfg(feature = "tidal")]
        let tidal_id: Option<Id> = self.to_value("tidal_id")?;
        #[cfg(feature = "tidal")]
        let tidal_artist_id: Option<Id> = self.to_value("tidal_artist_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_id: Option<Id> = self.to_value("qobuz_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_artist_id: Option<Id> = self.to_value("qobuz_artist_id")?;
        #[cfg(feature = "yt")]
        let yt_id: Option<Id> = self.to_value("yt_id")?;
        #[cfg(feature = "yt")]
        let yt_artist_id: Option<Id> = self.to_value("yt_artist_id")?;

        let id = self.to_value("id")?;
        let artist_id = self.to_value("artist_id")?;

        Ok(LibraryAlbum {
            id,
            artist: self.to_value("artist").unwrap_or_default(),
            artist_id,
            title: self.to_value("title")?,
            album_type: album_type.unwrap_or_default(),
            date_released: self.to_value("date_released")?,
            date_added: self.to_value("date_added")?,
            artwork: self.to_value("artwork")?,
            directory: self.to_value("directory")?,
            source: AlbumSource::Local,
            blur: self.to_value("blur")?,
            versions: vec![],
            album_sources: {
                #[allow(unused_mut)]
                let mut sources = ApiSources::default().with_source(ApiSource::Library, id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_id);
                }

                sources
            },
            artist_sources: {
                #[allow(unused_mut)]
                let mut sources =
                    ApiSources::default().with_source(ApiSource::Library, artist_id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_artist_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_artist_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_artist_id);
                }

                sources
            },
        })
    }
}

impl moosicbox_core::sqlite::models::AsModelResultMapped<LibraryAlbum, DbError>
    for Vec<moosicbox_database::Row>
{
    #[allow(clippy::too_many_lines)]
    fn as_model_mapped(&self) -> Result<Vec<LibraryAlbum>, DbError> {
        let mut results: Vec<LibraryAlbum> = vec![];
        let mut last_album_id = 0;

        for row in self {
            let album_id: u64 = row
                .get("album_id")
                .ok_or(DbError::InvalidRequest)?
                .try_into()
                .map_err(|_| DbError::InvalidRequest)?;

            if album_id != last_album_id {
                if let Some(ref mut album) = results.last_mut() {
                    log::trace!(
                        "Sorting versions for album id={} count={}",
                        album.id,
                        album.versions.len()
                    );
                    sort_album_versions(&mut album.versions);
                }
                match row.to_value_type() {
                    Ok(album) => {
                        results.push(album);
                    }
                    Err(err) => {
                        log::error!("Failed to parse Album for album id={}: {err:?}", album_id);
                        continue;
                    }
                }
                last_album_id = album_id;
            }

            if let Some(album) = results.last_mut() {
                if let Some(_source) = row.get("source") {
                    match row.to_value_type() {
                        Ok(version) => {
                            album.versions.push(version);
                            log::trace!(
                                "Added version to album id={} count={}",
                                album.id,
                                album.versions.len()
                            );
                        }
                        Err(err) => {
                            log::error!(
                                "Failed to parse AlbumVersionQuality for album id={}: {err:?}",
                                album.id
                            );
                        }
                    }
                } else {
                    #[cfg(feature = "tidal")]
                    if album
                        .album_sources
                        .iter()
                        .any(|x| x.source == ApiSource::Tidal)
                    {
                        album
                            .versions
                            .push(moosicbox_core::sqlite::models::AlbumVersionQuality {
                                format: None,
                                bit_depth: None,
                                sample_rate: None,
                                channels: None,
                                source: TrackApiSource::Tidal,
                            });
                        log::trace!(
                            "Added Tidal version to album id={} count={}",
                            album.id,
                            album.versions.len()
                        );
                    }
                    #[cfg(feature = "qobuz")]
                    if album
                        .album_sources
                        .iter()
                        .any(|x| x.source == ApiSource::Qobuz)
                    {
                        album
                            .versions
                            .push(moosicbox_core::sqlite::models::AlbumVersionQuality {
                                format: None,
                                bit_depth: None,
                                sample_rate: None,
                                channels: None,
                                source: TrackApiSource::Qobuz,
                            });
                        log::trace!(
                            "Added Qobuz version to album id={} count={}",
                            album.id,
                            album.versions.len()
                        );
                    }
                    #[cfg(feature = "yt")]
                    if album
                        .album_sources
                        .iter()
                        .any(|x| x.source == ApiSource::Yt)
                    {
                        album
                            .versions
                            .push(moosicbox_core::sqlite::models::AlbumVersionQuality {
                                format: None,
                                bit_depth: None,
                                sample_rate: None,
                                channels: None,
                                source: TrackApiSource::Yt,
                            });
                        log::trace!(
                            "Added Yt version to album id={} count={}",
                            album.id,
                            album.versions.len()
                        );
                    }
                }
            }
        }

        if let Some(ref mut album) = results.last_mut() {
            log::trace!(
                "Sorting versions for last album id={} count={}",
                album.id,
                album.versions.len()
            );
            sort_album_versions(&mut album.versions);
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl moosicbox_core::sqlite::models::AsModelQuery<LibraryAlbum> for &moosicbox_database::Row {
    async fn as_model_query(
        &self,
        db: std::sync::Arc<Box<dyn Database>>,
    ) -> Result<LibraryAlbum, DbError> {
        #[cfg(any(feature = "tidal", feature = "qobuz", feature = "yt"))]
        use moosicbox_core::sqlite::models::Id;

        #[cfg(feature = "tidal")]
        let tidal_id: Option<Id> = self.to_value("tidal_id")?;
        #[cfg(feature = "tidal")]
        let tidal_artist_id: Option<Id> = self.to_value("tidal_artist_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_id: Option<Id> = self.to_value("qobuz_id")?;
        #[cfg(feature = "qobuz")]
        let qobuz_artist_id: Option<Id> = self.to_value("qobuz_artist_id")?;
        #[cfg(feature = "yt")]
        let yt_id: Option<Id> = self.to_value("yt_id")?;
        #[cfg(feature = "yt")]
        let yt_artist_id: Option<Id> = self.to_value("yt_artist_id")?;

        let id = self.to_value("id")?;
        let artist_id = self.to_value("artist_id")?;
        let album_type: Option<LibraryAlbumType> = self.to_value("album_type")?;

        Ok(LibraryAlbum {
            id,
            artist: self
                .to_value::<Option<String>>("artist")?
                .unwrap_or_default(),
            artist_id,
            title: self.to_value("title")?,
            album_type: album_type.unwrap_or_default(),
            date_released: self.to_value("date_released")?,
            date_added: self.to_value("date_added")?,
            artwork: self.to_value("artwork")?,
            directory: self.to_value("directory")?,
            source: AlbumSource::Local,
            blur: self.to_value("blur")?,
            versions: get_album_version_qualities(&db.into(), id).await?,
            album_sources: {
                #[allow(unused_mut)]
                let mut sources = ApiSources::default().with_source(ApiSource::Library, id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_id);
                }

                sources
            },
            artist_sources: {
                #[allow(unused_mut)]
                let mut sources =
                    ApiSources::default().with_source(ApiSource::Library, artist_id.into());

                #[cfg(feature = "tidal")]
                {
                    sources = sources.with_source_opt(ApiSource::Tidal, tidal_artist_id);
                }
                #[cfg(feature = "qobuz")]
                {
                    sources = sources.with_source_opt(ApiSource::Qobuz, qobuz_artist_id);
                }
                #[cfg(feature = "yt")]
                {
                    sources = sources.with_source_opt(ApiSource::Yt, yt_artist_id);
                }

                sources
            },
        })
    }
}

impl AsId for LibraryAlbum {
    fn as_id(&self) -> DatabaseValue {
        DatabaseValue::Number(self.id.try_into().unwrap())
    }
}

impl moosicbox_core::sqlite::models::AsModel<LibraryTrack> for &moosicbox_database::Row {
    fn as_model(&self) -> LibraryTrack {
        moosicbox_core::sqlite::models::AsModelResult::as_model(self).unwrap()
    }
}

impl ToValueType<LibraryTrack> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<LibraryTrack, ParseError> {
        let album_type: Option<LibraryAlbumType> = self.to_value("album_type")?;
        Ok(LibraryTrack {
            id: self.to_value("id")?,
            number: self.to_value("number")?,
            title: self.to_value("title")?,
            duration: self.to_value("duration")?,
            album: self.to_value("album").unwrap_or_default(),
            album_id: self.to_value("album_id")?,
            album_type: album_type.unwrap_or_default(),
            date_released: self.to_value("date_released").unwrap_or_default(),
            date_added: self.to_value("date_added").unwrap_or_default(),
            artist: self.to_value("artist").unwrap_or_default(),
            artist_id: self.to_value("artist_id").unwrap_or_default(),
            file: self.to_value("file")?,
            artwork: self.to_value("artwork").unwrap_or_default(),
            blur: self.to_value("blur").unwrap_or_default(),
            bytes: self.to_value("bytes").unwrap_or_default(),
            format: self
                .to_value::<Option<String>>("format")
                .unwrap_or(None)
                .map(|s| {
                    AudioFormat::from_str(&s)
                        .map_err(|_e| ParseError::ConvertType(format!("Invalid format: {s}")))
                })
                .transpose()?,
            bit_depth: self.to_value("bit_depth").unwrap_or_default(),
            audio_bitrate: self.to_value("audio_bitrate").unwrap_or_default(),
            overall_bitrate: self.to_value("overall_bitrate").unwrap_or_default(),
            sample_rate: self.to_value("sample_rate").unwrap_or_default(),
            channels: self.to_value("channels").unwrap_or_default(),
            source: TrackApiSource::from_str(&self.to_value::<String>("source")?)
                .expect("Missing source"),
            api_source: ApiSource::Library,
            qobuz_id: self.to_value("qobuz_id")?,
            tidal_id: self.to_value("tidal_id")?,
            yt_id: self.to_value("yt_id")?,
        })
    }
}

impl moosicbox_core::sqlite::models::AsModelResult<LibraryTrack, ParseError>
    for &moosicbox_database::Row
{
    fn as_model(&self) -> Result<LibraryTrack, ParseError> {
        let album_type: Option<LibraryAlbumType> = self.to_value("album_type")?;
        Ok(LibraryTrack {
            id: self.to_value("id")?,
            number: self.to_value("number")?,
            title: self.to_value("title")?,
            duration: self.to_value("duration")?,
            album: self.to_value("album").unwrap_or_default(),
            album_id: self.to_value("album_id")?,
            album_type: album_type.unwrap_or_default(),
            date_released: self.to_value("date_released").unwrap_or_default(),
            date_added: self.to_value("date_added").unwrap_or_default(),
            artist: self.to_value("artist").unwrap_or_default(),
            artist_id: self.to_value("artist_id").unwrap_or_default(),
            file: self.to_value("file")?,
            artwork: self.to_value("artwork").unwrap_or_default(),
            blur: self.to_value("blur").unwrap_or_default(),
            bytes: self.to_value("bytes").unwrap_or_default(),
            format: self
                .to_value::<Option<String>>("format")
                .unwrap_or(None)
                .map(|s| {
                    AudioFormat::from_str(&s)
                        .map_err(|_e| ParseError::ConvertType(format!("Invalid format: {s}")))
                })
                .transpose()?,
            bit_depth: self.to_value("bit_depth").unwrap_or_default(),
            audio_bitrate: self.to_value("audio_bitrate").unwrap_or_default(),
            overall_bitrate: self.to_value("overall_bitrate").unwrap_or_default(),
            sample_rate: self.to_value("sample_rate").unwrap_or_default(),
            channels: self.to_value("channels").unwrap_or_default(),
            source: TrackApiSource::from_str(&self.to_value::<String>("source")?)
                .expect("Missing source"),
            api_source: ApiSource::Library,
            qobuz_id: self.to_value("qobuz_id")?,
            tidal_id: self.to_value("tidal_id")?,
            yt_id: self.to_value("yt_id")?,
        })
    }
}
