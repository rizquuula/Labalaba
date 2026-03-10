use std::sync::Arc;
use labalaba_shared::task::{TaskId, TaskRequest};
use crate::domain::task::entity::Task;
use crate::domain::task::repository::TaskRepository;

pub struct EditTask {
    pub repo: Arc<dyn TaskRepository>,
}

impl EditTask {
    pub async fn execute(&self, id: TaskId, req: TaskRequest) -> anyhow::Result<Task> {
        let existing = self.repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;

        let updated = Task {
            id: existing.id,
            description: req.description,
            executable: req.executable,
            arguments: req.arguments,
            working_directory: req.working_directory,
            environment: req.environment,
            run_as_admin: req.run_as_admin,
            auto_restart: req.auto_restart,
            schedule: req.schedule,
            startup_delay_ms: req.startup_delay_ms,
            depends_on: req.depends_on,
            runner_prefix: req.runner_prefix,
            pids: existing.pids, // Preserve existing PIDs during edit
        };
        updated.validate()?;
        self.repo.save(&updated).await?;
        Ok(updated)
    }
}
