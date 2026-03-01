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
