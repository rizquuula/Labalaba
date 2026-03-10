use std::sync::Arc;
use labalaba_shared::task::{TaskId, TaskRequest};
use crate::domain::task::entity::Task;
use crate::domain::task::repository::TaskRepository;

pub struct CreateTask {
    pub repo: Arc<dyn TaskRepository>,
}

impl CreateTask {
    pub async fn execute(&self, req: TaskRequest) -> anyhow::Result<Task> {
        let task = Task {
            id: TaskId::new(),
            description: req.description,
            executable: req.executable,
            arguments: req.arguments,
            working_directory: req.working_directory,
            environment: req.environment,
            run_as_admin: req.run_as_admin,
            auto_restart: req.auto_restart,
            schedule: req.schedule,
            startup_delay_ms: req.startup_delay_ms,
            depends_on: req.depends_on,
            runner_prefix: req.runner_prefix,
            pids: req.pids,
        };
        task.validate()?;
        self.repo.save(&task).await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::entity::TaskValidationError;

    #[tokio::test]
    async fn test_create_task_success() {
        // We'll test the validation logic primarily
        let req = TaskRequest {
            description: "Test Task".to_string(),
            executable: "node".to_string(),
            arguments: vec!["app.js".to_string()],
            working_directory: Some("/app".to_string()),
            environment: std::collections::HashMap::new(),
            run_as_admin: false,
            auto_restart: true,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids: vec![],
        };

        // Test that the task would be created correctly
        let task = Task {
            id: TaskId::new(),
            description: req.description.clone(),
            executable: req.executable.clone(),
            arguments: req.arguments.clone(),
            working_directory: req.working_directory.clone(),
            environment: req.environment.clone(),
            run_as_admin: req.run_as_admin,
            auto_restart: req.auto_restart,
            schedule: req.schedule.clone(),
            startup_delay_ms: req.startup_delay_ms,
            depends_on: req.depends_on.clone(),
            runner_prefix: req.runner_prefix.clone(),
            pids: req.pids.clone(),
        };
        
        assert!(task.validate().is_ok());
        assert_eq!(task.description, "Test Task");
        assert_eq!(task.executable, "node");
        assert!(task.auto_restart);
    }

    #[tokio::test]
    async fn test_create_task_fails_on_empty_description() {
        let req = TaskRequest {
            description: "   ".to_string(),
            executable: "node".to_string(),
            arguments: vec![],
            working_directory: None,
            environment: std::collections::HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids: vec![],
        };

        let task = Task {
            id: TaskId::new(),
            description: req.description.clone(),
            executable: req.executable.clone(),
            arguments: req.arguments.clone(),
            working_directory: req.working_directory.clone(),
            environment: req.environment.clone(),
            run_as_admin: req.run_as_admin,
            auto_restart: req.auto_restart,
            schedule: req.schedule.clone(),
            startup_delay_ms: req.startup_delay_ms,
            depends_on: req.depends_on.clone(),
            runner_prefix: req.runner_prefix.clone(),
            pids: req.pids.clone(),
        };

        let result = task.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TaskValidationError::EmptyDescription
        ));
    }

    #[tokio::test]
    async fn test_create_task_fails_on_empty_executable() {
        let req = TaskRequest {
            description: "Test Task".to_string(),
            executable: "".to_string(),
            arguments: vec![],
            working_directory: None,
            environment: std::collections::HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids: vec![],
        };

        let task = Task {
            id: TaskId::new(),
            description: req.description.clone(),
            executable: req.executable.clone(),
            arguments: req.arguments.clone(),
            working_directory: req.working_directory.clone(),
            environment: req.environment.clone(),
            run_as_admin: req.run_as_admin,
            auto_restart: req.auto_restart,
            schedule: req.schedule.clone(),
            startup_delay_ms: req.startup_delay_ms,
            depends_on: req.depends_on.clone(),
            runner_prefix: req.runner_prefix.clone(),
            pids: req.pids.clone(),
        };

        let result = task.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TaskValidationError::EmptyExecutable
        ));
    }
}
