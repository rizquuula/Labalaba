use std::net::SocketAddr;
use std::time::Duration;

/// Send a shutdown request to a running daemon and wait for the port to close.
/// Returns `Ok(true)` if the daemon was stopped, `Ok(false)` if it was already
/// down (no token file, or connection refused before the request was sent).
pub async fn stop_running_daemon() -> anyhow::Result<bool> {
    let (settings, _) = crate::load_settings().await;
    let port = settings.daemon_port;

    let token_path = crate::data_dir().join("daemon.token");
    let token = match tokio::fs::read_to_string(&token_path).await {
        Ok(t) => {
            let t = t.trim().to_string();
            if t.is_empty() {
                tracing::info!("daemon.token is empty — daemon is not running");
                return Ok(false);
            }
            t
        }
        Err(_) => {
            tracing::info!("daemon.token not found — daemon is not running");
            return Ok(false);
        }
    };

    let url = format!("http://127.0.0.1:{port}/api/system/shutdown");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    match client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(resp) => {
            tracing::info!("Shutdown request returned status {}", resp.status());
        }
        Err(e) if is_connection_refused(&e) => {
            tracing::info!("Daemon is not reachable (connection refused) — already down");
            return Ok(false);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("shutdown request failed: {e}"));
        }
    }

    // Poll until the port stops accepting connections (max 5 s).
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse()?;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_err() {
            tracing::info!("Daemon port {port} is no longer listening — stopped");
            return Ok(true);
        }
    }

    tracing::warn!("Daemon did not stop within 5 s after shutdown request");
    Ok(true)
}

/// Probe a possibly-running daemon's public `/api/health` endpoint and return
/// the version it reports, or `None` if nothing answered within a short window.
/// `None` covers "unreachable", "a foreign process holds the port", and "a
/// daemon that accepts the connection but is mid-teardown and never replies".
/// Used by the GUI to decide whether a daemon already on the port is a healthy,
/// current-version daemon worth reusing — versus one that must be reclaimed.
pub async fn daemon_health(port: u16) -> Option<String> {
    let url = format!("http://127.0.0.1:{port}/api/health");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("data")?
        .get("version")?
        .as_str()
        .map(|s| s.to_string())
}

fn is_connection_refused(e: &reqwest::Error) -> bool {
    use std::error::Error as StdError;
    // Walk the error chain looking for a ConnectionRefused io error.
    let mut cause: Option<&dyn StdError> = Some(e);
    while let Some(c) = cause {
        if let Some(io) = c.downcast_ref::<std::io::Error>() {
            if io.kind() == std::io::ErrorKind::ConnectionRefused {
                return true;
            }
        }
        // Also check string representation for cases where downcast doesn't
        // reach the inner io::Error directly.
        let msg = c.to_string();
        if msg.contains("Connection refused") || msg.contains("connection refused") {
            return true;
        }
        cause = c.source();
    }
    false
}

/// Best-effort delete of all user-data artifacts produced by the daemon.
/// Each deletion is attempted independently; failures are logged but do not
/// abort the rest.  The data directory itself is NOT removed.
pub fn purge_user_data() -> anyhow::Result<()> {
    let base = crate::data_dir();

    // Load settings to resolve config_path and log_dir; fall back to defaults
    // when settings cannot be read (e.g. settings.yaml already deleted).
    let (settings_path_resolved, log_dir_resolved) = {
        let settings_file = base.join("settings.yaml");
        if settings_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&settings_file) {
                if let Ok(s) = serde_yaml::from_str::<labalaba_shared::api::AppSettings>(&contents) {
                    let cfg = resolve_path(&base, &s.config_path);
                    let logs = resolve_path(&base, &s.log_dir);
                    (cfg, logs)
                } else {
                    (base.join("tasks.yaml"), base.join("logs"))
                }
            } else {
                (base.join("tasks.yaml"), base.join("logs"))
            }
        } else {
            (base.join("tasks.yaml"), base.join("logs"))
        }
    };

    // daemon.token
    let token_path = base.join("daemon.token");
    if token_path.exists() {
        match std::fs::remove_file(&token_path) {
            Ok(()) => tracing::info!("Purged {}", token_path.display()),
            Err(e) => tracing::warn!("Failed to remove {}: {e}", token_path.display()),
        }
    }

    // tasks.yaml (config_path)
    if settings_path_resolved.exists() {
        match std::fs::remove_file(&settings_path_resolved) {
            Ok(()) => tracing::info!("Purged {}", settings_path_resolved.display()),
            Err(e) => tracing::warn!("Failed to remove {}: {e}", settings_path_resolved.display()),
        }
    }

    // settings.yaml
    let settings_yaml = base.join("settings.yaml");
    if settings_yaml.exists() {
        match std::fs::remove_file(&settings_yaml) {
            Ok(()) => tracing::info!("Purged {}", settings_yaml.display()),
            Err(e) => tracing::warn!("Failed to remove {}: {e}", settings_yaml.display()),
        }
    }

    // logs/ directory (recursive)
    if log_dir_resolved.exists() {
        match std::fs::remove_dir_all(&log_dir_resolved) {
            Ok(()) => tracing::info!("Purged {}", log_dir_resolved.display()),
            Err(e) => tracing::warn!("Failed to remove {}: {e}", log_dir_resolved.display()),
        }
    }

    Ok(())
}

fn resolve_path(base: &std::path::Path, p: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(p.trim_start_matches("./"))
    }
}

/// Stop the running daemon, remove its autostart entry, and optionally purge
/// all user data.
pub async fn cleanup(purge: bool) -> anyhow::Result<()> {
    tracing::info!("Stopping daemon…");
    match stop_running_daemon().await {
        Ok(true) => tracing::info!("Daemon stopped"),
        Ok(false) => tracing::info!("Daemon was not running"),
        Err(e) => tracing::warn!("Could not stop daemon: {e}"),
    }

    tracing::info!("Removing autostart…");
    match crate::infrastructure::autostart::uninstall() {
        Ok(()) => tracing::info!("Autostart entry removed"),
        Err(e) => tracing::warn!("Could not remove autostart entry: {e}"),
    }

    if purge {
        tracing::info!("Purging user data…");
        purge_user_data()?;
    }

    tracing::info!("Cleanup complete.");
    Ok(())
}
