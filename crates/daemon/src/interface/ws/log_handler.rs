use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use labalaba_shared::task::TaskId;
use crate::application::log::stream_logs::StreamLogs;
use crate::infrastructure::state::AppState;

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Response {
    let id = Uuid::parse_str(&id_str)
        .map(TaskId)
        .unwrap_or_else(|_| TaskId::new());

    ws.on_upgrade(move |socket| stream_to_socket(socket, state, id))
}

async fn stream_to_socket(socket: WebSocket, state: Arc<AppState>, task_id: TaskId) {
    let (mut sender, mut receiver) = socket.split();

    let uc = StreamLogs { state };
    let mut log_rx = uc.subscribe(&task_id).await;

    // Forward log entries to the WebSocket client
    loop {
        tokio::select! {
            entry = log_rx.recv() => {
                match entry {
                    Ok(log) => {
                        if let Ok(json) = serde_json::to_string(&log) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WS log subscriber lagged by {} messages", n);
                    }
                    Err(_) => break,
                }
            }
            // Handle client close
            msg = receiver.next() => {
                if msg.is_none() { break; }
            }
        }
    }
}
