use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a task
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Current lifecycle state of a managed task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Stopping,
    Crashed,
}

/// Optional cron-based schedule for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub cron: String,
}

/// Full task configuration as persisted in tasks.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub id: TaskId,
    pub name: String,
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default)]
    pub auto_restart: bool,
    pub schedule: Option<Schedule>,
    #[serde(default)]
    pub startup_delay_ms: u64,
    #[serde(default)]
    pub depends_on: Vec<TaskId>,
}

/// Task config plus live runtime status — sent to GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
}

/// Payload for creating or updating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub name: String,
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default)]
    pub auto_restart: bool,
    pub schedule: Option<Schedule>,
    #[serde(default)]
    pub startup_delay_ms: u64,
    #[serde(default)]
    pub depends_on: Vec<TaskId>,
}

/// Summary counts shown in the top statistics bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub crashed: usize,
}
