//! Portable mode: keep `tasks.yaml`, `settings.yaml`, `daemon.token` and
//! `logs/` in a `data\` folder next to the installed binaries instead of the
//! per-user data dir.
//!
//! This lives in the GUI, not the daemon, for two reasons: switching means
//! stopping and restarting the daemon (which the daemon cannot do to itself),
//! and the token the GUI authenticates with moves, so only the GUI can pick the
//! new one up. That matches the existing split — the GUI owns daemon lifecycle.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::commands::daemon::{
    is_listening, reclaim_port, start_or_connect_daemon, DaemonConnection, DaemonHandle,
};
use tauri::Manager;

const MARKER_CONTENTS: &str = "\
This file makes Labalaba keep its data in the `data` folder next to this file,
instead of in %APPDATA%\\labalaba. Delete it to go back — Labalaba will not move
the data for you. The Data Location section in Settings does both properly.
";

/// The write probe's filename. Not `.tmp`: it must never look like something a
/// cleaner should collect while we still hold the invariant that its absence
/// means "we cleaned up".
const PROBE_FILE: &str = ".labalaba-write-probe";

#[derive(Clone, serde::Serialize)]
pub struct DataLocation {
    /// Where the daemon reads and writes today.
    pub data_dir: String,
    /// What the "Reveal" button should point the file explorer at.
    pub reveal_path: String,
    pub portable_active: bool,
    /// Whether this platform supports portable mode at all.
    pub portable_supported: bool,
    /// Whether the toggle can actually be used right now.
    pub portable_available: bool,
    /// Why not, when `portable_available` is false. Shown inline in Settings —
    /// a disabled control with no reason is a support ticket.
    pub reason: Option<String>,
    /// Where a toggle would move the data to.
    pub target_dir: Option<String>,
    /// The target already holds a task file, so it — not the current one — is
    /// what the daemon would come up on. The user has to be told.
    pub target_has_data: bool,
    /// Last-modified time of that task file, in epoch milliseconds. Formatted
    /// by the frontend, which knows the user's locale.
    pub target_modified_ms: Option<u64>,
}

fn modified_ms(path: &Path) -> Option<u64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let since = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(since.as_millis() as u64)
}

/// Actually write a file and delete it again.
///
/// Not `metadata().permissions().readonly()`: on Windows that reflects the
/// read-only *attribute*, not the ACL, so it reports a `C:\Program Files`
/// directory as perfectly writable and we would only find out after having
/// already stopped the daemon.
fn probe_writable(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{} cannot be created: {e}", dir.display()))?;
    let probe = dir.join(PROBE_FILE);
    std::fs::write(&probe, b"").map_err(|e| format!("{} is not writable: {e}", dir.display()))?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

/// A dev override that pins the data dir, if any — portable mode would be
/// ignored under it, so the toggle would lie.
fn env_override() -> Option<String> {
    for (var, what) in [
        ("LABALABA_DATA_DIR", "the data directory is pinned by LABALABA_DATA_DIR"),
        // The GUI and the daemon each look for the marker next to their *own*
        // executable. Normally that is the same directory. Point the daemon
        // elsewhere and the two disagree about where the data lives — the GUI
        // would read one base and the daemon write another.
        ("LABALABA_DAEMON_BIN", "the daemon binary is overridden by LABALABA_DAEMON_BIN"),
    ] {
        if std::env::var(var).map(|v| !v.is_empty()).unwrap_or(false) {
            return Some(format!("{what} (development override)"));
        }
    }
    None
}

fn describe_location() -> DataLocation {
    let base = labalaba_daemon::data_dir();
    let (settings, _) = tauri::async_runtime::block_on(labalaba_daemon::load_settings());

    // Reveal the task file when it exists — that is what people are actually
    // looking for. It only appears once a task has been saved, and
    // `revealItemInDir` errors on a missing path, so fall back to the directory.
    let config = labalaba_daemon::resolve(&base, &settings.config_path);
    let reveal_path = if config.exists() { config } else { base.clone() };

    let portable_supported = labalaba_daemon::portable_marker_dir().is_some();
    let portable_active = labalaba_daemon::is_portable_active();

    // The dir a toggle would land on: the opposite of wherever we are now.
    let target_dir = if portable_active {
        Some(labalaba_daemon::platform_data_dir())
    } else {
        labalaba_daemon::portable_data_dir()
    };

    let target_tasks = target_dir
        .as_ref()
        .map(|d| labalaba_daemon::resolve(d, &settings.config_path));
    let target_has_data = target_tasks.as_ref().map(|p| p.exists()).unwrap_or(false);
    let target_modified_ms = target_tasks.as_deref().and_then(modified_ms);

    let reason = if !portable_supported {
        Some("Portable mode is available on Windows only.".to_string())
    } else if let Some(o) = env_override() {
        Some(format!("Unavailable because {o}."))
    } else {
        // Enabling needs to write the marker next to the exe; disabling needs to
        // delete it there. Either way that directory has to be writable.
        labalaba_daemon::portable_marker_dir()
            .and_then(|d| probe_writable(&d).err())
            .map(|e| {
                format!(
                    "Labalaba is installed somewhere that needs administrator rights, so it \
                     cannot keep data there: {e}"
                )
            })
    };

    DataLocation {
        data_dir: base.to_string_lossy().into_owned(),
        reveal_path: reveal_path.to_string_lossy().into_owned(),
        portable_active,
        portable_supported,
        portable_available: reason.is_none(),
        reason,
        target_dir: target_dir.map(|d| d.to_string_lossy().into_owned()),
        target_has_data,
        target_modified_ms,
    }
}

#[tauri::command]
pub fn get_data_location() -> DataLocation {
    describe_location()
}

/// Releases the `switching` flag however the switch ends — including the early
/// returns, of which there are many.
struct SwitchGuard<'a>(&'a AtomicBool);

