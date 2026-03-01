use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::infrastructure::state::AppState;

pub struct StopTask {
    pub state: Arc<AppState>,
}

impl StopTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<()> {
        let pid = {
            let states = self.state.runtime_states.read().await;
            states.get(&id)
                .and_then(|s| s.pid)
                .ok_or_else(|| anyhow::anyhow!("Task {} is not running", id))?
        };

        {
            let mut states = self.state.runtime_states.write().await;
            if let Some(s) = states.get_mut(&id) {
                s.mark_stopping();
            }
        }

        self.state.spawner.kill(pid).await?;

        {
            let mut states = self.state.runtime_states.write().await;
            if let Some(s) = states.get_mut(&id) {
                s.mark_stopped(None);
            }
        }

        Ok(())
    }
}
