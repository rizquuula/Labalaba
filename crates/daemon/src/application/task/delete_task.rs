use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::domain::task::repository::TaskRepository;
use crate::infrastructure::log::file_writer::LogFileWriter;

pub struct DeleteTask {
    pub repo: Arc<dyn TaskRepository>,
    pub log_writer: LogFileWriter,
}

impl DeleteTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<()> {
        self.repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;
        self.repo.delete(&id).await?;

        // Best-effort: close any open writer and remove the task's log files so
        // they do not linger as orphans. Failures here are logged, not fatal.
        self.log_writer.delete_task_logs(&id).await;

        Ok(())
    }
}
