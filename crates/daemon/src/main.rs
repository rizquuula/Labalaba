mod domain;
mod application;
mod infrastructure;
mod interface;

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::{EnvFilter, fmt};
use labalaba_shared::api::AppSettings;
use labalaba_shared::task::TaskId;
use infrastructure::{
    persistence::yaml_repository::YamlTaskRepository,
    process::spawner::OsProcessSpawner,
    updater::github_updater::GithubUpdater,
    state::AppState,
};
use interface::http::router;
use application::task::start_task::StartTask;

/// Check if a process with the given PID is still running
#[cfg(target_os = "windows")]
fn is_process_running(pid: u32) -> bool {
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
fn is_process_running(pid: u32) -> bool {
    // Check if process exists by sending signal 0
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("labalaba_daemon=info".parse()?),
        )
        .init();

    let settings = load_settings().await;
    let port = settings.daemon_port;
    let config_path = settings.config_path.clone();

    // Restart channel: background tasks send TaskId here to request a restart
    let (restart_tx, restart_rx) = mpsc::channel::<TaskId>(64);

    let repo = Arc::new(YamlTaskRepository::new(&config_path));
    let spawner = Arc::new(OsProcessSpawner);
    let updater = Arc::new(GithubUpdater::new());
    let state = AppState::new(repo, spawner, updater, settings, restart_tx);

    // Recover runtime states from persisted PIDs on startup
    recover_task_states(Arc::clone(&state)).await;

    // Background loop: drain restart requests and call StartTask
    tokio::spawn(restart_loop(restart_rx, Arc::clone(&state)));

    let app = router::build(Arc::clone(&state));
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Labalaba daemon listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Receives restart requests from crashed tasks and re-executes them.
async fn restart_loop(mut rx: mpsc::Receiver<TaskId>, state: Arc<AppState>) {
    while let Some(id) = rx.recv().await {
        tracing::info!("Auto-restarting task {}", id);
        let uc = StartTask { state: Arc::clone(&state) };
        if let Err(e) = uc.execute(id.clone()).await {
            tracing::warn!("Auto-restart of {} failed: {}", id, e);
        }
    }
}

/// Recover runtime states for tasks with persisted PIDs on daemon startup.
/// Checks if PIDs are still running and updates the runtime state accordingly.
async fn recover_task_states(state: Arc<AppState>) {
    match state.task_repo.find_all().await {
        Ok(tasks) => {
            for task in tasks {
                if !task.pids.is_empty() {
                    // Check which PIDs are still alive
                    let alive_pids: Vec<u32> = task.pids.iter()
                        .filter(|&&pid| is_process_running(pid))
                        .copied()
                        .collect();

                    let mut states = state.runtime_states.write().await;
                    let rt = states.entry(task.id.clone()).or_default();

                    if !alive_pids.is_empty() {
                        // At least one PID is still running - mark as Running
                        rt.mark_running(*alive_pids.first().unwrap());
                        tracing::info!(
                            "Recovered task {} as Running with {} alive PID(s)",
                            task.id,
                            alive_pids.len()
                        );
                    } else {
                        // No PIDs alive - mark as Crashed
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

async fn load_settings() -> AppSettings {
    let path = std::path::Path::new("./settings.yaml");
    if path.exists() {
        if let Ok(contents) = tokio::fs::read_to_string(path).await {
            if let Ok(s) = serde_yaml::from_str::<AppSettings>(&contents) {
                return s;
            }
        }
    }
    AppSettings::default()
}
