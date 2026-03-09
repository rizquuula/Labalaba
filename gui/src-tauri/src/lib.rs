mod commands;

use std::sync::Arc;
use tauri::{Emitter, Manager};
use labalaba_daemon::init_app_state;
use labalaba_shared::api::{LogEntry, UpdateInfo};
use commands::{
    tasks::{list_tasks, get_task, create_task, update_task, delete_task,
            start_task, stop_task, restart_task, get_stats},
    settings::{get_settings, update_settings, check_update},
    logs::get_logs,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Wire log entries → Tauri events so the frontend can listen("log:{task_id}", cb)
            let log_app_handle = app_handle.clone();
            let log_cb: Arc<dyn Fn(LogEntry) + Send + Sync + 'static> =
                Arc::new(move |entry: LogEntry| {
                    let event = format!("log:{}", entry.task_id);
                    log_app_handle.emit(&event, &entry).ok();
                });

            // Wire update events → Tauri events for update popup
            let update_app_handle = app_handle.clone();
            let update_cb: Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static> =
                Arc::new(move |info: UpdateInfo| {
                    update_app_handle.emit("update-available", &info).ok();
                });

            let state = tauri::async_runtime::block_on(init_app_state(Some(log_cb), Some(update_cb)))
                .expect("Failed to initialize daemon state");

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            get_task,
            create_task,
            update_task,
            delete_task,
            start_task,
            stop_task,
            restart_task,
            get_stats,
            get_settings,
            update_settings,
            check_update,
            get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
