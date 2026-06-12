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

        // Mark Stopping BEFORE killing so the exit watcher reliably sees the
        // deliberate stop and treats the resulting exit as intentional rather
        // than a crash (which would otherwise trigger an auto-restart).
        {
            let mut states = self.state.runtime_states.write().await;
            states.entry(id.clone()).or_default().mark_stopping();
        }

        // Kill all PIDs in the task (process tree kill)
        for pid in &task.pids {
            if let Err(e) = self.state.spawner.kill_tree(*pid).await {
                tracing::warn!("Failed to kill PID {} for task {}: {}", pid, id, e);
            }
        }

        // Clear PIDs via a locked read-modify-write so a concurrent start that
        // just pushed a fresh PID isn't clobbered by saving a stale task copy.
        self.state.task_repo.update_pids(&id, Box::new(|_pids| Vec::new())).await?;

        // Unregister from resource monitor
        self.state.resource_monitor.unregister_task(&id).await;

        {
            let mut states = self.state.runtime_states.write().await;
            if let Some(s) = states.get_mut(&id) {
                s.mark_stopped(None);
            }
        }

        Ok(())
    }
}
