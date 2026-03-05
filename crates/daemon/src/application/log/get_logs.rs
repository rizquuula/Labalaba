use std::sync::Arc;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use labalaba_shared::task::TaskId;
use labalaba_shared::api::{LogEntry, LogStream};
use crate::infrastructure::state::AppState;

pub struct GetLogs {
    pub state: Arc<AppState>,
}

impl GetLogs {
    /// Get recent log lines from task log files (including rotated files)
    pub async fn execute(&self, task_id: &TaskId, limit: usize) -> anyhow::Result<Vec<LogEntry>> {
        let settings = self.state.settings.read().await;
        let log_dir = PathBuf::from(&settings.log_dir);
        drop(settings);

        let mut all_lines: Vec<(String, String, String)> = Vec::new();

        for i in 0..=5 {
            let log_path = if i == 0 {
                log_dir.join(format!("{}.log", task_id))
            } else {
                log_dir.join(format!("{}.log.{}", task_id, i))
            };

            if log_path.exists() {
                let file = File::open(&log_path).await?;
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(entry) = self.parse_log_line(task_id, &line) {
                        all_lines.push((
                            entry.task_id.clone(),
                            entry.timestamp,
                            format!("{}:{}", match entry.stream {
                                LogStream::Stdout => "stdout",
                                LogStream::Stderr => "stderr",
                            }, entry.line),
                        ));
                    }
                }
            }
        }

        let total_lines = all_lines.len();
        let start_idx = total_lines.saturating_sub(limit);

        let result: Vec<LogEntry> = all_lines
            .into_iter()
            .skip(start_idx)
            .map(|(task_id, timestamp, line)| {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                let stream_str = parts.first().unwrap_or(&"stdout");
                let line_content = parts.get(1).unwrap_or(&parts[0]).to_string();
                
                LogEntry {
                    task_id: task_id.clone(),
                    timestamp,
                    stream: if *stream_str == "stderr" {
                        LogStream::Stderr
                    } else {
                        LogStream::Stdout
                    },
                    line: line_content,
                }
            })
            .collect();

        Ok(result)
    }

    fn parse_log_line(&self, task_id: &TaskId, line: &str) -> Option<LogEntry> {
        let line = line.trim();
        if !line.starts_with('[') {
            return None;
        }

        let rest = &line[1..];
        let timestamp_end = rest.find(']')?;
        let timestamp = rest[..timestamp_end].to_string();
        let rest = &rest[timestamp_end + 1..];

        let stream_start = rest.find('[')?;
        let stream_end = rest.find(']')?;
        let stream_str = &rest[stream_start + 1..stream_end];
        let stream = if stream_str == "stderr" {
            LogStream::Stderr
        } else {
            LogStream::Stdout
        };

        let line_content = rest[stream_end + 1..].to_string();

        Some(LogEntry {
            task_id: task_id.to_string(),
            timestamp,
            stream,
            line: line_content,
        })
    }
}
