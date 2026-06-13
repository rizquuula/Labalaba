use axum::http::StatusCode;
use axum::Json;
use labalaba_shared::api::ApiResponse;

type Resp<T> = (StatusCode, Json<ApiResponse<T>>);

#[derive(serde::Serialize)]
pub struct HealthInfo {
    pub status: String,
    pub version: String,
}

#[derive(serde::Serialize)]
pub struct VersionInfo {
    pub version: String,
}

pub async fn health() -> Resp<HealthInfo> {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(HealthInfo {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })),
    )
}

pub async fn version() -> Resp<VersionInfo> {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
        })),
    )
}