impl Drop for SwitchGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn publish(handle: &DaemonHandle, conn: DaemonConnection, child: Option<std::process::Child>) {
    *handle.connection.lock().unwrap() = Some(conn);
    let mut guard = handle.child.lock().unwrap();
    if let Some(mut old) = guard.take() {
        let _ = old.try_wait();
    }
    *guard = child;
    *handle.error.lock().unwrap() = None;
}

/// Move the data and bring the daemon back up on it.
///
/// **The order here is load-bearing.** `stop_running_daemon` takes no arguments:
/// it re-resolves both the port and the token through `data_dir()` at call time.
/// Write the marker first and `data_dir()` flips to a directory holding neither
/// file, so the token read fails, it reports "not running" without ever sending
/// the shutdown, and the port falls back to the 27015 default. On a default port
/// that costs a graceful shutdown (a force-kill drops every task's buffered log
/// lines, since tokio's BufWriter does not flush on drop). On a *custom* port it
/// is worse and silent: the old daemon keeps running on the old port against the
/// old data while the new one binds the default port against the new data — two
/// schedulers and two auto-restart loops writing two diverging `tasks.yaml`.
///
/// So: everything that resolves state implicitly runs *before* the marker flip,
/// and the flip is the single commit point. Every step before it is either
/// read-only or a copy into a directory nobody is reading yet, so any failure
/// leaves the old world exactly as it was.
fn switch_portable_mode(app: &tauri::AppHandle, enabled: bool) -> Result<DataLocation, String> {
    let handle = app.state::<DaemonHandle>();

    if handle
        .switching
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("a data-location switch is already running".into());
    }
    let _guard = SwitchGuard(&handle.switching);

    // --- 1. Pre-checks. Nothing destructive has happened yet. ---
    let marker_dir = labalaba_daemon::portable_marker_dir()
        .ok_or_else(|| "portable mode is available on Windows only".to_string())?;

    if let Some(o) = env_override() {
        return Err(format!(
            "cannot switch while {o}. Unset it and restart Labalaba."
        ));
    }

    let old_base = labalaba_daemon::data_dir();
    let new_base = if enabled {
        marker_dir.join(labalaba_daemon::PORTABLE_DATA_SUBDIR)
    } else {
        labalaba_daemon::platform_data_dir()
    };
    if old_base == new_base {
        return Ok(describe_location());
    }

    probe_writable(&marker_dir)?;
    probe_writable(&new_base)?;

    // --- 2. Read the port while data_dir() still resolves to the old base. ---
    let (old_settings, _) = tauri::async_runtime::block_on(labalaba_daemon::load_settings());
    let old_port = old_settings.daemon_port;

    // --- 3. Stop, still under the old data dir. ---
    // `reclaim_port` sends the authenticated shutdown first and only force-kills
    // if that fails, so the log writers get their chance to flush.
    reclaim_port(old_port);
    if let Some(mut old) = handle.child.lock().unwrap().take() {
        let _ = old.try_wait();
    }
    if is_listening(old_port) {
        return Err(format!(
            "could not stop the daemon on port {old_port}, so the data location is unchanged. \
             Stop it and try again."
        ));
    }

    // --- 4. Copy. Both bases are explicit paths; nothing here reads data_dir(). ---
    // After the stop, so we cannot capture a half-written tasks.yaml or lose a
    // write the daemon makes to the base we are abandoning.
    let report = labalaba_daemon::copy_data_dir_for_switch(&old_base, &new_base, &old_settings);
    if let Some(pinned) = &report.skipped_absolute_config {
        eprintln!(
            "portable switch: config_path is absolute ({}), leaving the task file where it is",
            pinned.display()
        );
    }

    // --- 5. Flip the marker. data_dir() changes HERE. ---
    let marker = marker_dir.join(labalaba_daemon::PORTABLE_MARKER);
    let flip = if enabled {
        std::fs::write(&marker, MARKER_CONTENTS)
    } else {
        match std::fs::remove_file(&marker) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    };
    if let Err(e) = flip {
        // Nothing has committed. Put the daemon back and report.
        let _ = restart_at(&handle, &old_base);
        return Err(format!("could not update {}: {e}", marker.display()));
    }

    // --- 6. Respawn against the new base. ---
    match start_or_connect_daemon() {
        Ok((conn, child)) => {
            publish(&handle, conn, child);
            Ok(describe_location())
        }
        // --- 7. Roll back. The copy stays: it is only a copy, and deleting is
        // the far riskier operation. The source was never touched, so undoing
        // the marker restores the exact previous state.
        Err(e) => {
            let undo = if enabled {
                std::fs::remove_file(&marker).map(|_| ())
            } else {
                std::fs::write(&marker, MARKER_CONTENTS)
            };
            if let Err(u) = undo {
                *handle.error.lock().unwrap() = Some(format!("{e}"));
                return Err(format!(
                    "the daemon would not start at {} ({e}), and {} could not be restored ({u}). \
                     Restart Labalaba.",
                    new_base.display(),
                    marker.display()
                ));
            }
            match restart_at(&handle, &old_base) {
                Ok(()) => Err(format!(
                    "the daemon would not start at {} ({e}). Reverted to {}.",
                    new_base.display(),
                    old_base.display()
                )),
                Err(e2) => Err(format!(
                    "the daemon would not start at {} ({e}), and could not be restarted at {} \
                     either ({e2}). Restart Labalaba.",
                    new_base.display(),
                    old_base.display()
                )),
            }
        }
    }
}

/// Bring the daemon back up after an aborted switch. `base` is only for the
/// message — resolution follows the marker, which the caller has already put back.
fn restart_at(handle: &DaemonHandle, base: &Path) -> Result<(), String> {
    match start_or_connect_daemon() {
        Ok((conn, child)) => {
            publish(handle, conn, child);
            Ok(())
        }
        Err(e) => {
            let msg = format!("could not restart the daemon at {}: {e}", base.display());
            *handle.error.lock().unwrap() = Some(msg.clone());
            Err(msg)
        }
    }
}

/// Move the data dir and restart the daemon onto it.
///
/// `async` + a worker thread, not a plain sync command: Tauri runs sync commands
/// on the main thread and this blocks for 5-25s (shutdown, copy, spawn
/// readiness). A plain std thread rather than `spawn_blocking` because the work
/// calls `tauri::async_runtime::block_on`, which must not run inside a tokio
/// context — the same reason `connect_in_background` uses one.
#[tauri::command]
pub async fn set_portable_mode(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<DataLocation, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(switch_portable_mode(&app, enabled));
    });
    rx.await
        .map_err(|_| "the data-location switch failed unexpectedly".to_string())?
}
