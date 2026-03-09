use serde::{Deserialize, Serialize};

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
            notifications_enabled: default_true(),
            auto_check_updates: default_true(),
            update_check_interval_hours: default_update_interval(),
            launch_on_startup: false,
            log_dir: default_log_dir(),
            log_max_file_size_mb: default_log_max_file_size_mb(),
            log_max_rotated_files: default_log_max_rotated_files(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_values() {
        let settings = AppSettings::default();

        eprintln!("theme: {:?}", settings.theme);
        eprintln!("daemon_port: {:?}", settings.daemon_port);

        // Check that defaults are set (actual values from default_* functions)
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.daemon_port, 27015);
        assert_eq!(settings.log_buffer_lines, 5000);
        assert_eq!(settings.config_path, "./tasks.yaml");
        assert!(settings.notifications_enabled);
        assert!(settings.auto_check_updates);
        assert_eq!(settings.update_check_interval_hours, 24);
        assert!(!settings.launch_on_startup);
        assert_eq!(settings.log_dir, "./logs");
        assert_eq!(settings.log_max_file_size_mb, 10);
        assert_eq!(settings.log_max_rotated_files, 5);
    }

    #[test]
    fn test_save_and_load_settings() {
        let temp_dir = std::env::temp_dir();
        let settings_path = temp_dir.join("test_settings_labalaba.yaml");
        let settings_path_str = settings_path.to_string_lossy().to_string();

        // Clean up if exists
        let _ = std::fs::remove_file(&settings_path);

        let original = AppSettings {
            theme: "light".to_string(),
            daemon_port: 8080,
            log_buffer_lines: 10000,
            config_path: "./my_tasks.yaml".to_string(),
            notifications_enabled: false,
            auto_check_updates: false,
            update_check_interval_hours: 48,
            launch_on_startup: true,
            log_dir: "./my_logs".to_string(),
            log_max_file_size_mb: 20,
            log_max_rotated_files: 10,
        };

        // Save
        original.save_to_file(&settings_path_str).unwrap();

        // Load
        let loaded = AppSettings::load_from_file(&settings_path_str).unwrap();

        // Verify
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.daemon_port, 8080);
        assert_eq!(loaded.log_buffer_lines, 10000);
        assert_eq!(loaded.config_path, "./my_tasks.yaml");
        assert!(!loaded.notifications_enabled);
        assert!(!loaded.auto_check_updates);
        assert_eq!(loaded.update_check_interval_hours, 48);
        assert!(loaded.launch_on_startup);
        assert_eq!(loaded.log_dir, "./my_logs");
        assert_eq!(loaded.log_max_file_size_mb, 20);
        assert_eq!(loaded.log_max_rotated_files, 10);

        // Cleanup
        let _ = std::fs::remove_file(&settings_path);
    }

    #[test]
    fn test_load_nonexistent_file_returns_defaults() {
        let settings = AppSettings::load_from_file("/nonexistent/path/settings.yaml").unwrap();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.daemon_port, 27015);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let yaml = serde_yaml::to_string(&settings).unwrap();

        assert!(yaml.contains("theme: dark"));
        assert!(yaml.contains("daemon_port: 27015"));
        assert!(yaml.contains("log_buffer_lines: 5000"));
    }
}
