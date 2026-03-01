use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::infrastructure::state::AppState;
use super::{stop_task::StopTask, start_task::StartTask};

pub struct RestartTask {
    pub state: Arc<AppState>,
}

impl RestartTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<u32> {
        // Stop if running (ignore error if already stopped)
        let _ = StopTask { state: Arc::clone(&self.state) }
            .execute(id.clone())
            .await;

        // Give the OS a moment to release resources
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        StartTask { state: Arc::clone(&self.state) }
            .execute(id)
            .await
    }
}
