use std::sync::Arc;
use axum::{extract::State, Json};
use axum::http::StatusCode;
use labalaba_shared::api::{ApiResponse, AppSettings};
use crate::infrastructure::state::AppState;

type Resp<T> = (StatusCode, Json<ApiResponse<T>>);

pub async fn get(State(state): State<Arc<AppState>>) -> Resp<AppSettings> {
    let settings = state.settings.read().await.clone();
    (StatusCode::OK, Json(ApiResponse::ok(settings)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<AppSettings>,
) -> Resp<AppSettings> {
    // Update in-memory settings
    {
        let mut settings = state.settings.write().await;
        *settings = new_settings.clone();
    }
    
    // Persist to YAML file
    if let Err(e) = state.save_settings().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(format!("Failed to save settings: {}", e)))
        );
    }
    
    (StatusCode::OK, Json(ApiResponse::ok(new_settings)))
}
