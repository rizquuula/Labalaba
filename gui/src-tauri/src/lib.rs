mod commands;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Manager, RunEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use commands::daemon::{DaemonHandle, connect_in_background};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let quitting = Arc::new(AtomicBool::new(false));

    let quitting_window = Arc::clone(&quitting);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            // An installer may have torn down the autostart entry while replacing
            // our files (see restore_autostart_after_update). Put it back before
            // anything else touches the daemon.
            commands::daemon::restore_autostart_after_update();

            // Daemon lifecycle. Managed empty and filled in off-thread: setup
            // runs before the event loop starts, so connecting here would leave
            // the window unpainted for as long as it takes. `get_daemon_connection`
            // waits for the result, so the frontend still just awaits it once.
            app.manage(DaemonHandle::default());
            connect_in_background(app.handle().clone());

            // Tray icon
            let quitting_quit = Arc::clone(&quitting);

            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Labalaba")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        quitting_quit.store(true, Ordering::SeqCst);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                });
            // Use the app's window icon when available; skip otherwise rather
            // than feeding the tray a zero-size image (which misbehaves on some
            // platforms). Production bundles always provide an icon.
            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            tray.build(app)?;

            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if !quitting_window.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::daemon::get_daemon_connection,
            commands::daemon::daemon_status,
            commands::daemon::start_daemon,
            commands::daemon::cleanup_daemon,
            commands::daemon::prepare_for_update,
            commands::portable::get_data_location,
            commands::portable::set_portable_mode,
            commands::service::set_autostart,
            commands::service::get_autostart,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                if let Some(handle) = app_handle.try_state::<DaemonHandle>() {
                    let child = handle.child.lock().unwrap().take();
                    if let Some(mut child) = child {
                        // Ask it to exit cleanly rather than TerminateProcess-ing
                        // it: only the graceful path flushes the log writers, and
                        // tokio's BufWriter does not flush on drop, so a hard kill
                        // silently drops each task's most recent output.
                        //
                        // Running tasks are deliberately left alive — the daemon's
                        // shutdown does not touch them, and recover_task_states
                        // re-adopts them on the next launch.
                        let _ = tauri::async_runtime::block_on(
                            labalaba_daemon::stop_running_daemon(),
                        );
                        if matches!(child.try_wait(), Ok(None)) {
                            let _ = child.kill();
                        }
                        let _ = child.wait();
                    }
                }
            }
        });
}
