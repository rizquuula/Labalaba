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

        let child = std::process::Command::new(bin).spawn()?;

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
    let token = read_token_with_retry(&token_path)?;

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
