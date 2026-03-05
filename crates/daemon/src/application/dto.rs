use crate::domain::task::entity::Task;
use crate::domain::task::status::TaskRuntimeState;
use labalaba_shared::task::{TaskConfig, TaskDto};

/// Convert domain Task + runtime state into the DTO sent over HTTP
pub fn task_to_dto(task: &Task, state: &TaskRuntimeState) -> TaskDto {
    TaskDto {
        config: task_to_config(task),
        status: state.status.clone(),
        pid: state.pid,
        pids: task.pids.clone(),
        started_at: state.started_at,
        exit_code: state.exit_code,
    }
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
