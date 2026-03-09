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
    pub pids: Vec<u32>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_task() -> Task {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "value".to_string());

        Task {
            id: TaskId::new(),
            description: "Test Task".to_string(),
            executable: "node".to_string(),
            arguments: vec!["app.js".to_string()],
            working_directory: Some("/app".to_string()),
            environment: env,
            run_as_admin: false,
            auto_restart: true,
            schedule: None,
            startup_delay_ms: 1000,
            depends_on: vec![],
            pids: vec![],
        }
    }

    #[test]
    fn test_valid_task_creation() {
        let task = create_valid_task();
        assert!(task.validate().is_ok());
        assert_eq!(task.description, "Test Task");
        assert_eq!(task.executable, "node");
        assert!(task.auto_restart);
    }

    #[test]
    fn test_validation_fails_on_empty_description() {
        let mut task = create_valid_task();
        task.description = "   ".to_string();

        let result = task.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TaskValidationError::EmptyDescription => (),
            _ => panic!("Expected EmptyDescription error"),
        }
    }

    #[test]
    fn test_validation_fails_on_empty_executable() {
        let mut task = create_valid_task();
        task.executable = "".to_string();

        let result = task.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TaskValidationError::EmptyExecutable => (),
            _ => panic!("Expected EmptyExecutable error"),
        }
    }

    #[test]
    fn test_validation_fails_on_whitespace_only_description() {
        let mut task = create_valid_task();
        task.description = "   \t\n  ".to_string();

        let result = task.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TaskValidationError::EmptyDescription
        ));
    }

    #[test]
    fn test_task_with_optional_fields() {
        let mut env = HashMap::new();
        env.insert("ENV1".to_string(), "val1".to_string());
        env.insert("ENV2".to_string(), "val2".to_string());

        let task = Task {
            id: TaskId::new(),
            description: "Task with options".to_string(),
            executable: "python".to_string(),
            arguments: vec!["script.py".to_string(), "--verbose".to_string()],
            working_directory: Some("/home/user/project".to_string()),
            environment: env,
            run_as_admin: true,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 5000,
            depends_on: vec![TaskId::new(), TaskId::new()],
            pids: vec![1234, 5678],
        };

        assert!(task.validate().is_ok());
        assert_eq!(task.arguments.len(), 2);
        assert_eq!(task.environment.len(), 2);
        assert_eq!(task.depends_on.len(), 2);
        assert_eq!(task.pids.len(), 2);
    }

    #[test]
    fn test_task_with_minimum_fields() {
        let task = Task {
            id: TaskId::new(),
            description: "Minimal task".to_string(),
            executable: "bash".to_string(),
            arguments: vec![],
            working_directory: None,
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            pids: vec![],
        };

        assert!(task.validate().is_ok());
        assert!(task.working_directory.is_none());
        assert!(task.environment.is_empty());
        assert_eq!(task.startup_delay_ms, 0);
    }
}
