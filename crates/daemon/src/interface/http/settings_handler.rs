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
    let mut settings = state.settings.write().await;
    *settings = new_settings.clone();
    (StatusCode::OK, Json(ApiResponse::ok(new_settings)))
}
