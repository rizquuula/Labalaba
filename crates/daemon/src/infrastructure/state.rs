use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use labalaba_shared::task::TaskId;
use labalaba_shared::api::AppSettings;
use crate::domain::task::repository::TaskRepository;
use crate::domain::task::status::TaskRuntimeState;
use crate::domain::process::service::ProcessSpawner;
use crate::domain::log::entity::LogBroadcaster;
use crate::infrastructure::updater::github_updater::GithubUpdater;

/// Shared application state passed to all HTTP handlers and use cases.
/// Arc-wrapped for safe concurrent access across async tasks.
pub struct AppState {
    pub task_repo: Arc<dyn TaskRepository>,
    pub spawner: Arc<dyn ProcessSpawner>,
    pub updater: Arc<GithubUpdater>,
    pub settings: RwLock<AppSettings>,
    /// In-memory runtime status per task (not persisted)
    pub runtime_states: RwLock<HashMap<TaskId, TaskRuntimeState>>,
    /// Log broadcast channels per task
    pub log_channels: RwLock<HashMap<TaskId, LogBroadcaster>>,
    /// Channel for requesting a task restart from background tasks.
    /// Breaks the recursive Send issue in auto-restart logic.
    pub restart_tx: mpsc::Sender<TaskId>,
}

impl AppState {
    pub fn new(
        task_repo: Arc<dyn TaskRepository>,
        spawner: Arc<dyn ProcessSpawner>,
        updater: Arc<GithubUpdater>,
        settings: AppSettings,
        restart_tx: mpsc::Sender<TaskId>,
    ) -> Arc<Self> {
        Arc::new(Self {
            task_repo,
            spawner,
            updater,
            settings: RwLock::new(settings),
            runtime_states: RwLock::new(HashMap::new()),
            log_channels: RwLock::new(HashMap::new()),
            restart_tx,
        })
    }
}
