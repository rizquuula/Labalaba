use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use labalaba_daemon::infrastructure::net::{probe_port, PortState};
use tauri::Manager;

#[derive(Clone, serde::Serialize)]
pub struct DaemonConnection {
    pub base_url: String,
    pub ws_url: String,
    pub token: String,
}

pub struct DaemonHandle {
    pub connection: Mutex<Option<DaemonConnection>>,
    pub child: Mutex<Option<std::process::Child>>,
    /// Why the last connection attempt failed, if it did. Lets
    /// `get_daemon_connection` report the real reason instead of timing out on a
    /// connect that already gave up.
    pub error: Mutex<Option<String>>,
}

impl Default for DaemonHandle {
    fn default() -> Self {
        Self {
            connection: Mutex::new(None),
            child: Mutex::new(None),
            error: Mutex::new(None),
        }
    }
}

/// Connect to (or start) the daemon and publish the result into the managed
/// [`DaemonHandle`].
///
/// Runs off the main thread: `start_or_connect_daemon` can take ~20s in the bad
/// cases (reclaiming a port, waiting for a spawn), and Tauri's `setup` hook runs
/// before the event loop starts — blocking there leaves an unpainted window and
/// wedges `tauri_plugin_single_instance`, whose Windows `SendMessageW` has no
/// timeout and hangs every later launch until we pump messages again.
pub fn connect_in_background(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let result = start_or_connect_daemon();
        let state = app.state::<DaemonHandle>();
        match result {
            Ok((conn, child)) => {
                *state.connection.lock().unwrap() = Some(conn);
                *state.child.lock().unwrap() = child;
                *state.error.lock().unwrap() = None;
            }
            Err(e) => {
                // Release builds set windows_subsystem = "windows", so this
                // reaches no console — the error is surfaced through the handle.
                eprintln!("Failed to connect to daemon: {e}");
                *state.error.lock().unwrap() = Some(e.to_string());
            }
        }
    });
}

#[derive(Clone, serde::Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub port: u16,
    pub autostart: bool,
}

#[tauri::command]
pub fn daemon_status() -> DaemonStatus {
    let (settings, _) = tauri::async_runtime::block_on(labalaba_daemon::load_settings());
    let port = settings.daemon_port;
    let running = is_listening(port);
    let autostart = crate::commands::service::is_autostart_installed();
    DaemonStatus { running, port, autostart }
}

#[tauri::command]
pub fn start_daemon(state: tauri::State<'_, DaemonHandle>) -> Result<(), String> {
    let (conn, child) = start_or_connect_daemon().map_err(|e| {
        let msg = e.to_string();
        *state.error.lock().unwrap() = Some(msg.clone());
        msg
    })?;
    *state.connection.lock().unwrap() = Some(conn);
    *state.error.lock().unwrap() = None;
    let mut guard = state.child.lock().unwrap();
    if let Some(mut old) = guard.take() {
        // Reap it if it has already exited. Not killed: either `reclaim_port`
        // just stopped it, or `start_or_connect_daemon` reused it and it is the
        // daemon we are now connected to.
        let _ = old.try_wait();
    }
    *guard = child;
    Ok(())
}

#[tauri::command]
pub fn cleanup_daemon(state: tauri::State<'_, DaemonHandle>, purge: bool) -> Result<(), String> {
    let res = tauri::async_runtime::block_on(labalaba_daemon::cleanup(purge)).map_err(|e| e.to_string());
    // The daemon has been told to stop; drop our stale child handle so the Exit
    // handler doesn't later try to kill a dead pid.
    if let Some(mut old) = state.child.lock().unwrap().take() { let _ = old.try_wait(); }
    res
}

/// Records that autostart was enabled when an update install began. Written by
/// `prepare_for_update`, consumed by `restore_autostart_after_update` on the
/// next launch.
fn autostart_marker_path() -> PathBuf {
    labalaba_daemon::data_dir().join("autostart.pending")
}

