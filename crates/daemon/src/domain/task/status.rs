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

#[cfg(test)]
mod tests {
    use super::*;
    use labalaba_shared::task::TaskStatus;

    #[test]
    fn test_default_state_is_stopped() {
        let state = TaskRuntimeState::default();
        assert_eq!(state.status, TaskStatus::Stopped);
        assert!(state.pid.is_none());
        assert!(state.started_at.is_none());
        assert!(state.exit_code.is_none());
    }

    #[test]
    fn test_state_transition_starting() {
        let mut state = TaskRuntimeState::default();
        state.mark_starting();

        assert_eq!(state.status, TaskStatus::Starting);
        assert!(state.pid.is_none());
        assert!(state.started_at.is_none());
        assert!(state.exit_code.is_none());
    }

    #[test]
    fn test_state_transition_running() {
        let mut state = TaskRuntimeState::default();
        let pid = 12345u32;
        state.mark_running(pid);

        assert_eq!(state.status, TaskStatus::Running);
        assert_eq!(state.pid, Some(pid));
        assert!(state.started_at.is_some());
        assert!(state.exit_code.is_none());
    }

    #[test]
    fn test_state_transition_stopping() {
        let mut state = TaskRuntimeState::default();
        state.mark_starting();
        state.mark_running(12345);
        state.mark_stopping();

        assert_eq!(state.status, TaskStatus::Stopping);
    }

    #[test]
    fn test_state_transition_stopped() {
        let mut state = TaskRuntimeState::default();
        state.mark_stopped(Some(0));

        assert_eq!(state.status, TaskStatus::Stopped);
        assert!(state.pid.is_none());
        assert_eq!(state.exit_code, Some(0));
    }

    #[test]
    fn test_state_transition_stopped_with_exit_code() {
        let mut state = TaskRuntimeState::default();
        state.mark_stopped(Some(42));

        assert_eq!(state.status, TaskStatus::Stopped);
        assert_eq!(state.exit_code, Some(42));
    }

    #[test]
    fn test_state_transition_stopped_without_exit_code() {
        let mut state = TaskRuntimeState::default();
        state.mark_stopped(None);

        assert_eq!(state.status, TaskStatus::Stopped);
        assert!(state.exit_code.is_none());
    }

    #[test]
    fn test_state_transition_crashed() {
        let mut state = TaskRuntimeState::default();
        state.mark_crashed(Some(1));

        assert_eq!(state.status, TaskStatus::Crashed);
        assert!(state.pid.is_none());
        assert_eq!(state.exit_code, Some(1));
    }

    #[test]
    fn test_state_transition_crashed_no_exit_code() {
        let mut state = TaskRuntimeState::default();
        state.mark_crashed(None);

        assert_eq!(state.status, TaskStatus::Crashed);
        assert!(state.exit_code.is_none());
    }

    #[test]
    fn test_is_running_returns_true_for_starting() {
        let mut state = TaskRuntimeState::default();
        state.mark_starting();
        assert!(state.is_running());
    }

    #[test]
    fn test_is_running_returns_true_for_running() {
        let mut state = TaskRuntimeState::default();
        state.mark_running(12345);
        assert!(state.is_running());
    }

    #[test]
    fn test_is_running_returns_false_for_stopped() {
        let mut state = TaskRuntimeState::default();
        state.mark_stopped(Some(0));
        assert!(!state.is_running());
    }

    #[test]
    fn test_is_running_returns_false_for_crashed() {
        let mut state = TaskRuntimeState::default();
        state.mark_crashed(Some(1));
        assert!(!state.is_running());
    }

    #[test]
    fn test_full_lifecycle() {
        let mut state = TaskRuntimeState::default();

        // Initial state
        assert!(!state.is_running());

        // Start
        state.mark_starting();
        assert!(state.is_running());

        // Running
        state.mark_running(12345);
        assert!(state.is_running());
        assert!(state.pid.is_some());

        // Stop
        state.mark_stopping();
        assert!(!state.is_running());

        state.mark_stopped(Some(0));
        assert!(!state.is_running());
        assert!(state.pid.is_none());
        assert_eq!(state.exit_code, Some(0));
    }

    #[test]
    fn test_crash_resets_pid() {
        let mut state = TaskRuntimeState::default();
        state.mark_running(12345);
        assert!(state.pid.is_some());

        state.mark_crashed(Some(1));
        assert!(state.pid.is_none());
        assert_eq!(state.exit_code, Some(1));
    }
}
