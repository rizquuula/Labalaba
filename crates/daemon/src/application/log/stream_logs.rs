use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::domain::log::entity::{LogReceiver, new_log_channel};
use crate::infrastructure::state::AppState;

pub struct StreamLogs {
    pub state: Arc<AppState>,
}

impl StreamLogs {
    /// Subscribe to a task's log broadcast channel.
    /// Creates a channel if the task hasn't started yet.
    pub async fn subscribe(&self, task_id: &TaskId) -> LogReceiver {
        let mut channels = self.state.log_channels.write().await;
        let tx = channels
            .entry(task_id.clone())
            .or_insert_with(new_log_channel);
        tx.subscribe()
    }
}
