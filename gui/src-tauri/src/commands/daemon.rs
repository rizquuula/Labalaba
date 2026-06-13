use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, serde::Serialize)]
pub struct DaemonConnection {
    pub base_url: String,
    pub ws_url: String,
    pub token: String,
}

pub struct DaemonHandle {
    pub connection: Mutex<Option<DaemonConnection>>,
    pub child: Mutex<Option<std::process::Child>>,
}

impl Default for DaemonHandle {
    fn default() -> Self {
        Self {
            connection: Mutex::new(None),
            child: Mutex::new(None),
        }
    }
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
    let (conn, child) = start_or_connect_daemon().map_err(|e| e.to_string())?;
    *state.connection.lock().unwrap() = Some(conn);
    let mut guard = state.child.lock().unwrap();
    if let Some(mut old) = guard.take() {
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

#[tauri::command]
pub fn get_daemon_connection(
    state: tauri::State<'_, DaemonHandle>,
) -> Result<DaemonConnection, String> {
    state
        .connection
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "daemon not connected".into())
}

fn is_listening(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
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

    let child = if is_listening(port) {
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
        let child = cmd.spawn()?;

        let mut ready = false;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            if is_listening(port) {
                ready = true;
                break;
            }
        }
        if !ready {
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
