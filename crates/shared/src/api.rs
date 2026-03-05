use serde::{Deserialize, Serialize};

/// Generic API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// App-wide settings persisted alongside tasks.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_port")]
    pub daemon_port: u16,
    #[serde(default = "default_log_buffer")]
    pub log_buffer_lines: usize,
    #[serde(default = "default_config_path")]
    pub config_path: String,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    #[serde(default = "default_update_interval")]
    pub update_check_interval_hours: u64,
    #[serde(default)]
    pub launch_on_startup: bool,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default = "default_log_max_file_size_mb")]
    pub log_max_file_size_mb: usize,
    #[serde(default = "default_log_max_rotated_files")]
    pub log_max_rotated_files: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            daemon_port: default_port(),
            log_buffer_lines: default_log_buffer(),
            config_path: default_config_path(),
            notifications_enabled: true,
            auto_check_updates: true,
            update_check_interval_hours: default_update_interval(),
            launch_on_startup: false,
            log_dir: default_log_dir(),
            log_max_file_size_mb: default_log_max_file_size_mb(),
            log_max_rotated_files: default_log_max_rotated_files(),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}
fn default_port() -> u16 {
    27015
}
fn default_log_buffer() -> usize {
    5000
}
fn default_config_path() -> String {
    "./tasks.yaml".to_string()
}
fn default_true() -> bool {
    true
}
fn default_update_interval() -> u64 {
    24
}
fn default_log_dir() -> String {
    "./logs".to_string()
}
fn default_log_max_file_size_mb() -> usize {
    10
}
fn default_log_max_rotated_files() -> usize {
    5
}

/// Single log line sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub task_id: String,
    pub timestamp: String,
    pub stream: LogStream,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// Update check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub release_notes: Option<String>,
}
