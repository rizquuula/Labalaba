use async_trait::async_trait;
use crate::domain::process::entity::ProcessHandle;
use crate::domain::task::entity::Task;

/// Port: defines how the domain spawns and signals OS processes.
/// Concrete impl lives in infrastructure/process/.
#[async_trait]
pub trait ProcessSpawner: Send + Sync {
    /// Spawn the task executable and return a handle to the live process
    async fn spawn(&self, task: &Task) -> anyhow::Result<ProcessHandle>;

    /// Send a termination signal to the process
    async fn kill(&self, pid: u32) -> anyhow::Result<()>;
}
