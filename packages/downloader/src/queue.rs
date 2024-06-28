use std::{pin::Pin, sync::Arc, time::Duration};

use futures::Future;
use lazy_static::lazy_static;
use moosicbox_core::sqlite::db::DbError;
use moosicbox_database::{query::*, Database, DatabaseError, DatabaseValue, Row};
use thiserror::Error;
use tokio::{
    sync::{Mutex, RwLock},
    task::{JoinError, JoinHandle},
};

use crate::{
    db::models::{DownloadItem, DownloadTask, DownloadTaskState},
    DownloadAlbumError, DownloadTrackError, Downloader,
};

lazy_static! {
    static ref TIMEOUT_DURATION: Option<Duration> = Some(Duration::from_secs(30));
}

#[derive(Debug, Error)]
pub enum UpdateTaskError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("No database")]
    NoDatabase,
    #[error("No row")]
    NoRow,
}

#[derive(Debug, Error)]
pub enum ProcessDownloadQueueError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    UpdateTask(#[from] UpdateTaskError),
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error(transparent)]
    DownloadTrack(#[from] DownloadTrackError),
    #[error(transparent)]
    DownloadAlbum(#[from] DownloadAlbumError),
    #[error("No database")]
    NoDatabase,
    #[error("No downloader")]
    NoDownloader,
}

#[derive(Debug, Clone, PartialEq)]
struct ProcessDownloadTaskResponse {
    task_id: u64,
}

#[derive(Debug)]
struct DownloadQueueState {
    tasks: Vec<DownloadTask>,
    results: Vec<Result<ProcessDownloadTaskResponse, ProcessDownloadQueueError>>,
}

impl DownloadQueueState {
    fn new() -> Self {
        Self {
            tasks: vec![],
            results: vec![],
        }
    }

    fn add_task_to_queue(&mut self, task: DownloadTask) {
        self.tasks.push(task);
    }

    fn add_tasks_to_queue(&mut self, tasks: Vec<DownloadTask>) {
        self.tasks.extend(tasks);
    }

    fn finish_task(&mut self, task: &DownloadTask) {
        self.tasks
            .retain(|x| !(task.file_path == x.file_path && task.item == x.item));
    }
}

#[derive(Clone)]
pub enum GenericProgressEvent {
    Size { bytes: Option<u64> },
    Speed { bytes_per_second: f64 },
    BytesRead { read: usize, total: usize },
}

#[derive(Clone)]
pub enum ProgressEvent {
    Size {
        task: DownloadTask,
        bytes: Option<u64>,
    },
    Speed {
        task: DownloadTask,
        bytes_per_second: f64,
    },
    BytesRead {
        task: DownloadTask,
        read: usize,
        total: usize,
    },
    State {
        task: DownloadTask,
        state: DownloadTaskState,
    },
}

pub type ProgressListenerFut = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type ProgressListener =
    Box<dyn (FnMut(GenericProgressEvent) -> ProgressListenerFut) + Send + Sync>;
pub type ProgressListenerRefFut = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type ProgressListenerRef =
    Box<dyn (Fn(&ProgressEvent) -> ProgressListenerRefFut) + Send + Sync>;

#[derive(Clone)]
pub struct DownloadQueue {
    progress_listeners: Vec<Arc<ProgressListenerRef>>,
    database: Option<Arc<Box<dyn Database>>>,
    downloader: Option<Arc<Box<dyn Downloader + Send + Sync>>>,
    state: Arc<RwLock<DownloadQueueState>>,
    #[allow(clippy::type_complexity)]
    join_handle: Arc<Mutex<Option<JoinHandle<Result<(), ProcessDownloadQueueError>>>>>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self {
            progress_listeners: vec![],
            database: None,
            downloader: None,
            state: Arc::new(RwLock::new(DownloadQueueState::new())),
            join_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn has_database(&self) -> bool {
        self.database.is_some()
    }

    pub fn with_database(&mut self, database: Arc<Box<dyn Database>>) -> Self {
        self.database.replace(database);
        self.clone()
    }

    pub fn has_downloader(&self) -> bool {
        self.downloader.is_some()
    }

    pub fn with_downloader(&mut self, downloader: Box<dyn Downloader + Send + Sync>) -> Self {
        self.downloader.replace(Arc::new(downloader));
        self.clone()
    }

    pub fn add_progress_listener(&mut self, listener: ProgressListenerRef) -> Self {
        self.progress_listeners.push(Arc::new(listener));
        self.clone()
    }

    pub fn speed(&self) -> Option<f64> {
        self.downloader
            .clone()
            .and_then(|downloader| downloader.speed())
    }

    pub async fn add_task_to_queue(&mut self, task: DownloadTask) {
        self.state.write().await.add_task_to_queue(task);
    }

    pub async fn add_tasks_to_queue(&mut self, tasks: Vec<DownloadTask>) {
        self.state.write().await.add_tasks_to_queue(tasks);
    }

    pub fn process(&mut self) -> JoinHandle<Result<(), ProcessDownloadQueueError>> {
        let join_handle = self.join_handle.clone();
        let mut this = self.clone();

        tokio::spawn(async move {
            let mut handle = join_handle.lock().await;

            if let Some(handle) = handle.as_mut() {
                if !handle.is_finished() {
                    handle.await??;
                }
            }

            handle.replace(tokio::spawn(async move {
                this.process_inner().await?;
                Ok(())
            }));

            Ok::<_, ProcessDownloadQueueError>(())
        })
    }

    #[allow(unused)]
    async fn shutdown(&mut self) -> Result<(), ProcessDownloadQueueError> {
        let mut handle = self.join_handle.lock().await;

        if let Some(handle) = handle.as_mut() {
            if handle.is_finished() {
                Ok(())
            } else {
                Ok(handle.await??)
            }
        } else {
            Ok(())
        }
    }

    async fn process_inner(&mut self) -> Result<(), ProcessDownloadQueueError> {
        while let Some(mut task) = {
            let state = self.state.as_ref().read().await;
            state.tasks.first().cloned()
        } {
            let result = self.process_task(&mut task).await;

            let mut state = self.state.write().await;

            if let Err(ref err) = result {
                log::error!("Encountered error when processing task in DownloadQueue: {err:?}");
                self.update_task_state(&mut task, DownloadTaskState::Error)
                    .await?;
            }

            state.results.push(result);
            state.finish_task(&task);
        }

        Ok(())
    }

    async fn update_task_state(
        &self,
        task: &mut DownloadTask,
        state: DownloadTaskState,
    ) -> Result<Row, UpdateTaskError> {
        task.state = state;

        let row = self
            .update_task(
                task.id,
                &[(
                    "state",
                    DatabaseValue::String(task.state.as_ref().to_string()),
                )],
            )
            .await;

        for listener in self.progress_listeners.iter() {
            listener(&ProgressEvent::State {
                task: task.clone(),
                state,
            })
            .await;
        }

        row
    }

    async fn update_task(
        &self,
        task_id: u64,
        values: &[(&str, DatabaseValue)],
    ) -> Result<Row, UpdateTaskError> {
        let db = self.database.clone().ok_or(UpdateTaskError::NoDatabase)?;

        db.update("download_tasks")
            .where_eq("id", task_id)
            .values(values.to_vec())
            .execute_first(&**db)
            .await?
            .ok_or(UpdateTaskError::NoRow)
    }

    async fn process_task(
        &mut self,
        task: &mut DownloadTask,
    ) -> Result<ProcessDownloadTaskResponse, ProcessDownloadQueueError> {
        log::debug!("Processing task {task:?}");

        self.update_task_state(task, DownloadTaskState::Started)
            .await?;

        let mut task_size = None;
        let database = self
            .database
            .clone()
            .ok_or(ProcessDownloadQueueError::NoDatabase)?;

        let task_id = task.id;
        let listeners = self.progress_listeners.clone();
        let send_task = task.clone();

        let on_progress = Box::new(move |event: GenericProgressEvent| {
            let database = database.clone();
            let send_task = send_task.clone();
            let listeners = listeners.clone();
            Box::pin(async move {
                match event.clone() {
                    GenericProgressEvent::Size { bytes, .. } => {
                        log::debug!("Got size of task: {bytes:?}");
                        if let Some(size) = bytes {
                            task_size.replace(size);
                            let database = database.clone();
                            tokio::task::spawn(async move {
                                if let Err(err) = database
                                    .update("download_tasks")
                                    .where_eq("id", task_id)
                                    .value("total_bytes", size)
                                    .execute_first(&**database)
                                    .await
                                {
                                    log::error!("Failed to set DownloadTask total_bytes: {err:?}");
                                }
                            });
                        }
                    }
                    GenericProgressEvent::Speed { .. } => {}
                    GenericProgressEvent::BytesRead { .. } => {}
                }

                let event = match event {
                    GenericProgressEvent::Size { bytes } => ProgressEvent::Size {
                        task: send_task.clone(),
                        bytes,
                    },
                    GenericProgressEvent::Speed { bytes_per_second } => ProgressEvent::Speed {
                        task: send_task.clone(),
                        bytes_per_second,
                    },
                    GenericProgressEvent::BytesRead { read, total } => ProgressEvent::BytesRead {
                        task: send_task.clone(),
                        read,
                        total,
                    },
                };
                for listener in listeners.iter() {
                    listener(&event).await;
                }
            }) as ProgressListenerFut
        });

        let downloader = self
            .downloader
            .clone()
            .ok_or(ProcessDownloadQueueError::NoDownloader)?;

        match task.item {
            DownloadItem::Track {
                track_id,
                quality,
                source,
            } => {
                downloader
                    .download_track_id(
                        &task.file_path,
                        track_id,
                        quality,
                        source,
                        on_progress,
                        *TIMEOUT_DURATION,
                    )
                    .await?
            }
            DownloadItem::AlbumCover(album_id) => {
                downloader
                    .download_album_cover(&task.file_path, album_id, on_progress)
                    .await?;
            }
            DownloadItem::ArtistCover(album_id) => {
                downloader
                    .download_artist_cover(&task.file_path, album_id, on_progress)
                    .await?;
            }
        }

        if let Some(size) = task_size {
            task.total_bytes.replace(size);
        }

        self.update_task_state(task, DownloadTaskState::Finished)
            .await?;

        Ok(ProcessDownloadTaskResponse { task_id: task.id })
    }
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DownloadQueue {
    fn drop(&mut self) {
        let handle = self.join_handle.clone();

        tokio::task::spawn(async move {
            let mut handle = handle.lock().await;
            if let Some(handle) = handle.as_mut() {
                if !handle.is_finished() {
                    if let Err(err) = handle.await {
                        log::error!("Failed to drop DownloadQueue: {err:?}");
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use moosicbox_database::{query::*, Row};
    use moosicbox_files::files::track::TrackAudioQuality;
    use pretty_assertions::assert_eq;

    use crate::db::models::{DownloadApiSource, DownloadItem, DownloadTaskState};

    use super::*;

    struct TestDownloader {}

    #[async_trait]
    impl Downloader for TestDownloader {
        async fn download_track_id(
            &self,
            _path: &str,
            _track_id: u64,
            _quality: moosicbox_files::files::track::TrackAudioQuality,
            _source: crate::db::models::DownloadApiSource,
            _on_size: ProgressListener,
            _timeout_duration: Option<Duration>,
        ) -> Result<(), DownloadTrackError> {
            Ok(())
        }

        async fn download_album_cover(
            &self,
            _path: &str,
            _album_id: u64,
            _on_size: ProgressListener,
        ) -> Result<(), DownloadAlbumError> {
            Ok(())
        }

        async fn download_artist_cover(
            &self,
            _path: &str,
            _album_id: u64,
            _on_size: ProgressListener,
        ) -> Result<(), DownloadAlbumError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct TestDatabase {}

    #[async_trait]
    impl Database for TestDatabase {
        async fn query(&self, _query: &SelectQuery<'_>) -> Result<Vec<Row>, DatabaseError> {
            Ok(vec![])
        }

        async fn query_first(
            &self,
            _query: &SelectQuery<'_>,
        ) -> Result<Option<Row>, DatabaseError> {
            Ok(None)
        }

        async fn exec_delete(
            &self,
            _statement: &DeleteStatement<'_>,
        ) -> Result<Vec<Row>, DatabaseError> {
            Ok(vec![])
        }

        async fn exec_insert(
            &self,
            _statement: &InsertStatement<'_>,
        ) -> Result<Row, DatabaseError> {
            Ok(Row { columns: vec![] })
        }

        async fn exec_update(
            &self,
            _statement: &UpdateStatement<'_>,
        ) -> Result<Vec<Row>, DatabaseError> {
            Ok(vec![Row { columns: vec![] }])
        }

        async fn exec_update_first(
            &self,
            _statement: &UpdateStatement<'_>,
        ) -> Result<Option<Row>, DatabaseError> {
            Ok(Some(Row { columns: vec![] }))
        }

        async fn exec_upsert(
            &self,
            _statement: &UpsertStatement<'_>,
        ) -> Result<Vec<Row>, DatabaseError> {
            Ok(vec![Row { columns: vec![] }])
        }

        async fn exec_upsert_first(
            &self,
            _statement: &UpsertStatement<'_>,
        ) -> Result<Row, DatabaseError> {
            Ok(Row { columns: vec![] })
        }

        async fn exec_upsert_multi(
            &self,
            _statement: &UpsertMultiStatement<'_>,
        ) -> Result<Vec<Row>, DatabaseError> {
            Ok(vec![Row { columns: vec![] }])
        }
    }

    fn new_queue() -> DownloadQueue {
        DownloadQueue::new()
            .with_database(Arc::new(Box::new(TestDatabase {})))
            .with_downloader(Box::new(TestDownloader {}))
    }

    #[test_log::test(tokio::test)]
    async fn test_can_process_single_track_download_task() {
        let mut queue = new_queue();

        queue
            .add_task_to_queue(DownloadTask {
                id: 1,
                state: DownloadTaskState::Pending,
                item: DownloadItem::Track {
                    track_id: 1,
                    source: DownloadApiSource::Tidal,
                    quality: TrackAudioQuality::FlacHighestRes,
                },
                file_path: "".into(),
                created: "".into(),
                updated: "".into(),
                total_bytes: None,
            })
            .await;

        queue.process().await.unwrap().unwrap();
        queue.shutdown().await.unwrap();

        let responses = queue
            .state
            .read()
            .await
            .results
            .iter()
            .map(|result| result.as_ref().ok().cloned())
            .collect::<Vec<_>>();

        assert_eq!(
            responses,
            vec![Some(ProcessDownloadTaskResponse { task_id: 1 })]
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_can_process_multiple_track_download_tasks() {
        let mut queue = new_queue();

        queue
            .add_tasks_to_queue(vec![
                DownloadTask {
                    id: 1,
                    state: DownloadTaskState::Pending,
                    item: DownloadItem::Track {
                        track_id: 1,
                        source: DownloadApiSource::Tidal,
                        quality: TrackAudioQuality::FlacHighestRes,
                    },
                    file_path: "".into(),
                    created: "".into(),
                    updated: "".into(),
                    total_bytes: None,
                },
                DownloadTask {
                    id: 2,
                    state: DownloadTaskState::Pending,
                    item: DownloadItem::Track {
                        track_id: 2,
                        source: DownloadApiSource::Tidal,
                        quality: TrackAudioQuality::FlacHighestRes,
                    },
                    file_path: "".into(),
                    created: "".into(),
                    updated: "".into(),
                    total_bytes: None,
                },
            ])
            .await;

        queue.process().await.unwrap().unwrap();
        queue.shutdown().await.unwrap();

        let responses = queue
            .state
            .read()
            .await
            .results
            .iter()
            .map(|result| result.as_ref().ok().cloned())
            .collect::<Vec<_>>();

        assert_eq!(
            responses,
            vec![
                Some(ProcessDownloadTaskResponse { task_id: 1 }),
                Some(ProcessDownloadTaskResponse { task_id: 2 })
            ]
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_can_process_duplicate_track_download_tasks() {
        let mut queue = new_queue();

        queue
            .add_tasks_to_queue(vec![
                DownloadTask {
                    id: 1,
                    state: DownloadTaskState::Pending,
                    item: DownloadItem::Track {
                        track_id: 1,
                        source: DownloadApiSource::Tidal,
                        quality: TrackAudioQuality::FlacHighestRes,
                    },
                    file_path: "".into(),
                    created: "".into(),
                    updated: "".into(),
                    total_bytes: None,
                },
                DownloadTask {
                    id: 1,
                    state: DownloadTaskState::Pending,
                    item: DownloadItem::Track {
                        track_id: 1,
                        source: DownloadApiSource::Tidal,
                        quality: TrackAudioQuality::FlacHighestRes,
                    },
                    file_path: "".into(),
                    created: "".into(),
                    updated: "".into(),
                    total_bytes: None,
                },
            ])
            .await;

        queue.process().await.unwrap().unwrap();
        queue.shutdown().await.unwrap();

        let responses = queue
            .state
            .read()
            .await
            .results
            .iter()
            .map(|result| result.as_ref().ok().cloned())
            .collect::<Vec<_>>();

        assert_eq!(
            responses,
            vec![Some(ProcessDownloadTaskResponse { task_id: 1 })]
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_can_process_another_track_download_task_after_processing_has_already_started() {
        let mut queue = new_queue();

        queue
            .add_task_to_queue(DownloadTask {
                id: 1,
                state: DownloadTaskState::Pending,
                item: DownloadItem::Track {
                    track_id: 1,
                    source: DownloadApiSource::Tidal,
                    quality: TrackAudioQuality::FlacHighestRes,
                },
                file_path: "".into(),
                created: "".into(),
                updated: "".into(),
                total_bytes: None,
            })
            .await;

        queue.process();

        queue
            .add_task_to_queue(DownloadTask {
                id: 2,
                state: DownloadTaskState::Pending,
                item: DownloadItem::Track {
                    track_id: 2,
                    source: DownloadApiSource::Tidal,
                    quality: TrackAudioQuality::FlacHighestRes,
                },
                file_path: "".into(),
                created: "".into(),
                updated: "".into(),
                total_bytes: None,
            })
            .await;

        tokio::time::sleep(std::time::Duration::from_millis(0)).await;

        queue.shutdown().await.unwrap();

        let responses = queue
            .state
            .read()
            .await
            .results
            .iter()
            .map(|result| result.as_ref().ok().cloned())
            .collect::<Vec<_>>();

        assert_eq!(
            responses,
            vec![
                Some(ProcessDownloadTaskResponse { task_id: 1 }),
                Some(ProcessDownloadTaskResponse { task_id: 2 })
            ]
        );
    }
}
