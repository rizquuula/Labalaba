use std::sync::Arc;
use labalaba_shared::task::{TaskId, TaskRequest};
use crate::domain::task::entity::Task;
use crate::domain::task::repository::TaskRepository;

pub struct CreateTask {
    pub repo: Arc<dyn TaskRepository>,
}

impl CreateTask {
    pub async fn execute(&self, req: TaskRequest) -> anyhow::Result<Task> {
        let task = Task {
            id: TaskId::new(),
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
            pids: req.pids,
        };
        task.validate()?;
        self.repo.save(&task).await?;
        Ok(task)
    }
}
