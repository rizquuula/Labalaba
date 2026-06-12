use std::collections::VecDeque;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use labalaba_shared::task::TaskId;
use labalaba_shared::api::{LogEntry, LogStream};
use crate::infrastructure::state::AppState;

/// Upper bound on how many rotated files (besides the live `.log`) we ever scan,
/// even if `log_max_rotated_files` is configured higher. Keeps a misconfigured
/// setting from forcing an unbounded number of file opens per request.
const MAX_ROTATED_SCAN_CAP: usize = 100;

pub struct GetLogs {
    pub state: Arc<AppState>,
}

impl GetLogs {
    /// Get the most recent `limit` log lines from a task's log files (live file
    /// plus rotated siblings). Files are scanned newest-first and only as many
    /// older files as needed are opened, so memory stays O(limit) rather than
    /// loading every rotated file fully. Returned in chronological order.
    pub async fn execute(&self, task_id: &TaskId, limit: usize) -> anyhow::Result<Vec<LogEntry>> {
        let (log_dir, max_rotated) = {
            let settings = self.state.settings.read().await;
            (
                PathBuf::from(&settings.log_dir),
                settings.log_max_rotated_files,
            )
        };

        read_recent_lines(&log_dir, task_id, limit, max_rotated).await
    }
}

/// Get the most recent `limit` log lines from a task's log files (live file
/// plus rotated siblings) in `log_dir`. Files are scanned newest-first and only
/// as many older files as needed are opened, so memory stays O(limit) rather
/// than loading every rotated file fully. Returned in chronological order.
async fn read_recent_lines(
    log_dir: &Path,
    task_id: &TaskId,
    limit: usize,
    max_rotated: usize,
) -> anyhow::Result<Vec<LogEntry>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    // Scan up to the configured rotated-file count (mirroring the writer), but
    // never beyond a sane cap.
    let max_rotated_scan = max_rotated.min(MAX_ROTATED_SCAN_CAP);

    // Newest-first file order: live `.log`, then `.log.1`, `.log.2`, ...
    // Within a single file we keep the last `limit` lines in a bounded deque;
    // across files we stop once we have `limit` total.
    let mut collected: VecDeque<LogEntry> = VecDeque::with_capacity(limit);

    for i in 0..=max_rotated_scan {
        if collected.len() >= limit {
            break;
        }

        let log_path = if i == 0 {
            log_dir.join(format!("{}.log", task_id))
        } else {
            log_dir.join(format!("{}.log.{}", task_id, i))
        };

        if !log_path.exists() {
            continue;
        }

        // Read this file's parsed lines, keeping only its newest `limit`.
        let file = File::open(&log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut file_tail: VecDeque<LogEntry> = VecDeque::with_capacity(limit);

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(entry) = parse_log_line(task_id, &line) {
                if file_tail.len() == limit {
                    file_tail.pop_front();
                }
                file_tail.push_back(entry);
            }
        }

        // This file is older than everything already collected, so prepend its
        // lines (chronologically before the newer ones), capping total.
        let need = limit - collected.len();
        let take = file_tail.len().min(need);
        for entry in file_tail.into_iter().rev().take(take) {
            collected.push_front(entry);
        }
    }

    Ok(collected.into_iter().collect())
}

fn parse_log_line(task_id: &TaskId, line: &str) -> Option<LogEntry> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    async fn write_file(dir: &Path, name: &str, lines: &[&str]) {
        let path = dir.join(name);
        let mut f = tokio::fs::File::create(&path).await.unwrap();
        for l in lines {
            f.write_all(format!("{}\n", l).as_bytes()).await.unwrap();
        }
        f.flush().await.unwrap();
    }

    fn line(ts: &str, content: &str) -> String {
        format!("[{}] [stdout] {}", ts, content)
    }

    #[tokio::test]
    async fn reads_only_live_file_when_enough() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(
            dir.path(),
            &format!("{}.log", id),
            &[&line("t1", "a"), &line("t2", "b"), &line("t3", "c")],
        )
        .await;

        let out = read_recent_lines(dir.path(), &id, 2, 5).await.unwrap();
        assert_eq!(out.len(), 2);
        // Newest two, chronological. The parser preserves the leading space
        // after the stream marker (existing format — kept intentionally).
        assert_eq!(out[0].line, " b");
        assert_eq!(out[1].line, " c");
    }

    #[tokio::test]
    async fn spans_rotated_files_newest_first_chronological() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        // .log.2 is oldest, .log.1 next, .log is live (newest).
        write_file(dir.path(), &format!("{}.log.2", id), &[&line("t1", "1"), &line("t2", "2")]).await;
        write_file(dir.path(), &format!("{}.log.1", id), &[&line("t3", "3"), &line("t4", "4")]).await;
        write_file(dir.path(), &format!("{}.log", id), &[&line("t5", "5"), &line("t6", "6")]).await;

        // Ask for 5 — should pull all of live (6,5), all of .log.1 (4,3), and
        // the newest one from .log.2 (2), returned chronologically.
        let out = read_recent_lines(dir.path(), &id, 5, 5).await.unwrap();
        let lines: Vec<&str> = out.iter().map(|e| e.line.as_str()).collect();
        assert_eq!(lines, vec![" 2", " 3", " 4", " 5", " 6"]);
    }

    #[tokio::test]
    async fn does_not_open_older_files_when_satisfied_early() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(dir.path(), &format!("{}.log", id), &[&line("t5", "5"), &line("t6", "6")]).await;
        // An older rotated file exists but should not be needed for limit=1.
        write_file(dir.path(), &format!("{}.log.1", id), &[&line("t1", "old")]).await;

        let out = read_recent_lines(dir.path(), &id, 1, 5).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, " 6");
    }

    #[tokio::test]
    async fn honors_configured_rotated_count_beyond_legacy_five() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        // The oldest line lives in .log.7 — unreachable under the old hardcoded
        // scan of 5, reachable once we honor the configured rotated count.
        write_file(dir.path(), &format!("{}.log.7", id), &[&line("t0", "oldest")]).await;

        // With max_rotated = 5 the oldest file is NOT scanned.
        let out = read_recent_lines(dir.path(), &id, 10, 5).await.unwrap();
        assert!(out.is_empty());

        // With max_rotated = 8 it is reachable.
        let out = read_recent_lines(dir.path(), &id, 10, 8).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, " oldest");
    }

    #[tokio::test]
    async fn limit_zero_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(dir.path(), &format!("{}.log", id), &[&line("t1", "a")]).await;
        let out = read_recent_lines(dir.path(), &id, 0, 5).await.unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn preserves_stream_and_content_with_colons() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let p = dir.path().join(format!("{}.log", id));
        let mut f = tokio::fs::File::create(&p).await.unwrap();
        f.write_all(b"[t1] [stderr] error: boom\n").await.unwrap();
        f.flush().await.unwrap();

        let out = read_recent_lines(dir.path(), &id, 10, 5).await.unwrap();
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].stream, LogStream::Stderr));
        assert_eq!(out[0].line, " error: boom");
    }
}
