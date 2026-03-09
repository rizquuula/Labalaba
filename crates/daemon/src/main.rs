use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt};
use labalaba_daemon::{init_app_state, interface::http::router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("labalaba_daemon=info".parse()?),
        )
        .init();

    let state = init_app_state(None, None).await?;
    let port = state.settings.read().await.daemon_port;

    // Start resource monitor background refresh (5s interval)
    let resource_monitor = Arc::clone(&state.resource_monitor);
    tokio::spawn(async move {
        resource_monitor.start_background_refresh(5).await;
    });

    let app = router::build(Arc::clone(&state));
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Labalaba daemon listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
