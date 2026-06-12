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

    /// Atomically read-modify-write a task's `pids` under the repository lock.
    /// The closure receives the currently-persisted pid list and returns the new
    /// one, so concurrent start (push) and stop (clear) operations cannot clobber
    /// each other by saving a stale copy of the whole task.
    async fn update_pids(
        &self,
        id: &TaskId,
        mutate: Box<dyn FnOnce(Vec<u32>) -> Vec<u32> + Send>,
    ) -> anyhow::Result<()>;
}
