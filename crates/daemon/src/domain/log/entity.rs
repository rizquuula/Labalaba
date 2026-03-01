use labalaba_shared::task::TaskId;
use labalaba_shared::api::{LogEntry, LogStream};
use tokio::sync::broadcast;

/// Maximum log lines buffered per task in memory
pub const DEFAULT_LOG_BUFFER: usize = 5000;

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
