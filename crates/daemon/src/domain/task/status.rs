use chrono::{DateTime, Utc};
use labalaba_shared::task::TaskStatus;

/// Max consecutive rapid crash-restarts before a task is left Crashed.
pub const MAX_CONSECUTIVE_RESTARTS: u32 = 5;
/// Base auto-restart backoff delay in seconds (doubles each consecutive failure).
pub const BASE_RESTART_DELAY_SECS: u64 = 3;
/// Cap on the exponential backoff delay in seconds.
pub const MAX_RESTART_DELAY_SECS: u64 = 60;
/// A process must survive at least this long for the restart counter to reset.
pub const RESTART_RESET_AFTER_SECS: i64 = 30;

/// Runtime state tracked in memory for a running task
#[derive(Debug, Clone, Default)]
pub struct TaskRuntimeState {
    pub status: TaskStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    /// Consecutive rapid auto-restart attempts (reset once a run survives long enough).
    pub consecutive_restarts: u32,
    /// When the last auto-restart was queued (for backoff diagnostics).
    pub last_restart: Option<DateTime<Utc>>,
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

    /// Atomically claim the Starting state if the task is not already running.
    /// Returns true if the transition was made (caller should proceed to spawn),
    /// false if the task was already Starting/Running (caller should abort).
    pub fn mark_starting_if_stopped(&mut self) -> bool {
        if self.is_running() {
            return false;
        }
        self.mark_starting();
        true
    }

    /// True if the task is being deliberately stopped, so an exiting process
    /// must be treated as intentional rather than a crash.
    pub fn is_stopping_or_stopped(&self) -> bool {
        matches!(self.status, TaskStatus::Stopping | TaskStatus::Stopped)
    }

    /// True once consecutive rapid restarts have hit the cap.
    pub fn restart_cap_reached(&self) -> bool {
        self.consecutive_restarts >= MAX_CONSECUTIVE_RESTARTS
    }

    /// Record a crash-driven restart attempt. If the previous run survived
    /// longer than RESTART_RESET_AFTER_SECS the counter resets first.
    /// Returns the backoff delay (seconds) to wait before restarting.
    pub fn register_restart_attempt(&mut self) -> u64 {
        let survived_long_enough = self
            .started_at
            .map(|s| (Utc::now() - s).num_seconds() >= RESTART_RESET_AFTER_SECS)
            .unwrap_or(false);
        if survived_long_enough {
            self.consecutive_restarts = 0;
        }

        let delay = BASE_RESTART_DELAY_SECS
            .saturating_mul(1u64 << self.consecutive_restarts.min(63))
            .min(MAX_RESTART_DELAY_SECS);

        self.consecutive_restarts = self.consecutive_restarts.saturating_add(1);
        self.last_restart = Some(Utc::now());
        delay
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

    #[test]
    fn test_mark_starting_if_stopped_claims_when_stopped() {
        let mut state = TaskRuntimeState::default();
        assert!(state.mark_starting_if_stopped());
        assert_eq!(state.status, TaskStatus::Starting);
    }

    #[test]
    fn test_mark_starting_if_stopped_rejects_when_running() {
        let mut state = TaskRuntimeState::default();
        state.mark_running(123);
        assert!(!state.mark_starting_if_stopped());
        assert_eq!(state.status, TaskStatus::Running);
    }

    #[test]
    fn test_mark_starting_if_stopped_rejects_when_already_starting() {
        let mut state = TaskRuntimeState::default();
        state.mark_starting();
        // A second concurrent start must NOT win the claim.
        assert!(!state.mark_starting_if_stopped());
    }

    #[test]
    fn test_intentional_stop_detection() {
        let mut state = TaskRuntimeState::default();
        state.mark_running(123);
        assert!(!state.is_stopping_or_stopped());

        state.mark_stopping();
        assert!(state.is_stopping_or_stopped());

        state.mark_stopped(None);
        assert!(state.is_stopping_or_stopped());

        state.mark_crashed(Some(1));
        assert!(!state.is_stopping_or_stopped());
    }

    #[test]
    fn test_backoff_is_exponential_and_capped() {
        let mut state = TaskRuntimeState::default();
        // started_at stays None so the survival reset never fires.
        let mut delays = Vec::new();
        for _ in 0..MAX_CONSECUTIVE_RESTARTS {
            delays.push(state.register_restart_attempt());
        }
        // 3, 6, 12, 24, 48 — doubling from the base, all below the 60s cap.
        assert_eq!(delays, vec![3, 6, 12, 24, 48]);
        assert!(delays.iter().all(|&d| d <= MAX_RESTART_DELAY_SECS));
    }

    #[test]
    fn test_backoff_caps_at_max() {
        let mut state = TaskRuntimeState::default();
        // Force the counter high enough that the raw doubling would exceed 60.
        state.consecutive_restarts = 10;
        assert_eq!(state.register_restart_attempt(), MAX_RESTART_DELAY_SECS);
    }

    #[test]
    fn test_restart_cap_reached_after_max_attempts() {
        let mut state = TaskRuntimeState::default();
        assert!(!state.restart_cap_reached());
        for _ in 0..MAX_CONSECUTIVE_RESTARTS {
            state.register_restart_attempt();
        }
        assert!(state.restart_cap_reached());
    }

    #[test]
    fn test_counter_resets_after_long_survival() {
        let mut state = TaskRuntimeState::default();
        state.consecutive_restarts = 3;
        // Simulate a run that survived well past the reset threshold.
        state.started_at =
            Some(Utc::now() - chrono::Duration::seconds(RESTART_RESET_AFTER_SECS + 5));
        // The reset happens first, so the returned delay is the base again.
        assert_eq!(state.register_restart_attempt(), BASE_RESTART_DELAY_SECS);
        // And after this attempt the counter is at 1, not 4.
        assert_eq!(state.consecutive_restarts, 1);
    }
}
