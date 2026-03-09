use crate::domain::task::entity::Task;
use crate::domain::task::status::TaskRuntimeState;
use crate::infrastructure::state::AppState;
use labalaba_shared::task::{TaskConfig, TaskDto};

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