/// Stop the daemon so an update installer can overwrite its binary.
///
/// The daemon is a separate process installed next to the app (see
/// `resolve_daemon_bin`) and it deliberately outlives the window. At install
/// time it therefore still holds an open handle on the very file the installer
/// wants to replace — on Windows that lock makes the install fail outright.
///
/// Call this only *after* the update has been downloaded and its signature
/// verified. Stopping the daemon kills the user's running tasks, so doing it
/// for an update that then fails to download would be a pointless outage.
#[tauri::command]
pub fn prepare_for_update(state: tauri::State<'_, DaemonHandle>) -> Result<(), String> {
    // The Windows installer runs `labalaba-daemon.exe cleanup` from the NSIS
    // pre-uninstall hook, and cleanup removes the autostart entry as well as
    // stopping the daemon. Record the current state so the next launch can put
    // it back rather than silently losing it on every update.
    let marker = autostart_marker_path();
    if crate::commands::service::is_autostart_installed() {
        if let Err(e) = std::fs::write(&marker, "1") {
            eprintln!(
                "could not record autostart state at {}: {e}",
                marker.display()
            );
        }
    } else {
        let _ = std::fs::remove_file(&marker);
    }

    let (settings, _) = tauri::async_runtime::block_on(labalaba_daemon::load_settings());
    let port = settings.daemon_port;

    reclaim_port(port);

    // We told the daemon to die; drop our child handle so the Exit handler
    // doesn't later try to kill a pid that's already gone.
    if let Some(mut old) = state.child.lock().unwrap().take() {
        let _ = old.try_wait();
    }

    // Report rather than let the installer fail with an opaque "file in use".
    if is_listening(port) {
        return Err(format!(
            "the daemon is still listening on port {port}; the installer would fail to \
             replace labalaba-daemon — stop it manually and try again"
        ));
    }

    Ok(())
}

/// Re-install the autostart entry if an update installer removed it. No-op
/// unless `prepare_for_update` left a marker behind, so a user who deliberately
/// turned autostart off never has it forced back on.
pub(crate) fn restore_autostart_after_update() {
    let marker = autostart_marker_path();
    if !marker.exists() {
        return;
    }
    let _ = std::fs::remove_file(&marker);

    if labalaba_daemon::infrastructure::autostart::is_installed() {
        return;
    }

    let Some(bin) = resolve_daemon_bin() else {
        eprintln!("cannot restore autostart after update: daemon binary not found");
        return;
    };
    match labalaba_daemon::infrastructure::autostart::install(&bin) {
        Ok(()) => eprintln!("restored the autostart entry removed during the update"),
        Err(e) => eprintln!("could not restore autostart entry after update: {e}"),
    }
}

/// How long the frontend's first call waits for the background connect. Must
/// exceed the worst case inside `start_or_connect_daemon` (~20s: health probe,
/// port reclaim, spawn readiness, token read).
const CONNECT_WAIT: Duration = Duration::from_secs(30);
const CONNECT_POLL: Duration = Duration::from_millis(100);

/// Await the connection established by [`connect_in_background`].
///
/// `async` on purpose: Tauri runs sync commands on the main thread, so blocking
/// here would freeze the very window this wait exists to keep responsive.
#[tauri::command]
pub async fn get_daemon_connection(
    state: tauri::State<'_, DaemonHandle>,
) -> Result<DaemonConnection, String> {
    let deadline = std::time::Instant::now() + CONNECT_WAIT;
    loop {
        // Scoped so no std MutexGuard is held across an await.
        {
            if let Some(conn) = state.connection.lock().unwrap().clone() {
                return Ok(conn);
            }
            if let Some(err) = state.error.lock().unwrap().clone() {
                return Err(err);
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err("timed out waiting for the daemon to start".into());
        }
        tokio::time::sleep(CONNECT_POLL).await;
    }
}

/// Whether something is answering on `port`.
///
/// Only asks "is a server there?". It cannot tell a free port from one that is
/// bound but refusing connections — use
/// [`probe_port`](labalaba_daemon::infrastructure::net::probe_port) where that
/// distinction matters, i.e. before deciding to spawn a daemon.
fn is_listening(port: u16) -> bool {
    labalaba_daemon::infrastructure::net::is_connectable(port)
}

/// Best-effort: stop whatever Labalaba daemon is holding `port` and wait for it
/// to free up so a fresh daemon can bind. Tries a graceful shutdown first
/// (authenticated via the persisted token), then force-kills any lingering
/// daemon process by image name. Never errors — if a *foreign* process keeps
/// the port, the subsequent spawn fails loudly instead of hanging the UI.
fn reclaim_port(port: u16) {
    let _ = tauri::async_runtime::block_on(labalaba_daemon::stop_running_daemon());

    if is_listening(port) {
        force_kill_daemon();
    }

    // Wait up to ~5s for the port to actually free.
    for _ in 0..50 {
        if !is_listening(port) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Force-terminate any running Labalaba daemon by image name. Targets only the
/// daemon binary, never an unrelated process that happens to hold the port.
fn force_kill_daemon() {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", "labalaba-daemon.exe", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-x", "labalaba-daemon"])
            .output();
    }
}

