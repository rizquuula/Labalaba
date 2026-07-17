use crate::domain::log::entity::LogBroadcaster;
use crate::domain::process::service::ProcessSpawner;
use crate::domain::task::repository::TaskRepository;
use crate::domain::task::status::TaskRuntimeState;
use crate::infrastructure::log::file_writer::LogFileWriter;
use crate::infrastructure::process::resource_monitor::ResourceMonitor;
use crate::infrastructure::scheduler::cron_scheduler::CronScheduler;
use crate::infrastructure::updater::github_updater::GithubUpdater;
use labalaba_shared::api::{LogEntry, UpdateInfo};
use labalaba_shared::settings::AppSettings;
use labalaba_shared::task::TaskId;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::{mpsc, Notify, RwLock};

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
    /// Latest update found by a background check, stored so the frontend can pull
    /// it on mount even if it registered its listener after the event fired.
    pub pending_update: RwLock<Option<UpdateInfo>>,
    /// Cron scheduler — set once after AppState is Arc-wrapped so the scheduler
    /// can hold a Weak back-reference to AppState without a cycle.
    pub scheduler: OnceLock<Arc<CronScheduler>>,
    /// Shared secret token required on all API requests (Bearer auth).
    pub auth_token: String,
    /// Fired by the `POST /api/system/shutdown` handler to request a graceful
    /// shutdown of the daemon process (the bin's shutdown_signal awaits it).
    pub shutdown_notify: Arc<Notify>,
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
        auth_token: String,
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
            pending_update: RwLock::new(None),
            scheduler: OnceLock::new(),
            auth_token,
            shutdown_notify: Arc::new(Notify::new()),
        })
    }

    pub async fn save_settings(&self) -> anyhow::Result<()> {
        let settings = self.settings.read().await.clone();
        settings.save_to_file(&self.settings_path)?;
        Ok(())
    }

    /// Push the current log-related settings into the running [`LogFileWriter`]
    /// so a settings update takes effect without a restart. `max_file_size_mb`
    /// and `max_rotated_files` apply to subsequent writes/rotations; `log_dir`
    /// applies only to writers opened after this call (already-open files keep
    /// their path). Resolves `log_dir` the same way startup does (against
    /// `data_dir()`), so the writer keeps logging to the same place the daemon
    /// was launched against.
    pub async fn apply_log_settings(&self) {
        let (log_dir, max_file_size_mb, max_rotated_files) = {
            let s = self.settings.read().await;
            (
                s.log_dir.clone(),
                s.log_max_file_size_mb,
                s.log_max_rotated_files,
            )
        };

        let resolved = crate::resolve(&crate::data_dir(), &log_dir);

        self.log_writer
            .update_config(resolved, max_file_size_mb, max_rotated_files)
            .await;
        let _ = self.log_writer.init_dir().await;
    }

    /// Best-effort flush on app exit. Flushes/closes all log writers so buffered
    /// lines reach disk. Managed child processes are deliberately left running
    /// (the "survive app close" feature) — this does NOT kill anything.
    pub async fn shutdown(&self) {
        self.log_writer.close_all().await;
    }
}
