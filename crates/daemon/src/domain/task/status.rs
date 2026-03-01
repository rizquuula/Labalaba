use chrono::{DateTime, Utc};
use labalaba_shared::task::TaskStatus;

/// Runtime state tracked in memory for a running task
#[derive(Debug, Clone, Default)]
pub struct TaskRuntimeState {
    pub status: TaskStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
}

impl TaskRuntimeState {
    pub fn mark_starting(&mut self) {
        self.status = TaskStatus::Starting;
        self.pid = None;
        self.started_at = None;
        self.exit_code = None;
    }

    pub fn mark_running(&mut self, pid: u32) {
        self.status = TaskStatus::Running;
        self.pid = Some(pid);
        self.started_at = Some(Utc::now());
        self.exit_code = None;
    }

    pub fn mark_stopping(&mut self) {
        self.status = TaskStatus::Stopping;
    }

    pub fn mark_stopped(&mut self, exit_code: Option<i32>) {
        self.status = TaskStatus::Stopped;
        self.pid = None;
        self.exit_code = exit_code;
    }

    pub fn mark_crashed(&mut self, exit_code: Option<i32>) {
        self.status = TaskStatus::Crashed;
        self.pid = None;
        self.exit_code = exit_code;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, TaskStatus::Running | TaskStatus::Starting)
    }
}
