use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use labalaba_shared::task::TaskId;
use crate::domain::scheduler::schedule::ValidatedSchedule;
use crate::domain::scheduler::service::SchedulerService;
use crate::infrastructure::state::AppState;
use crate::application::task::start_task::StartTask;

pub struct CronScheduler {
    handles: RwLock<HashMap<TaskId, JoinHandle<()>>>,
    state: Arc<AppState>,
}

impl CronScheduler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { handles: RwLock::new(HashMap::new()), state }
    }
}

#[async_trait]
impl SchedulerService for CronScheduler {
    async fn schedule(&self, task_id: TaskId, cron_expr: &str) -> anyhow::Result<()> {
        let schedule = ValidatedSchedule::parse(cron_expr)?;
        let state = Arc::clone(&self.state);
        let id = task_id.clone();

        let handle = tokio::spawn(async move {
            loop {
                let Some(next) = schedule.next_run() else { break };
                let delay = (next - chrono::Utc::now()).to_std()
                    .unwrap_or(std::time::Duration::from_secs(1));
                tokio::time::sleep(delay).await;
                let uc = StartTask { state: Arc::clone(&state) };
                if let Err(e) = uc.execute(id.clone()).await {
                    tracing::warn!("Scheduled start of {} failed: {}", id, e);
                }
            }
        });

        self.handles.write().await.insert(task_id, handle);
        Ok(())
    }

    async fn unschedule(&self, task_id: &TaskId) -> anyhow::Result<()> {
        if let Some(h) = self.handles.write().await.remove(task_id) {
            h.abort();
        }
        Ok(())
    }
}
