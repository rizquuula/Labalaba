use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use labalaba_shared::api::{ApiResponse, LogEntry};
use labalaba_shared::task::TaskId;
use uuid::Uuid;
use crate::infrastructure::state::AppState;
use crate::application::log::get_logs::GetLogs;

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_limit")]
    pub lines: usize,
}

fn default_limit() -> usize { 500 }

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(task_id_str): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Json<ApiResponse<LogsResponse>> {
    let uc = GetLogs { state };
    
    let task_id = match Uuid::parse_str(&task_id_str) {
        Ok(uuid) => TaskId(uuid),
        Err(_) => return Json(ApiResponse::err("Invalid task ID")),
    };
    
    match uc.execute(&task_id, query.lines).await {
        Ok(logs) => Json(ApiResponse::ok(LogsResponse { logs })),
        Err(e) => Json(ApiResponse::err(format!("Failed to get logs: {}", e))),
    }
}
