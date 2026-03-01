use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use labalaba_shared::api::ApiResponse;
use labalaba_shared::task::{TaskDto, TaskId, TaskRequest, TaskStats};
use uuid::Uuid;
use crate::application::dto::task_to_dto;
use crate::application::task::{
    create_task::CreateTask, delete_task::DeleteTask, edit_task::EditTask,
    restart_task::RestartTask, start_task::StartTask, stop_task::StopTask,
};
use crate::domain::task::status::TaskRuntimeState;
use crate::infrastructure::state::AppState;
use labalaba_shared::task::TaskStatus;

type Resp<T> = (StatusCode, Json<ApiResponse<T>>);

fn ok<T: serde::Serialize>(data: T) -> Resp<T> {
    (StatusCode::OK, Json(ApiResponse::ok(data)))
}

fn err<T>(msg: impl Into<String>, code: StatusCode) -> Resp<T> {
    (code, Json(ApiResponse::err(msg)))
}

pub async fn list(State(state): State<Arc<AppState>>) -> Resp<Vec<TaskDto>> {
    match state.task_repo.find_all().await {
        Ok(tasks) => {
            let states = state.runtime_states.read().await;
            let dtos = tasks.iter().map(|t| {
                let rt = states.get(&t.id).cloned().unwrap_or_default();
                task_to_dto(t, &rt)
            }).collect();
            ok(dtos)
        }
        Err(e) => err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Resp<TaskDto> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let id = TaskId(uuid);
    match state.task_repo.find_by_id(&id).await {
        Ok(Some(task)) => {
            let states = state.runtime_states.read().await;
            let rt = states.get(&id).cloned().unwrap_or_default();
            ok(task_to_dto(&task, &rt))
        }
        Ok(None) => err("Task not found", StatusCode::NOT_FOUND),
        Err(e) => err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TaskRequest>,
) -> Resp<TaskDto> {
    let uc = CreateTask { repo: state.task_repo.as_ref() };
    match uc.execute(req).await {
        Ok(task) => {
            let rt = TaskRuntimeState::default();
            ok(task_to_dto(&task, &rt))
        }
        Err(e) => err(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(req): Json<TaskRequest>,
) -> Resp<TaskDto> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let id = TaskId(uuid);
    let uc = EditTask { repo: state.task_repo.as_ref() };
    match uc.execute(id.clone(), req).await {
        Ok(task) => {
            let states = state.runtime_states.read().await;
            let rt = states.get(&id).cloned().unwrap_or_default();
            ok(task_to_dto(&task, &rt))
        }
        Err(e) => err(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

pub async fn remove(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Resp<()> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let id = TaskId(uuid);
    let uc = DeleteTask { repo: state.task_repo.as_ref() };
    match uc.execute(id).await {
        Ok(()) => ok(()),
        Err(e) => err(e.to_string(), StatusCode::NOT_FOUND),
    }
}

pub async fn start(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Resp<u32> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let uc = StartTask { state: Arc::clone(&state) };
    match uc.execute(TaskId(uuid)).await {
        Ok(pid) => ok(pid),
        Err(e) => err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn stop(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Resp<()> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let uc = StopTask { state: Arc::clone(&state) };
    match uc.execute(TaskId(uuid)).await {
        Ok(()) => ok(()),
        Err(e) => err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn restart(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Resp<u32> {
    let Ok(uuid) = Uuid::parse_str(&id_str) else {
        return err("Invalid task ID", StatusCode::BAD_REQUEST);
    };
    let uc = RestartTask { state: Arc::clone(&state) };
    match uc.execute(TaskId(uuid)).await {
        Ok(pid) => ok(pid),
        Err(e) => err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn stats(State(state): State<Arc<AppState>>) -> Resp<TaskStats> {
    let tasks = match state.task_repo.find_all().await {
        Ok(t) => t,
        Err(e) => return err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    };
    let states = state.runtime_states.read().await;
    let total = tasks.len();
    let running = tasks.iter()
        .filter(|t| states.get(&t.id).map(|s| s.is_running()).unwrap_or(false))
        .count();
    let crashed = tasks.iter()
        .filter(|t| matches!(
            states.get(&t.id).map(|s| &s.status),
            Some(TaskStatus::Crashed)
        ))
        .count();
    ok(TaskStats { total, running, stopped: total - running - crashed, crashed })
}
