use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub description: String,
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
    #[serde(default)]
    pub runner_prefix: Option<String>,
    #[serde(default)]
    pub pids: Vec<u32>,
}

/// Task config plus live runtime status — sent to GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub pid: Option<u32>,
    pub pids: Vec<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
}

/// Payload for creating or updating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub description: String,
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
    #[serde(default)]
    pub runner_prefix: Option<String>,
    #[serde(default)]
    pub pids: Vec<u32>,
}

/// Summary counts shown in the top statistics bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub crashed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();

        assert_ne!(id1, id2); // UUIDs should be unique
    }

    #[test]
    fn test_task_id_default() {
        let id = TaskId::default();
        assert!(id.0.as_u128() != 0); // Should be a valid UUID
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new();
        let display = id.to_string();

        // UUID format: 8-4-4-4-12 hex characters with hyphens
        assert_eq!(display.len(), 36);
        assert_eq!(display.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn test_task_status_serialization() {
        assert_eq!(
            serde_json::to_string(&TaskStatus::Stopped).unwrap(),
            "\"stopped\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Starting).unwrap(),
            "\"starting\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Stopping).unwrap(),
            "\"stopping\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Crashed).unwrap(),
            "\"crashed\""
        );
    }

    #[test]
    fn test_task_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"stopped\"").unwrap(),
            TaskStatus::Stopped
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"starting\"").unwrap(),
            TaskStatus::Starting
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"running\"").unwrap(),
            TaskStatus::Running
        );
    }

    #[test]
    fn test_task_stats_default() {
        let stats = TaskStats {
            total: 5,
            running: 2,
            stopped: 2,
            crashed: 1,
        };

        assert_eq!(stats.total, 5);
        assert_eq!(stats.running, 2);
        assert_eq!(stats.stopped, 2);
        assert_eq!(stats.crashed, 1);
    }

    #[test]
    fn test_task_stats_serialization() {
        let stats = TaskStats {
            total: 3,
            running: 1,
            stopped: 1,
            crashed: 1,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total\":3"));
        assert!(json.contains("\"running\":1"));
    }
}
