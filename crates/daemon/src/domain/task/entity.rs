use labalaba_shared::task::{Schedule, TaskId};
use std::collections::HashMap;

/// Core task domain entity — pure business logic, no I/O
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub environment: HashMap<String, String>,
    pub run_as_admin: bool,
    pub auto_restart: bool,
    pub schedule: Option<Schedule>,
    pub startup_delay_ms: u64,
    pub depends_on: Vec<TaskId>,
}

impl Task {
    /// Validate the task configuration before persisting or spawning
    pub fn validate(&self) -> Result<(), TaskValidationError> {
        if self.description.trim().is_empty() {
            return Err(TaskValidationError::EmptyDescription);
        }
        if self.executable.trim().is_empty() {
            return Err(TaskValidationError::EmptyExecutable);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskValidationError {
    #[error("Task description must not be empty")]
    EmptyDescription,
    #[error("Executable path must not be empty")]
    EmptyExecutable,
}
