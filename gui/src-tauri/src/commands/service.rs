use std::path::Path;

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let daemon_path = crate::commands::daemon::resolve_daemon_bin()
        .ok_or_else(|| "daemon binary not found".to_string())?;

    if enabled {
        install_autostart(&daemon_path).map_err(|e| e.to_string())
    } else {
        uninstall_autostart().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    Ok(is_installed())
}

// ── Linux (systemd --user) ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn unit_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("systemd/user/labalaba-daemon.service"))
}

#[cfg(target_os = "linux")]
fn unit_contents(daemon_path: &Path) -> String {
    format!(
        "[Unit]\nDescription=Labalaba Daemon\nAfter=network.target\n\n\
         [Service]\nExecStart={}\nRestart=on-failure\n\n\
         [Install]\nWantedBy=default.target\n",
        daemon_path.display()
    )
}

#[cfg(target_os = "linux")]
fn install_autostart(daemon_path: &Path) -> std::io::Result<()> {
    let unit = unit_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "cannot determine config dir")
    })?;

    if let Some(parent) = unit.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&unit, unit_contents(daemon_path))?;

    let reload = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    if let Err(e) = reload {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("systemctl not available: {e}"),
        ));
    }

    let enable = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", "labalaba-daemon.service"])
        .status()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    if !enable.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("systemctl enable returned exit code {:?}", enable.code()),
        ));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_autostart() -> std::io::Result<()> {
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "labalaba-daemon.service"])
        .status();

    if let Some(unit) = unit_path() {
        if unit.exists() {
            std::fs::remove_file(&unit)?;
        }
    }

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    Ok(())
}

#[cfg(target_os = "linux")]
fn is_installed() -> bool {
    unit_path().map(|p| p.exists()).unwrap_or(false)
}

// ── macOS (launchd LaunchAgent) ───────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn plist_path() -> Option<std::path::PathBuf> {
    dirs::home_dir()
        .map(|d| d.join("Library/LaunchAgents/com.rizquuula.labalaba.daemon.plist"))
}

#[cfg(target_os = "macos")]
fn plist_contents(daemon_path: &Path) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>com.rizquuula.labalaba.daemon</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         \t\t<string>{}</string>\n\
         \t</array>\n\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         \t<key>KeepAlive</key>\n\
         \t<true/>\n\
         </dict>\n\
         </plist>\n",
        daemon_path.display()
    )
}

#[cfg(target_os = "macos")]
fn install_autostart(daemon_path: &Path) -> std::io::Result<()> {
    let plist = plist_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "cannot determine home dir")
    })?;

    if let Some(parent) = plist.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&plist, plist_contents(daemon_path))?;

    let _ = std::process::Command::new("launchctl")
        .args(["load", "-w", plist.to_str().unwrap_or("")])
        .status();

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_autostart() -> std::io::Result<()> {
    if let Some(plist) = plist_path() {
        let _ = std::process::Command::new("launchctl")
            .args(["unload", "-w", plist.to_str().unwrap_or("")])
            .status();
        if plist.exists() {
            std::fs::remove_file(&plist)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn is_installed() -> bool {
    plist_path().map(|p| p.exists()).unwrap_or(false)
}

// ── Windows (HKCU Run key via reg.exe) ───────────────────────────────────────

#[cfg(target_os = "windows")]
const REG_KEY: &str =
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const REG_VALUE: &str = "Labalaba Daemon";

#[cfg(target_os = "windows")]
fn install_autostart(daemon_path: &Path) -> std::io::Result<()> {
    let status = std::process::Command::new("reg")
        .args([
            "add",
            REG_KEY,
            "/v",
            REG_VALUE,
            "/t",
            "REG_SZ",
            "/d",
            daemon_path.to_str().unwrap_or(""),
            "/f",
        ])
        .status()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("reg add returned exit code {:?}", status.code()),
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_autostart() -> std::io::Result<()> {
    let _ = std::process::Command::new("reg")
        .args(["delete", REG_KEY, "/v", REG_VALUE, "/f"])
        .status();
    Ok(())
}

#[cfg(target_os = "windows")]
fn is_installed() -> bool {
    std::process::Command::new("reg")
        .args(["query", REG_KEY, "/v", REG_VALUE])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
