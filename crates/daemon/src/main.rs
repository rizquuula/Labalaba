use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt};
use labalaba_daemon::{init_app_state, interface::http::router};
use labalaba_daemon::infrastructure::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("labalaba_daemon=info".parse()?),
        )
        .init();

    // ── Subcommand: cleanup ───────────────────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("cleanup") {
        let mut purge = false;
        let mut yes = false;
        for arg in args.iter().skip(2) {
            match arg.as_str() {
                "--purge" => purge = true,
                "--yes" => yes = true,
                other => {
                    eprintln!("Unknown flag: {other}");
                    std::process::exit(1);
                }
            }
        }

        if purge && !yes {
            use std::io::IsTerminal;
            if std::io::stdin().is_terminal() {
                let data_dir = labalaba_daemon::data_dir();
                eprintln!(
                    "WARNING: This will permanently delete data files in {}.",
                    data_dir.display()
                );
                eprint!("Type 'y' or 'yes' to confirm: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let trimmed = input.trim().to_lowercase();
                if trimmed != "y" && trimmed != "yes" {
                    eprintln!("Aborted.");
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "Error: refusing to purge user data without --yes (stdin is not a terminal)"
                );
                std::process::exit(1);
            }
        }

        labalaba_daemon::cleanup(purge).await?;
        println!("Cleanup complete.");
        return Ok(());
    }

    // ── Normal server startup ─────────────────────────────────────────────────
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

    // Before any task is spawned: tasks inherit every inheritable handle we own,
    // and an inherited listener keeps this port bound after we exit — bricking
    // every later daemon's bind. See infrastructure::net.
    #[cfg(windows)]
    labalaba_daemon::infrastructure::net::disable_handle_inheritance(&listener)?;

    tracing::info!("Labalaba daemon listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(Arc::clone(&state)))
        .await?;

    // Reached only after graceful shutdown has drained in-flight requests and
    // log writers were flushed in shutdown_signal — the process is now exiting.
    tracing::info!("Daemon shut down cleanly");
    Ok(())
}

async fn shutdown_signal(state: Arc<AppState>) {
    let api_shutdown = Arc::clone(&state.shutdown_notify);
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received Ctrl-C, shutting down");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, shutting down");
            }
            _ = api_shutdown.notified() => {
                tracing::info!("Received API shutdown request, shutting down");
            }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received Ctrl-C, shutting down");
            }
            _ = api_shutdown.notified() => {
                tracing::info!("Received API shutdown request, shutting down");
            }
        }
    }
    state.shutdown().await;
}
