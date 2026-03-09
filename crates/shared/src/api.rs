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

/// Re-export AppSettings from settings module
pub use crate::settings::AppSettings;

/// Single log line sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub task_id: String,
    pub timestamp: String,
    pub stream: LogStream,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_ok() {
        let response = ApiResponse::ok("test data");

        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_err() {
        let response: ApiResponse<()> = ApiResponse::err("something went wrong");

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            task_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            stream: LogStream::Stdout,
            line: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("task_id"));
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_log_stream_serialization() {
        assert_eq!(
            serde_json::to_string(&LogStream::Stdout).unwrap(),
            "\"stdout\""
        );
        assert_eq!(
            serde_json::to_string(&LogStream::Stderr).unwrap(),
            "\"stderr\""
        );
    }

    #[test]
    fn test_log_stream_deserialization() {
        assert_eq!(
            serde_json::from_str::<LogStream>("\"stdout\"").unwrap(),
            LogStream::Stdout
        );
        assert_eq!(
            serde_json::from_str::<LogStream>("\"stderr\"").unwrap(),
            LogStream::Stderr
        );
    }

    #[test]
    fn test_update_info_serialization() {
        let info = UpdateInfo {
            available: true,
            current_version: "1.0.0".to_string(),
            latest_version: Some("2.0.0".to_string()),
            release_url: Some("https://github.com/...".to_string()),
            release_notes: Some("Bug fixes".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"available\":true"));
        assert!(json.contains("1.0.0"));
        assert!(json.contains("2.0.0"));
    }

    #[test]
    fn test_update_info_no_update() {
        let info = UpdateInfo {
            available: false,
            current_version: "1.0.0".to_string(),
            latest_version: None,
            release_url: None,
            release_notes: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"available\":false"));
        assert!(json.contains("latest_version")); // Will be "latest_version":null
    }
}
