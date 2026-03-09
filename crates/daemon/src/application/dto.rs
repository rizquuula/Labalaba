use crate::domain::task::entity::Task;
use crate::domain::task::status::TaskRuntimeState;
use crate::infrastructure::state::AppState;
use labalaba_shared::task::{TaskConfig, TaskDto, TaskId};

/// Convert domain Task + runtime state into the DTO sent over HTTP
pub async fn task_to_dto(
    task: &Task, 
    state: &TaskRuntimeState,
    app_state: &AppState,
) -> TaskDto {
    let (cpu_percent, memory_bytes) = if state.pid.is_some() {
        if let Some(usage) = app_state.resource_monitor.get_usage(&task.id).await {
            usage
        } else {
            (0.0, 0)
        }
    } else {
        (0.0, 0)
    };

    TaskDto {
        config: task_to_config(task),
        status: state.status.clone(),
        pid: state.pid,
        pids: task.pids.clone(),
        started_at: state.started_at,
        exit_code: state.exit_code,
        cpu_percent: Some(cpu_percent),
        memory_bytes: Some(memory_bytes),
    }
}

/// Resource stats for a task
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskResourceStats {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

/// Convert domain Task into its config representation
pub fn task_to_config(task: &Task) -> TaskConfig {
    TaskConfig {
        id: task.id.clone(),
        description: task.description.clone(),
        executable: task.executable.clone(),
        arguments: task.arguments.clone(),
        working_directory: task.working_directory.clone(),
        environment: task.environment.clone(),
        run_as_admin: task.run_as_admin,
        auto_restart: task.auto_restart,
        schedule: task.schedule.clone(),
        startup_delay_ms: task.startup_delay_ms,
        depends_on: task.depends_on.clone(),
        pids: task.pids.clone(),
    }
}

/// Convert a TaskConfig (from YAML) into a domain Task
pub fn config_to_task(config: TaskConfig) -> Task {
    Task {
        id: config.id,
        description: config.description,
        executable: config.executable,
        arguments: config.arguments,
        working_directory: config.working_directory,
        environment: config.environment,
        run_as_admin: config.run_as_admin,
        auto_restart: config.auto_restart,
        schedule: config.schedule,
        startup_delay_ms: config.startup_delay_ms,
        depends_on: config.depends_on,
        pids: config.pids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_task() -> Task {
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
            pids: vec![1234],
        }
    }

    #[test]
    fn test_task_to_config_preserves_all_fields() {
        let task = create_test_task();
        let config = task_to_config(&task);
        
        assert_eq!(config.id, task.id);
        assert_eq!(config.description, task.description);
        assert_eq!(config.executable, task.executable);
        assert_eq!(config.arguments, task.arguments);
        assert_eq!(config.working_directory, task.working_directory);
        assert_eq!(config.environment, task.environment);
        assert_eq!(config.run_as_admin, task.run_as_admin);
        assert_eq!(config.auto_restart, task.auto_restart);
        assert_eq!(config.startup_delay_ms, task.startup_delay_ms);
        assert_eq!(config.depends_on, task.depends_on);
        assert_eq!(config.pids, task.pids);
    }

    #[test]
    fn test_config_to_task_preserves_all_fields() {
        let task = create_test_task();
        let config = task_to_config(&task);
        let reconstructed = config_to_task(config);
        
        assert_eq!(reconstructed.id, task.id);
        assert_eq!(reconstructed.description, task.description);
        assert_eq!(reconstructed.executable, task.executable);
        assert_eq!(reconstructed.arguments, task.arguments);
        assert_eq!(reconstructed.working_directory, task.working_directory);
        assert_eq!(reconstructed.environment, task.environment);
        assert_eq!(reconstructed.run_as_admin, task.run_as_admin);
        assert_eq!(reconstructed.auto_restart, task.auto_restart);
        assert_eq!(reconstructed.startup_delay_ms, task.startup_delay_ms);
        assert_eq!(reconstructed.depends_on, task.depends_on);
        assert_eq!(reconstructed.pids, task.pids);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_task = create_test_task();
        let config = task_to_config(&original_task);
        let reconstructed = config_to_task(config);
        
        // All fields should match
        assert_eq!(original_task.id, reconstructed.id);
        assert_eq!(original_task.description, reconstructed.description);
        assert_eq!(original_task.executable, reconstructed.executable);
        assert_eq!(original_task.arguments, reconstructed.arguments);
        assert_eq!(original_task.working_directory, reconstructed.working_directory);
        assert_eq!(original_task.environment, reconstructed.environment);
        assert_eq!(original_task.run_as_admin, reconstructed.run_as_admin);
        assert_eq!(original_task.auto_restart, reconstructed.auto_restart);
        // Skip schedule comparison - doesn't implement PartialEq
        assert_eq!(original_task.startup_delay_ms, reconstructed.startup_delay_ms);
        assert_eq!(original_task.depends_on, reconstructed.depends_on);
        assert_eq!(original_task.pids, reconstructed.pids);
    }

    #[test]
    fn test_config_to_task_with_empty_fields() {
        let config = TaskConfig {
            id: TaskId::new(),
            description: "Minimal".to_string(),
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
        
        let task = config_to_task(config);
        
        assert!(task.arguments.is_empty());
        assert!(task.working_directory.is_none());
        assert!(task.environment.is_empty());
        assert!(!task.run_as_admin);
        assert!(!task.auto_restart);
        assert!(task.schedule.is_none());
        assert_eq!(task.startup_delay_ms, 0);
        assert!(task.depends_on.is_empty());
        assert!(task.pids.is_empty());
    }
}