fn daemon_name() -> &'static str {
    if cfg!(windows) {
        "labalaba-daemon.exe"
    } else {
        "labalaba-daemon"
    }
}

pub(crate) fn resolve_daemon_bin() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("LABALABA_DAEMON_BIN") {
        if !val.is_empty() {
            let p = PathBuf::from(val);
            if p.exists() {
                return Some(p);
            }
        }
    }

    if let Some(p) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(daemon_name())))
    {
        if p.exists() {
            return Some(p);
        }
    }

    None
}

pub fn start_or_connect_daemon() -> anyhow::Result<(DaemonConnection, Option<std::process::Child>)> {
    let base = labalaba_daemon::data_dir();
    let (settings, _) = tauri::async_runtime::block_on(labalaba_daemon::load_settings());
    let port = settings.daemon_port;

    let base_url = format!("http://127.0.0.1:{port}");
    let ws_url = format!("ws://127.0.0.1:{port}");

    // Only reuse a daemon already on the port if it answers /api/health AND
    // reports the version we bundle. Otherwise we'd risk binding the new UI to a
    // stale, unresponsive, or older daemon left over from a previous version —
    // e.g. one still running because the window was only closed to the tray, or
    // one being torn down mid-upgrade. Both of those used to hang the UI.
    let bundled_version = env!("CARGO_PKG_VERSION");
    let mut reuse = false;
    match probe_port(port) {
        // Nothing holds it — go straight to spawning.
        PortState::Free => {}

        PortState::Serving => {
            match tauri::async_runtime::block_on(labalaba_daemon::daemon_health(port)) {
                Some(ref v) if v == bundled_version => reuse = true,
                Some(v) => eprintln!(
                    "daemon on port {port} reports version {v}, expected {bundled_version}; restarting it"
                ),
                None => eprintln!(
                    "a process holds port {port} but /api/health did not respond; reclaiming it"
                ),
            }
            if !reuse {
                reclaim_port(port);
            }
        }

        // Bound but refusing connections: a task is holding a dead daemon's
        // inherited listener socket. Nothing can bind until that task dies, so
        // spawning here would fail with an unbindable port and surface as a
        // silent "daemon did not become reachable".
        PortState::Blocked => {
            let killed =
                tauri::async_runtime::block_on(labalaba_daemon::reclaim_port_from_orphan_tasks(port));
            if probe_port(port) == PortState::Blocked {
                let detail = if killed.is_empty() {
                    "no Labalaba task owns it".to_string()
                } else {
                    format!("it stayed blocked after killing {} orphaned task(s)", killed.len())
                };
                anyhow::bail!(
                    "port {port} is bound but refusing connections and {detail} — another \
                     application may be using it. Close it, or change daemon_port in settings."
                );
            }
            if !killed.is_empty() {
                eprintln!(
                    "freed port {port} by killing {} orphaned task process(es) left by a \
                     previous daemon",
                    killed.len()
                );
            }
        }
    }

    let child = if reuse {
        None
    } else {
        let bin = resolve_daemon_bin().ok_or_else(|| {
            anyhow::anyhow!(
                "daemon binary not found; set LABALABA_DAEMON_BIN or place labalaba-daemon next to the app"
            )
        })?;

        let mut cmd = std::process::Command::new(bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd.spawn()?;

        let mut ready = false;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            if is_listening(port) {
                ready = true;
                break;
            }
        }
        if !ready {
            // `Child::drop` does not kill the process. Leaving it would let a
            // daemon we've given up on keep initialising and bind the port
            // behind our back, so every failed launch would strand one more.
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("daemon did not become reachable on port {port} within 5 seconds");
        }

        Some(child)
    };

    let token_path = base.join("daemon.token");
    let token = read_token_with_retry(&token_path).map_err(|_| {
        if child.is_none() {
            // We didn't spawn it: something else is on the port but there's no
            // daemon token — likely a foreign process, not the Labalaba daemon.
            anyhow::anyhow!(
                "a process is already listening on 127.0.0.1:{port} but no Labalaba daemon token \
                 was found at {} — that port may be in use by another application",
                token_path.display()
            )
        } else {
            anyhow::anyhow!(
                "the daemon started but its token file did not appear at {}",
                token_path.display()
            )
        }
    })?;

    Ok((DaemonConnection { base_url, ws_url, token }, child))
}

fn read_token_with_retry(path: &std::path::Path) -> anyhow::Result<String> {
    for _ in 0..10 {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    anyhow::bail!("failed to read daemon token from {}", path.display())
}
