use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl AppSettings {
    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        if std::fs::metadata(path).is_err() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)?;
        let settings: AppSettings = serde_yaml::from_str(&contents)?;
        Ok(settings)
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
