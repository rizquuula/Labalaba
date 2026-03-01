use std::collections::HashMap;
use labalaba_shared::task::{Schedule, TaskId};

/// Core task domain entity — pure business logic, no I/O
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
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
    pub fn new(
        id: TaskId,
        name: impl Into<String>,
        executable: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            executable: executable.into(),
            arguments: Vec::new(),
            working_directory: None,
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: Vec::new(),
        }
    }

    /// Validate the task configuration before persisting or spawning
    pub fn validate(&self) -> Result<(), TaskValidationError> {
        if self.name.trim().is_empty() {
            return Err(TaskValidationError::EmptyName);
        }
        if self.executable.trim().is_empty() {
            return Err(TaskValidationError::EmptyExecutable);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskValidationError {
    #[error("Task name must not be empty")]
    EmptyName,
    #[error("Executable path must not be empty")]
    EmptyExecutable,
}
