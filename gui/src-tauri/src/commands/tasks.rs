use std::sync::Arc;
use labalaba_daemon::infrastructure::state::AppState;
use labalaba_daemon::application::{
    dto::task_to_dto,
    task::{
        create_task::CreateTask,
        delete_task::DeleteTask,
        edit_task::EditTask,
        restart_task::RestartTask,
        start_task::StartTask,
        stop_task::StopTask,
    },
};
use labalaba_daemon::domain::task::status::TaskRuntimeState;
use labalaba_shared::task::{TaskDto, TaskRequest, TaskStats, TaskStatus};
use uuid::Uuid;
use labalaba_shared::task::TaskId;

#[tauri::command]
pub async fn list_tasks(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<TaskDto>, String> {
    let state = Arc::clone(&*state);
    let tasks = state.task_repo.find_all().await.map_err(|e| e.to_string())?;
    let states = state.runtime_states.read().await;
    let default_rt = TaskRuntimeState::default();
    let dtos = tasks.iter()
        .map(|t| task_to_dto(t, states.get(&t.id).unwrap_or(&default_rt)))
        .collect();
    Ok(dtos)
}

#[tauri::command]
pub async fn get_task(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<TaskDto, String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let task_id = TaskId(uuid);
    let task = state.task_repo.find_by_id(&task_id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())?;
    let states = state.runtime_states.read().await;
    let default_rt = TaskRuntimeState::default();
    let rt = states.get(&task_id).unwrap_or(&default_rt);
    Ok(task_to_dto(&task, rt))
}

#[tauri::command]
pub async fn create_task(state: tauri::State<'_, Arc<AppState>>, req: TaskRequest) -> Result<TaskDto, String> {
    let state = Arc::clone(&*state);
    let uc = CreateTask { repo: Arc::clone(&state.task_repo) };
    let task = uc.execute(req).await.map_err(|e| e.to_string())?;
    Ok(task_to_dto(&task, &TaskRuntimeState::default()))
}

#[tauri::command]
pub async fn update_task(state: tauri::State<'_, Arc<AppState>>, id: String, req: TaskRequest) -> Result<TaskDto, String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let task_id = TaskId(uuid);
    let uc = EditTask { repo: Arc::clone(&state.task_repo) };
    let task = uc.execute(task_id.clone(), req).await.map_err(|e| e.to_string())?;
    let states = state.runtime_states.read().await;
    let default_rt = TaskRuntimeState::default();
    let rt = states.get(&task_id).unwrap_or(&default_rt);
    Ok(task_to_dto(&task, rt))
}

#[tauri::command]
pub async fn delete_task(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let uc = DeleteTask { repo: Arc::clone(&state.task_repo) };
    uc.execute(TaskId(uuid)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_task(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<u32, String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let uc = StartTask { state: Arc::clone(&state) };
    uc.execute(TaskId(uuid)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_task(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let uc = StopTask { state: Arc::clone(&state) };
    uc.execute(TaskId(uuid)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restart_task(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<u32, String> {
    let state = Arc::clone(&*state);
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid task ID".to_string())?;
    let uc = RestartTask { state: Arc::clone(&state) };
    uc.execute(TaskId(uuid)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats(state: tauri::State<'_, Arc<AppState>>) -> Result<TaskStats, String> {
    let state = Arc::clone(&*state);
    let tasks = state.task_repo.find_all().await.map_err(|e| e.to_string())?;
    let states = state.runtime_states.read().await;
    let total = tasks.len();
    let running = tasks.iter()
        .filter(|t| states.get(&t.id).map(|s| s.is_running()).unwrap_or(false))
        .count();
    let crashed = tasks.iter()
        .filter(|t| matches!(states.get(&t.id).map(|s| &s.status), Some(TaskStatus::Crashed)))
        .count();
    Ok(TaskStats { total, running, stopped: total - running - crashed, crashed })
}
