use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::infrastructure::state::AppState;

pub struct StopTask {
    pub state: Arc<AppState>,
}

impl StopTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<()> {
        // Get task to access its PIDs
        let task = self.state.task_repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;

        // Check if there are any PIDs to stop
        if task.pids.is_empty() {
            anyhow::bail!("Task {} has no running PIDs", id);
        }

        {
            let mut states = self.state.runtime_states.write().await;
            if let Some(s) = states.get_mut(&id) {
                s.mark_stopping();
            }
        }

        // Kill all PIDs in the task (process tree kill)
        for pid in &task.pids {
            if let Err(e) = self.state.spawner.kill_tree(*pid).await {
                tracing::warn!("Failed to kill PID {} for task {}: {}", pid, id, e);
            }
        }

        // Clear PIDs and persist
        let mut task = task;
        task.pids.clear();
        self.state.task_repo.save(&task).await?;

        {
            let mut states = self.state.runtime_states.write().await;
            if let Some(s) = states.get_mut(&id) {
                s.mark_stopped(None);
            }
        }

        Ok(())
    }
}
