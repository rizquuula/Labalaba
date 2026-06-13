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
    /// Get a page of a task's log lines, newest-anchored. Returns the `limit`
    /// lines that sit immediately *older* than the newest `offset` lines —
    /// `offset = 0` yields the most recent `limit` lines (the common case),
    /// while increasing `offset` walks backwards through history for a
    /// "load older" pager. Files (live `.log` plus rotated siblings) are
    /// scanned newest-first and only as many older files as needed are opened.
    /// Returned in chronological order.
    pub async fn execute(
        &self,
        task_id: &TaskId,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<LogEntry>> {
        let (log_dir, max_rotated) = {
            let settings = self.state.settings.read().await;
            (
                PathBuf::from(&settings.log_dir),
                settings.log_max_rotated_files,
            )
        };

        read_recent_lines(&log_dir, task_id, limit, offset, max_rotated).await
    }
}

/// Return the `limit` log lines immediately older than the newest `offset`
/// lines for `task_id` in `log_dir`. We collect the newest `offset + limit`
/// lines (live `.log` then `.log.1`, `.log.2`, … as needed) and drop the
/// newest `offset`; the remainder (at most `limit`) is the requested page, in
/// chronological order. Memory stays O(offset + limit).
async fn read_recent_lines(
    log_dir: &Path,
    task_id: &TaskId,
    limit: usize,
    offset: usize,
    max_rotated: usize,
) -> anyhow::Result<Vec<LogEntry>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    // We need the newest `offset + limit` lines, then drop the newest `offset`.
    let target = offset.saturating_add(limit);
    // Cap only the preallocation hint so a huge `offset` can't ask for a giant
    // up-front allocation; the deque still grows to `target` if the data exists.
    let cap_hint = target.min(4096);

    // Scan up to the configured rotated-file count (mirroring the writer), but
    // never beyond a sane cap.
    let max_rotated_scan = max_rotated.min(MAX_ROTATED_SCAN_CAP);

    // Newest-first file order: live `.log`, then `.log.1`, `.log.2`, ...
    // Within a single file we keep the last `target` lines in a bounded deque;
    // across files we stop once we have `target` total.
    let mut collected: VecDeque<LogEntry> = VecDeque::with_capacity(cap_hint);

    for i in 0..=max_rotated_scan {
        if collected.len() >= target {
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

        // Read this file's parsed lines, keeping only its newest `target`.
        let file = File::open(&log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut file_tail: VecDeque<LogEntry> = VecDeque::with_capacity(cap_hint);

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(entry) = parse_log_line(task_id, &line) {
                if file_tail.len() == target {
                    file_tail.pop_front();
                }
                file_tail.push_back(entry);
            }
        }

        // This file is older than everything already collected, so prepend its
        // lines (chronologically before the newer ones), capping total.
        let need = target - collected.len();
        let take = file_tail.len().min(need);
        for entry in file_tail.into_iter().rev().take(take) {
            collected.push_front(entry);
        }
    }

    // `collected` is chronological (oldest..newest) and at most `target` long.
    // Drop the newest `offset` lines; the remainder (≤ limit) is the page.
    let keep = collected.len().saturating_sub(offset);
    Ok(collected.into_iter().take(keep).collect())
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

        let out = read_recent_lines(dir.path(), &id, 2, 0, 5).await.unwrap();
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
        let out = read_recent_lines(dir.path(), &id, 5, 0, 5).await.unwrap();
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

        let out = read_recent_lines(dir.path(), &id, 1, 0, 5).await.unwrap();
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
        let out = read_recent_lines(dir.path(), &id, 10, 0, 5).await.unwrap();
        assert!(out.is_empty());

        // With max_rotated = 8 it is reachable.
        let out = read_recent_lines(dir.path(), &id, 10, 0, 8).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, " oldest");
    }

    #[tokio::test]
    async fn limit_zero_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(dir.path(), &format!("{}.log", id), &[&line("t1", "a")]).await;
        let out = read_recent_lines(dir.path(), &id, 0, 0, 5).await.unwrap();
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

        let out = read_recent_lines(dir.path(), &id, 10, 0, 5).await.unwrap();
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].stream, LogStream::Stderr));
        assert_eq!(out[0].line, " error: boom");
    }

    #[tokio::test]
    async fn offset_walks_backwards_through_a_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(
            dir.path(),
            &format!("{}.log", id),
            &[
                &line("t1", "1"),
                &line("t2", "2"),
                &line("t3", "3"),
                &line("t4", "4"),
                &line("t5", "5"),
                &line("t6", "6"),
            ],
        )
        .await;

        // Page size 2, paging from newest backwards.
        let names = |out: Vec<LogEntry>| -> Vec<String> {
            out.into_iter().map(|e| e.line).collect()
        };

        let page0 = read_recent_lines(dir.path(), &id, 2, 0, 5).await.unwrap();
        assert_eq!(names(page0), vec![" 5", " 6"]);

        let page1 = read_recent_lines(dir.path(), &id, 2, 2, 5).await.unwrap();
        assert_eq!(names(page1), vec![" 3", " 4"]);

        let page2 = read_recent_lines(dir.path(), &id, 2, 4, 5).await.unwrap();
        assert_eq!(names(page2), vec![" 1", " 2"]);

        // Beyond the start: nothing older remains.
        let page3 = read_recent_lines(dir.path(), &id, 2, 6, 5).await.unwrap();
        assert!(page3.is_empty());
    }

    #[tokio::test]
    async fn offset_spans_rotated_files() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        // Chronological: .log.1 (oldest) = 1,2 ; .log (newest) = 3,4.
        write_file(dir.path(), &format!("{}.log.1", id), &[&line("t1", "1"), &line("t2", "2")]).await;
        write_file(dir.path(), &format!("{}.log", id), &[&line("t3", "3"), &line("t4", "4")]).await;

        // Skipping the newest 2 (the live file) must reach into the rotated file.
        let older = read_recent_lines(dir.path(), &id, 2, 2, 5).await.unwrap();
        let lines: Vec<&str> = older.iter().map(|e| e.line.as_str()).collect();
        assert_eq!(lines, vec![" 1", " 2"]);
    }

    #[tokio::test]
    async fn offset_partial_page_at_the_start() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        write_file(
            dir.path(),
            &format!("{}.log", id),
            &[&line("t1", "1"), &line("t2", "2"), &line("t3", "3")],
        )
        .await;

        // Only 1 line older than the newest 2 remains; a full page isn't possible.
        let older = read_recent_lines(dir.path(), &id, 5, 2, 5).await.unwrap();
        let lines: Vec<&str> = older.iter().map(|e| e.line.as_str()).collect();
        assert_eq!(lines, vec![" 1"]);
    }
}
