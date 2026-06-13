mod commands;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Manager, RunEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use commands::daemon::{DaemonHandle, start_or_connect_daemon};

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
        .setup(move |app| {
            // Daemon lifecycle
            match start_or_connect_daemon() {
                Ok((conn, child)) => {
                    let handle = DaemonHandle {
                        connection: std::sync::Mutex::new(Some(conn)),
                        child: std::sync::Mutex::new(child),
                    };
                    app.manage(handle);
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon: {e}");
                    app.manage(DaemonHandle::default());
                }
            }

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
            commands::service::set_autostart,
            commands::service::get_autostart,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                if let Some(handle) = app_handle.try_state::<DaemonHandle>() {
                    if let Some(mut child) = handle.child.lock().unwrap().take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
            }
        });
}
