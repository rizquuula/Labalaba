use std::sync::Arc;
use axum::{Router, routing};
use tower_http::cors::CorsLayer;
use crate::infrastructure::state::AppState;
use super::task_handler as tasks;
use super::settings_handler as settings;
use super::update_handler as updates;
use crate::interface::ws::log_handler;

pub fn build(state: Arc<AppState>) -> Router {
    Router::new()
        // Task CRUD
        .route("/api/tasks", routing::get(tasks::list).post(tasks::create))
        .route("/api/tasks/{id}", routing::get(tasks::get_one).put(tasks::update).delete(tasks::remove))
        // Task control
        .route("/api/tasks/{id}/start", routing::post(tasks::start))
        .route("/api/tasks/{id}/stop", routing::post(tasks::stop))
        .route("/api/tasks/{id}/restart", routing::post(tasks::restart))
        // Stats
        .route("/api/stats", routing::get(tasks::stats))
        // Settings
        .route("/api/settings", routing::get(settings::get).put(settings::update))
        // Updates
        .route("/api/update/check", routing::post(updates::check))
        // WebSocket log stream
        .route("/ws/logs/{id}", routing::get(log_handler::handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
