use std::sync::Arc;
use axum::{extract::State, Json};
use axum::http::StatusCode;
use labalaba_shared::api::{ApiResponse, UpdateInfo};
use crate::application::update::check_update::CheckUpdate;
use crate::infrastructure::state::AppState;

type Resp<T> = (StatusCode, Json<ApiResponse<T>>);

pub async fn check(State(state): State<Arc<AppState>>) -> Resp<UpdateInfo> {
    let uc = CheckUpdate { state: Arc::clone(&state) };
    match uc.execute().await {
        Ok(info) => (StatusCode::OK, Json(ApiResponse::ok(info))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}
