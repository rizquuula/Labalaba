use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use labalaba_shared::api::ApiResponse;
use crate::infrastructure::state::AppState;
use crate::infrastructure::system::interpreter::detect_interpreter;

type Resp<T> = (StatusCode, Json<ApiResponse<T>>);

#[derive(serde::Deserialize)]
pub struct DetectInterpreterRequest {
    pub kind: String,
}

pub async fn detect(Json(req): Json<DetectInterpreterRequest>) -> Resp<Option<String>> {
    let path = detect_interpreter(&req.kind);
    (StatusCode::OK, Json(ApiResponse::ok(path)))
}

/// Request a graceful shutdown of the daemon. The response is sent before the
/// server stops accepting connections; the actual exit happens via the bin's
/// graceful-shutdown path (which flushes log writers). Authenticated like all
/// other /api routes.
pub async fn shutdown(State(state): State<Arc<AppState>>) -> Resp<()> {
    tracing::info!("Shutdown requested via API");
    state.shutdown_notify.notify_one();
    (StatusCode::OK, Json(ApiResponse::ok(())))
}
