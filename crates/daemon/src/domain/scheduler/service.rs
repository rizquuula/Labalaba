use async_trait::async_trait;
use labalaba_shared::task::TaskId;

/// Port: manages scheduled task execution triggers
#[allow(dead_code)]
#[async_trait]
pub trait SchedulerService: Send + Sync {
    /// Register a cron schedule for a task
    async fn schedule(&self, task_id: TaskId, cron_expr: &str) -> anyhow::Result<()>;
    /// Remove a task's schedule
    async fn unschedule(&self, task_id: &TaskId) -> anyhow::Result<()>;
}
