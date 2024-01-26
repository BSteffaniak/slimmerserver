use std::{collections::HashMap, path::Path, sync::RwLock};

use moosicbox_core::{
    app::Db,
    sqlite::{
        db::{get_artist, DbError},
        models::ArtistId,
    },
};
use moosicbox_qobuz::QobuzArtist;
use moosicbox_tidal::TidalArtist;
use once_cell::sync::Lazy;
use thiserror::Error;

use crate::{
    fetch_and_save_bytes_from_remote_url, sanitize_filename, FetchAndSaveBytesFromRemoteUrlError,
};

pub enum ArtistCoverSource {
    LocalFilePath(String),
}

#[derive(Debug, Error)]
pub enum FetchArtistCoverError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    FetchAndSaveBytesFromRemoteUrl(#[from] FetchAndSaveBytesFromRemoteUrlError),
}

async fn get_or_fetch_artist_cover_from_remote_url(
    url: &str,
    source: &str,
    artist_name: &str,
) -> Result<String, FetchArtistCoverError> {
    static IMAGE_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

    let path = moosicbox_config::get_cache_dir_path()
        .expect("Failed to get cache directory")
        .join(source)
        .join(sanitize_filename(artist_name));

    let filename = "artist.jpg";
    let file_path = path.join(filename);

    if Path::exists(&file_path) {
        Ok(file_path.to_str().unwrap().to_string())
    } else {
        Ok(
            fetch_and_save_bytes_from_remote_url(&IMAGE_CLIENT, &file_path, url)
                .await?
                .to_str()
                .unwrap()
                .to_string(),
        )
    }
}

#[derive(Debug, Error)]
pub enum ArtistCoverError {
    #[error("Artist cover not found for album: {0:?}")]
    NotFound(ArtistId),
    #[error("Invalid source")]
    InvalidSource,
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    FetchArtistCover(#[from] FetchArtistCoverError),
    #[error(transparent)]
    TidalArtist(#[from] moosicbox_tidal::TidalArtistError),
    #[error(transparent)]
    QobuzArtist(#[from] moosicbox_qobuz::QobuzArtistError),
    #[error("Failed to read file with path: {0} ({1})")]
    File(String, String),
}

pub async fn get_artist_cover(
    artist_id: ArtistId,
    db: Db,
) -> Result<ArtistCoverSource, ArtistCoverError> {
    let path = match &artist_id {
        ArtistId::Library(library_artist_id) => get_artist(
            &db.library.lock().as_ref().unwrap().inner,
            *library_artist_id,
        )?
        .and_then(|artist| artist.cover)
        .ok_or(ArtistCoverError::NotFound(artist_id.clone()))?,
        ArtistId::Tidal(tidal_artist_id) => {
            static ARTIST_CACHE: Lazy<RwLock<HashMap<u64, TidalArtist>>> =
                Lazy::new(|| RwLock::new(HashMap::new()));

            let artist = if let Some(artist) = {
                let binding = ARTIST_CACHE.read().unwrap();
                binding.get(tidal_artist_id).cloned()
            } {
                artist
            } else {
                let artist =
                    moosicbox_tidal::artist(&db, *tidal_artist_id, None, None, None, None).await?;
                ARTIST_CACHE
                    .write()
                    .as_mut()
                    .unwrap()
                    .insert(*tidal_artist_id, artist.clone());
                artist
            };

            let cover = artist
                .picture_url(750)
                .ok_or(ArtistCoverError::NotFound(artist_id.clone()))?;

            get_or_fetch_artist_cover_from_remote_url(&cover, "tidal", &artist.name).await?
        }
        ArtistId::Qobuz(qobuz_artist_id) => {
            static ARTIST_CACHE: Lazy<RwLock<HashMap<u64, QobuzArtist>>> =
                Lazy::new(|| RwLock::new(HashMap::new()));

            let artist = if let Some(artist) = {
                let binding = ARTIST_CACHE.read().unwrap();
                binding.get(qobuz_artist_id).cloned()
            } {
                artist
            } else {
                let artist = moosicbox_qobuz::artist(&db, *qobuz_artist_id, None, None).await?;
                ARTIST_CACHE
                    .write()
                    .as_mut()
                    .unwrap()
                    .insert(*qobuz_artist_id, artist.clone());
                artist
            };

            let cover = artist
                .cover_url()
                .ok_or(ArtistCoverError::NotFound(artist_id.clone()))?;

            get_or_fetch_artist_cover_from_remote_url(&cover, "qobuz", &artist.name).await?
        }
    };

    Ok(ArtistCoverSource::LocalFilePath(path))
}
