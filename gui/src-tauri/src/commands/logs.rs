use std::sync::Arc;
use labalaba_daemon::infrastructure::state::AppState;
use labalaba_daemon::application::log::get_logs::GetLogs;
use labalaba_shared::api::LogEntry;
use labalaba_shared::task::TaskId;
use uuid::Uuid;

#[tauri::command]
pub async fn get_logs(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    lines: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let task_id = TaskId(uuid);
    let uc = GetLogs { state };
    uc.execute(&task_id, lines.unwrap_or(500)).await.map_err(|e| e.to_string())
}
