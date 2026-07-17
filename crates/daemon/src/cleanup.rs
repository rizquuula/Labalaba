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

/// Free `port` when it is [`Blocked`](crate::infrastructure::net::PortState::Blocked)
/// by a task holding a dead daemon's inherited listener socket.
///
/// A daemon that exits without stopping its tasks used to leak its listening
/// socket into every task it had spawned (see
/// [`disable_handle_inheritance`](crate::infrastructure::net::disable_handle_inheritance)).
/// The socket outlives the daemon, so the port stays bound while refusing
/// connections: no later daemon can bind it and the GUI never reaches
/// `/api/health`. The install is bricked until the task is killed by hand.
///
/// The TCP table cannot name the culprit — it still attributes the socket to the
/// pid of the daemon that created it, which no longer exists — so the only
/// candidates are the task pids the previous daemon persisted. Kills them one at
/// a time, re-probing after each, and stops as soon as the port frees, so tasks
/// that happen not to hold it are left running.
///
/// No-op unless the port is genuinely `Blocked`: a healthy daemon (`Serving`) or
/// a free port is never touched, and neither is a foreign process's port — only
/// pids that still match their own task's executable are killed.
///
/// Daemons built after the inheritance fix cannot produce this state; this
/// recovers installs bricked by an older one. Returns the pids killed.
pub async fn reclaim_port_from_orphan_tasks(port: u16) -> Vec<u32> {
    use crate::domain::process::service::ProcessSpawner;
    use crate::infrastructure::net::{probe_port, PortState};
    use crate::infrastructure::persistence::yaml_repository::YamlTaskRepository;
    use crate::infrastructure::process::liveness::expected_process_stem;
    use crate::domain::task::repository::TaskRepository;

    let mut killed = Vec::new();
    if probe_port(port) != PortState::Blocked {
        return killed;
    }

    let (settings, _) = crate::load_settings().await;
    let repo = YamlTaskRepository::new(crate::resolve(&crate::data_dir(), &settings.config_path));
    let tasks = match repo.find_all().await {
        Ok(tasks) => tasks,
        Err(e) => {
            tracing::warn!("port {port} is blocked but tasks could not be read: {e}");
            return killed;
        }
    };

    let spawner = crate::infrastructure::process::spawner::OsProcessSpawner;
    for task in tasks {
        let stem = expected_process_stem(&task);
        for pid in &task.pids {
            // Identity-checked: never kill a recycled pid now owned by something else.
            if !crate::is_process_running(*pid, stem.as_deref()) {
                continue;
            }
            tracing::warn!(
                "port {port} is held by a leaked listener socket from a dead daemon; \
                 killing orphaned task process {pid} ({})",
                task.description
            );
            if let Err(e) = spawner.kill_tree(*pid).await {
                tracing::warn!("could not kill orphaned pid {pid}: {e}");
                continue;
            }
            killed.push(*pid);

            // Closing the last handle is what releases the address; give the OS
            // a moment before asking again.
            tokio::time::sleep(Duration::from_millis(200)).await;
            if probe_port(port) != PortState::Blocked {
                tracing::info!("port {port} released after killing {} orphan(s)", killed.len());
                return killed;
            }
        }
    }

    if !killed.is_empty() {
        tracing::warn!("port {port} is still blocked after killing {} orphan(s)", killed.len());
    }
    killed
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
/// abort the rest.
pub fn purge_user_data() -> anyhow::Result<()> {
    purge_user_data_in(&crate::data_dir(), crate::portable_marker_dir().as_deref())
}

/// [`purge_user_data`] against an explicit base, so it can be tested without
/// pointing the process's real data dir at a tempdir.
///
/// `marker_dir` is where the portable marker lives (`None` when portable mode is
/// unsupported). When a marker is found there it is removed **last**, after the
/// data itself: the marker is the flag that says "the data is over here", so
/// clearing it first would orphan whatever a crash left behind. Removing it at
/// all is deliberate — purge is a factory reset, and a marker surviving one
/// would silently bring a reinstall back up in portable mode pointing at an
/// empty directory, with no obvious way for the user to see why.
pub fn purge_user_data_in(
    base: &std::path::Path,
    marker_dir: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    // Load settings to resolve config_path and log_dir; fall back to defaults
    // when settings cannot be read (e.g. settings.yaml already deleted).
    let settings_yaml = base.join("settings.yaml");
    let (config_resolved, log_dir_resolved) = std::fs::read_to_string(&settings_yaml)
        .ok()
        .and_then(|c| serde_yaml::from_str::<labalaba_shared::api::AppSettings>(&c).ok())
        .map(|s| {
            (
                crate::resolve(base, &s.config_path),
                crate::resolve(base, &s.log_dir),
            )
        })
        .unwrap_or_else(|| (base.join("tasks.yaml"), base.join("logs")));

    for file in [&base.join("daemon.token"), &config_resolved, &settings_yaml] {
        if file.exists() {
            match std::fs::remove_file(file) {
                Ok(()) => tracing::info!("Purged {}", file.display()),
                Err(e) => tracing::warn!("Failed to remove {}: {e}", file.display()),
            }
        }
    }

    if log_dir_resolved.exists() {
        match std::fs::remove_dir_all(&log_dir_resolved) {
            Ok(()) => tracing::info!("Purged {}", log_dir_resolved.display()),
            Err(e) => tracing::warn!("Failed to remove {}: {e}", log_dir_resolved.display()),
        }
    }

    // Portable marker last — see the doc comment.
    if let Some(marker) = marker_dir.map(|d| d.join(crate::PORTABLE_MARKER)) {
        if marker.exists() {
            match std::fs::remove_file(&marker) {
                Ok(()) => tracing::info!("Purged {}", marker.display()),
                Err(e) => tracing::warn!("Failed to remove {}: {e}", marker.display()),
            }
            // Non-recursive on purpose: in portable mode `base` sits inside the
            // install dir, and it should vanish only if we really did empty it.
            // Anything the user put there keeps the directory — and keeps it out
            // of our hands.
            let _ = std::fs::remove_dir(base);
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purge_removes_data_then_marker_and_the_portable_dir() {
        let install = tempfile::tempdir().unwrap();
        let base = install.path().join("data");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("tasks.yaml"), "tasks: []").unwrap();
        std::fs::write(base.join("settings.yaml"), "daemon_port: 27015").unwrap();
        std::fs::write(base.join("daemon.token"), "tok").unwrap();
        std::fs::create_dir_all(base.join("logs")).unwrap();
        std::fs::write(base.join("logs").join("t.log"), "line").unwrap();
        let marker = install.path().join(crate::PORTABLE_MARKER);
        std::fs::write(&marker, "").unwrap();

        purge_user_data_in(&base, Some(install.path())).unwrap();

        assert!(!base.join("tasks.yaml").exists());
        assert!(!base.join("settings.yaml").exists());
        assert!(!base.join("daemon.token").exists());
        assert!(!base.join("logs").exists());
        assert!(!marker.exists(), "purge must not leave the app in portable mode");
        assert!(!base.exists(), "an emptied portable data dir should go too");
        // The install dir itself is not ours to delete.
        assert!(install.path().exists());
    }

    #[test]
    fn purge_keeps_the_portable_dir_when_the_user_left_files_in_it() {
        let install = tempfile::tempdir().unwrap();
        let base = install.path().join("data");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("tasks.yaml"), "tasks: []").unwrap();
        std::fs::write(base.join("my-notes.txt"), "keep me").unwrap();
        std::fs::write(install.path().join(crate::PORTABLE_MARKER), "").unwrap();

        purge_user_data_in(&base, Some(install.path())).unwrap();

        assert!(!base.join("tasks.yaml").exists());
        assert!(base.join("my-notes.txt").exists(), "never delete what we did not write");
        assert!(base.exists());
    }

    #[test]
    fn purge_without_portable_behaves_exactly_as_before() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("tasks.yaml"), "tasks: []").unwrap();
        std::fs::write(base.path().join("daemon.token"), "tok").unwrap();

        purge_user_data_in(base.path(), None).unwrap();

        assert!(!base.path().join("tasks.yaml").exists());
        assert!(!base.path().join("daemon.token").exists());
        // No marker involved, so the base survives.
        assert!(base.path().exists());
    }

    #[test]
    fn purge_follows_an_absolute_config_path_out_of_the_data_dir() {
        let base = tempfile::tempdir().unwrap();
        let pinned = tempfile::tempdir().unwrap();
        let pinned_tasks = pinned.path().join("my-tasks.yaml");
        std::fs::write(&pinned_tasks, "tasks: []").unwrap();
        std::fs::write(
            base.path().join("settings.yaml"),
            format!(
                "config_path: {}",
                serde_yaml::to_string(&pinned_tasks.to_string_lossy()).unwrap().trim()
            ),
        )
        .unwrap();

        purge_user_data_in(base.path(), None).unwrap();

        assert!(!pinned_tasks.exists(), "purge means purge, wherever the user pinned it");
    }
}
