pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use labalaba_shared::api::{AppSettings, LogEntry, UpdateInfo};
use labalaba_shared::task::TaskId;
use infrastructure::{
    persistence::yaml_repository::YamlTaskRepository,
    process::spawner::OsProcessSpawner,
    updater::github_updater::GithubUpdater,
    state::AppState,
    log::file_writer::LogFileWriter,
};
use application::task::start_task::StartTask;
use application::update::check_update::CheckUpdate;

/// Base directory for all data files (tasks.yaml, settings.yaml, logs/).
/// Reads `LABALABA_DATA_DIR` env var; falls back to the current working directory.
pub fn data_dir() -> PathBuf {
    std::env::var("LABALABA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolve a path that may be relative (e.g. "./tasks.yaml") against a base dir.
fn resolve(base: &Path, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(p.trim_start_matches("./"))
    }
}

/// Check if a process with the given PID is still running.
#[cfg(target_os = "windows")]
pub fn is_process_running(pid: u32) -> bool {
    use std::process::Command;
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Load settings from `{data_dir}/settings.yaml`, falling back to defaults.
pub async fn load_settings() -> (AppSettings, String) {
    let base = data_dir();
    let settings_path = base.join("settings.yaml");
    let settings_path_str = settings_path.to_string_lossy().to_string();
    
    if settings_path.exists() {
        if let Ok(contents) = tokio::fs::read_to_string(&settings_path).await {
            if let Ok(s) = serde_yaml::from_str::<AppSettings>(&contents) {
                return (s, settings_path_str);
            }
        }
    }
    (AppSettings::default(), settings_path_str)
}

/// Initialize the full daemon AppState, recover task states, and spawn the restart loop.
///
/// `log_event_callback` — when `Some`, called for every log line produced by managed tasks.
/// Pass `None` for the standalone daemon (which streams via WebSocket instead).
/// `update_event_callback` — when `Some`, called when an update is available.
pub async fn init_app_state(
    log_event_callback: Option<Arc<dyn Fn(LogEntry) + Send + Sync + 'static>>,
    update_event_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
) -> anyhow::Result<Arc<AppState>> {
    let (settings, settings_path) = load_settings().await;

    let (restart_tx, restart_rx) = mpsc::channel::<TaskId>(64);

    let base = data_dir();
    let log_writer = LogFileWriter::new(
        resolve(&base, &settings.log_dir),
        settings.log_max_file_size_mb,
        settings.log_max_rotated_files,
    );
    log_writer.init_dir().await?;

    let config_path = resolve(&base, &settings.config_path);
    let repo = Arc::new(YamlTaskRepository::new(config_path));
    let spawner = Arc::new(OsProcessSpawner);
    let updater = Arc::new(GithubUpdater::new());
    let state = AppState::new(repo, spawner, updater, settings.clone(), settings_path.clone(), restart_tx, log_writer, log_event_callback, update_event_callback);

    // Save settings to file if it doesn't exist
    if !std::path::Path::new(&settings_path).exists() {
        if let Err(e) = state.save_settings().await {
            tracing::warn!("Failed to save initial settings: {}", e);
        }
    }

    recover_task_states(Arc::clone(&state)).await;
    tokio::spawn(restart_loop(restart_rx, Arc::clone(&state)));

    // Auto-check for updates if enabled
    if settings.auto_check_updates {
        let callback = state.update_event_callback.clone();
        spawn_update_checker(Arc::clone(&state), callback);
    }

    Ok(state)
}

/// Receives restart requests from crashed tasks and re-executes them.
pub async fn restart_loop(mut rx: mpsc::Receiver<TaskId>, state: Arc<AppState>) {
    while let Some(id) = rx.recv().await {
        tracing::info!("Auto-restarting task {}", id);
        let uc = StartTask { state: Arc::clone(&state) };
        if let Err(e) = uc.execute(id.clone()).await {
            tracing::warn!("Auto-restart of {} failed: {}", id, e);
        }
    }
}

/// Recover runtime states for tasks with persisted PIDs on daemon startup.
pub async fn recover_task_states(state: Arc<AppState>) {
    match state.task_repo.find_all().await {
        Ok(tasks) => {
            for task in tasks {
                if !task.pids.is_empty() {
                    let alive_pids: Vec<u32> = task.pids.iter()
                        .filter(|&&pid| is_process_running(pid))
                        .copied()
                        .collect();

                    let mut states = state.runtime_states.write().await;
                    let rt = states.entry(task.id.clone()).or_default();

                    if !alive_pids.is_empty() {
                        rt.mark_running(*alive_pids.first().unwrap());
                        tracing::info!(
                            "Recovered task {} as Running with {} alive PID(s)",
                            task.id,
                            alive_pids.len()
                        );
                    } else {
                        rt.mark_crashed(None);
                        tracing::info!(
                            "Recovered task {} as Crashed (no alive PIDs from: {:?})",
                            task.id,
                            task.pids
                        );
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load tasks for recovery: {}", e);
        }
    }
}

/// Spawn a background task to check for updates and emit event if available.
fn spawn_update_checker(
    state: Arc<AppState>,
    update_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
) {
    tokio::spawn(async move {
        let uc = CheckUpdate { state };
        match uc.execute().await {
            Ok(info) => {
                if info.available {
                    tracing::info!(
                        "Update available: {} (current: {})",
                        info.latest_version.as_ref().unwrap_or(&"unknown".to_string()),
                        info.current_version
                    );
                    if let Some(cb) = &update_callback {
                        cb(info);
                    }
                } else {
                    tracing::debug!("No update available (current: {})", info.current_version);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to check for updates: {}", e);
            }
        }
    });
}
