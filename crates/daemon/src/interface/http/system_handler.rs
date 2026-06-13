use axum::http::StatusCode;
use axum::Json;
use labalaba_shared::api::ApiResponse;
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
