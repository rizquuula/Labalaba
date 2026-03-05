use std::sync::Arc;
use labalaba_shared::task::TaskId;
use crate::domain::task::repository::TaskRepository;

pub struct DeleteTask {
    pub repo: Arc<dyn TaskRepository>,
}

impl DeleteTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<()> {
        self.repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;
        self.repo.delete(&id).await
    }
}
