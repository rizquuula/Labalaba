#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let daemon_path = crate::commands::daemon::resolve_daemon_bin()
        .ok_or_else(|| "daemon binary not found".to_string())?;

    if enabled {
        labalaba_daemon::infrastructure::autostart::install(&daemon_path)
            .map_err(|e| e.to_string())
    } else {
        labalaba_daemon::infrastructure::autostart::uninstall()
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    Ok(labalaba_daemon::infrastructure::autostart::is_installed())
}

pub(crate) fn is_autostart_installed() -> bool {
    labalaba_daemon::infrastructure::autostart::is_installed()
}
