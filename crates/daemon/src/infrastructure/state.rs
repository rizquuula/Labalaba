use crate::domain::log::entity::LogBroadcaster;
use crate::domain::process::service::ProcessSpawner;
use crate::domain::task::repository::TaskRepository;
use crate::domain::task::status::TaskRuntimeState;
use crate::infrastructure::log::file_writer::LogFileWriter;
use crate::infrastructure::process::resource_monitor::ResourceMonitor;
use crate::infrastructure::updater::github_updater::GithubUpdater;
use labalaba_shared::api::{AppSettings, LogEntry, UpdateInfo};
use labalaba_shared::task::TaskId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Shared application state passed to all HTTP handlers and use cases.
/// Arc-wrapped for safe concurrent access across async tasks.
pub struct AppState {
    pub task_repo: Arc<dyn TaskRepository>,
    pub spawner: Arc<dyn ProcessSpawner>,
    pub updater: Arc<GithubUpdater>,
    pub resource_monitor: Arc<ResourceMonitor>,
    pub settings_path: String,
    pub settings: RwLock<AppSettings>,
    /// In-memory runtime status per task (not persisted)
    pub runtime_states: RwLock<HashMap<TaskId, TaskRuntimeState>>,
    /// Log broadcast channels per task
    pub log_channels: RwLock<HashMap<TaskId, LogBroadcaster>>,
    /// Channel for requesting a task restart from background tasks.
    /// Breaks the recursive Send issue in auto-restart logic.
    pub restart_tx: mpsc::Sender<TaskId>,
    /// Log file writer for persisting logs to disk
    pub log_writer: LogFileWriter,
    /// Optional callback invoked on every log entry (used for Tauri event emission).
    /// Keeps the daemon crate Tauri-agnostic.
    pub log_event_callback: Option<Arc<dyn Fn(LogEntry) + Send + Sync + 'static>>,
    /// Optional callback invoked when an update is available (used for Tauri event emission).
    pub update_event_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
}

impl AppState {
    pub fn new(
        task_repo: Arc<dyn TaskRepository>,
        spawner: Arc<dyn ProcessSpawner>,
        updater: Arc<GithubUpdater>,
        settings: AppSettings,
        settings_path: String,
        restart_tx: mpsc::Sender<TaskId>,
        log_writer: LogFileWriter,
        log_event_callback: Option<Arc<dyn Fn(LogEntry) + Send + Sync + 'static>>,
        update_event_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
    ) -> Arc<Self> {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        
        Arc::new(Self {
            task_repo,
            spawner,
            updater,
            resource_monitor,
            settings_path,
            settings: RwLock::new(settings),
            runtime_states: RwLock::new(HashMap::new()),
            log_channels: RwLock::new(HashMap::new()),
            restart_tx,
            log_writer,
            log_event_callback,
            update_event_callback,
        })
    }

    pub async fn save_settings(&self) -> anyhow::Result<()> {
        let settings = self.settings.read().await;
        settings.save_to_file(&self.settings_path)?;
        Ok(())
    }
}
