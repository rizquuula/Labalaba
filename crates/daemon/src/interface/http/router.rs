use super::health_handler as health;
use super::logs_handler as logs;
use super::settings_handler as settings;
use super::system_handler as system;
use super::task_handler as tasks;
use super::update_handler as updates;
use crate::infrastructure::state::AppState;
use crate::interface::ws::log_handler;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{middleware, routing, Json, Router};
use labalaba_shared::api::ApiResponse;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

async fn require_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let authorized = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| token == state.auth_token)
        .unwrap_or(false);

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::err("unauthorized")),
        )
            .into_response();
    }

    next.run(req).await
}

pub fn build(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        // Task CRUD
        .route("/api/tasks", routing::get(tasks::list).post(tasks::create))
        .route(
            "/api/tasks/{id}",
            routing::get(tasks::get_one)
                .put(tasks::update)
                .delete(tasks::remove),
        )
        // Task control
        .route("/api/tasks/{id}/start", routing::post(tasks::start))
        .route("/api/tasks/{id}/stop", routing::post(tasks::stop))
        .route("/api/tasks/{id}/restart", routing::post(tasks::restart))
        // Task resource stats
        .route("/api/tasks/{id}/stats", routing::get(tasks::get_task_stats))
        // Stats
        .route("/api/stats", routing::get(tasks::stats))
        // Settings
        .route(
            "/api/settings",
            routing::get(settings::get).put(settings::update),
        )
        // Updates
        .route("/api/update/check", routing::post(updates::check))
        .route("/api/update/pending", routing::get(updates::pending))
        // Version (protected; /api/health remains public)
        .route("/api/app/version", routing::get(health::version))
        // System utilities
        .route("/api/system/detect-interpreter", routing::post(system::detect))
        .route("/api/system/shutdown", routing::post(system::shutdown))
        // Historical logs
        .route("/api/logs/{id}", routing::get(logs::handler))
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            require_token,
        ));

    let public = Router::new()
        // Health — always public (liveness probing)
        .route("/api/health", routing::get(health::health))
        // WebSocket log stream — token validated inside the handler via ?token=
        .route("/ws/logs/{id}", routing::get(log_handler::handler));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(CorsLayer::permissive())
        .with_state(state)
}
