use labalaba_shared::task::TaskId;
use labalaba_shared::api::{LogEntry, LogStream};
use tokio::sync::broadcast;

/// A broadcast channel for streaming log lines to WebSocket subscribers
pub type LogBroadcaster = broadcast::Sender<LogEntry>;
pub type LogReceiver = broadcast::Receiver<LogEntry>;

/// Creates a new log broadcast channel for a task
pub fn new_log_channel() -> LogBroadcaster {
    broadcast::channel(1024).0
}

/// Constructs a LogEntry from raw output
pub fn make_log_entry(task_id: &TaskId, stream: LogStream, line: String) -> LogEntry {
    LogEntry {
        task_id: task_id.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        stream,
        line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_log_channel_creation() {
        let broadcaster = new_log_channel();
        // Just verify we can create a channel and subscribe to it
        let _receiver = broadcaster.subscribe();
    }

    #[test]
    fn test_make_log_entry_stdout() {
        let task_id = TaskId::new();
        let line = "Hello, world!".to_string();
        let entry = make_log_entry(&task_id, LogStream::Stdout, line.clone());
        
        assert_eq!(entry.task_id, task_id.to_string());
        assert!(matches!(entry.stream, LogStream::Stdout));
        assert_eq!(entry.line, line);
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn test_make_log_entry_stderr() {
        let task_id = TaskId::new();
        let line = "Error message".to_string();
        let entry = make_log_entry(&task_id, LogStream::Stderr, line.clone());
        
        assert_eq!(entry.task_id, task_id.to_string());
        assert!(matches!(entry.stream, LogStream::Stderr));
        assert_eq!(entry.line, line);
    }

    #[test]
    fn test_log_entry_timestamp_format() {
        let task_id = TaskId::new();
        let entry = make_log_entry(&task_id, LogStream::Stdout, "test".to_string());
        
        // RFC3339 timestamp should contain T separator
        assert!(entry.timestamp.contains('T'));
        // Should parse as valid RFC3339
        assert!(chrono::DateTime::parse_from_rfc3339(&entry.timestamp).is_ok());
    }

    #[tokio::test]
    async fn test_log_broadcaster_send_receive() {
        let broadcaster = new_log_channel();
        let mut receiver = broadcaster.subscribe();
        
        let task_id = TaskId::new();
        let entry = make_log_entry(&task_id, LogStream::Stdout, "test log".to_string());
        
        // Send the entry
        let _ = broadcaster.send(entry.clone());
        
        // Receive it
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.task_id, entry.task_id);
        assert_eq!(received.line, entry.line);
        assert!(matches!(received.stream, LogStream::Stdout));
    }

    #[tokio::test]
    async fn test_log_broadcaster_multiple_subscribers() {
        let broadcaster = new_log_channel();
        let mut receiver1 = broadcaster.subscribe();
        let mut receiver2 = broadcaster.subscribe();
        
        let task_id = TaskId::new();
        let entry = make_log_entry(&task_id, LogStream::Stderr, "broadcast".to_string());
        
        // Send once
        let _ = broadcaster.send(entry.clone());
        
        // Both should receive
        let recv1 = receiver1.recv().await.unwrap();
        let recv2 = receiver2.recv().await.unwrap();
        
        assert_eq!(recv1.line, recv2.line);
        assert_eq!(recv1.task_id, recv2.task_id);
    }
}
