use async_trait::async_trait;
use labalaba_shared::task::TaskId;
use crate::domain::task::entity::Task;

/// Port: defines persistence operations the domain needs.
/// Concrete implementations live in infrastructure/persistence/.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_all(&self) -> anyhow::Result<Vec<Task>>;
    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Option<Task>>;
    async fn save(&self, task: &Task) -> anyhow::Result<()>;
    async fn delete(&self, id: &TaskId) -> anyhow::Result<()>;
}
