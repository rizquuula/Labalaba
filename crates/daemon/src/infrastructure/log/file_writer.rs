use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;
use labalaba_shared::task::TaskId;
use labalaba_shared::api::LogEntry;

/// Flush the buffer once this many lines have accumulated since the last flush.
const FLUSH_EVERY_LINES: u64 = 50;
/// Flush the buffer once this long has elapsed since the last flush.
const FLUSH_EVERY: Duration = Duration::from_millis(500);

/// Runtime-tunable log config. Held behind a `Mutex` on the writer so a settings
/// update can change rotation behaviour without restarting the daemon. `log_dir`
/// changes apply to writers opened *after* the change (already-open files keep
/// their path); `max_file_size_mb` / `max_rotated_files` take effect on the next
/// write/rotation.
#[derive(Clone)]
struct LogConfig {
    log_dir: PathBuf,
    max_file_size_mb: usize,
    max_rotated_files: usize,
}

pub struct LogFileWriter {
    config: Arc<Mutex<LogConfig>>,
    /// Map of per-task writer handles. The outer map lock is held only to
    /// fetch/insert a handle; the actual write/flush happens under the
    /// per-task lock, so tasks no longer serialize against each other.
    writers: Arc<Mutex<std::collections::HashMap<TaskId, Arc<Mutex<WriterHandle>>>>>,
}

struct WriterHandle {
    writer: BufWriter<File>,
    current_size: u64,
    /// Lines buffered since the last flush.
    pending_lines: u64,
    /// When the buffer was last flushed.
    last_flush: Instant,
}

impl WriterHandle {
    fn new(writer: BufWriter<File>, current_size: u64) -> Self {
        Self {
            writer,
            current_size,
            pending_lines: 0,
            last_flush: Instant::now(),
        }
    }

    /// Flush iff the policy threshold (line count or elapsed time) is met.
    async fn maybe_flush(&mut self) -> anyhow::Result<()> {
        if self.pending_lines >= FLUSH_EVERY_LINES || self.last_flush.elapsed() >= FLUSH_EVERY {
            self.writer.flush().await?;
            self.pending_lines = 0;
            self.last_flush = Instant::now();
        }
        Ok(())
    }
}

impl LogFileWriter {
    pub fn new(log_dir: PathBuf, max_file_size_mb: usize, max_rotated_files: usize) -> Self {
        Self {
            config: Arc::new(Mutex::new(LogConfig {
                log_dir,
                max_file_size_mb,
                max_rotated_files,
            })),
            writers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Apply updated log settings at runtime (called from the settings update
    /// path). `max_file_size_mb` and `max_rotated_files` take effect on the next
    /// write/rotation; `log_dir` applies only to writers opened after this call.
    /// `log_dir` is resolved/absolute as supplied by the caller.
    pub async fn update_config(
        &self,
        log_dir: PathBuf,
        max_file_size_mb: usize,
        max_rotated_files: usize,
    ) {
        let mut cfg = self.config.lock().await;
        cfg.log_dir = log_dir;
        cfg.max_file_size_mb = max_file_size_mb;
        cfg.max_rotated_files = max_rotated_files;
    }

    async fn log_dir(&self) -> PathBuf {
        self.config.lock().await.log_dir.clone()
    }

    pub async fn init_dir(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.log_dir().await).await?;
        Ok(())
    }

    /// Fetch the per-task handle, opening (and inserting) it on first use.
    /// Only the outer map lock is held here, briefly.
    async fn handle_for(&self, task_id: &TaskId) -> anyhow::Result<Arc<Mutex<WriterHandle>>> {
        {
            let writers = self.writers.lock().await;
            if let Some(handle) = writers.get(task_id) {
                return Ok(Arc::clone(handle));
            }
        }

        let handle = self.open_handle(task_id).await?;
        let handle = Arc::new(Mutex::new(handle));

        let mut writers = self.writers.lock().await;
        // Another caller may have inserted while we were opening; prefer theirs.
        let entry = writers
            .entry(task_id.clone())
            .or_insert_with(|| Arc::clone(&handle));
        Ok(Arc::clone(entry))
    }

    /// Open the log file for a task and build a fresh handle (no map mutation).
    async fn open_handle(&self, task_id: &TaskId) -> anyhow::Result<WriterHandle> {
        let log_dir = self.log_dir().await;
        let log_path = log_dir.join(format!("{}.log", task_id));
        fs::create_dir_all(&log_dir).await?;

        let metadata = fs::metadata(&log_path).await.ok();
        let current_size = metadata.map(|m| m.len()).unwrap_or(0);

        let writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;

        Ok(WriterHandle::new(BufWriter::new(writer), current_size))
    }

    /// Eagerly open a task's writer (preserves the previous public API). The
    /// handle is opened fresh, replacing any existing one.
    pub async fn open(&self, task_id: &TaskId) -> anyhow::Result<()> {
        let handle = self.open_handle(task_id).await?;
        let mut writers = self.writers.lock().await;
        writers.insert(task_id.clone(), Arc::new(Mutex::new(handle)));
        Ok(())
    }

    pub async fn write(&self, task_id: &TaskId, entry: &LogEntry) -> anyhow::Result<()> {
        let line = format!("[{}] [{}] {}\n", entry.timestamp, match entry.stream {
            labalaba_shared::api::LogStream::Stdout => "stdout",
            labalaba_shared::api::LogStream::Stderr => "stderr",
        }, entry.line);
        let line_bytes = line.as_bytes();
        let line_len = line_bytes.len() as u64;

        let max_file_size_mb = self.config.lock().await.max_file_size_mb;

        let handle = self.handle_for(task_id).await?;

        // Decide whether a rotation is needed under the per-task lock, but
        // perform the rotation outside it (rotate() reopens the file and swaps
        // the writer in), then re-fetch the (possibly new) handle to write.
        let needs_rotate = {
            let h = handle.lock().await;
            h.current_size + line_len > (max_file_size_mb as u64 * 1024 * 1024)
        };

        if needs_rotate {
            // Flush the soon-to-be-rotated writer first. tokio's BufWriter does
            // NOT flush on drop, so without this any buffered-but-unflushed
            // lines (held back by the flush policy) would be lost the moment the
            // live file is renamed and the handle replaced.
            {
                let mut h = handle.lock().await;
                let _ = h.writer.flush().await;
                let _ = h.writer.shutdown().await;
            }
            self.rotate(task_id).await?;
            self.open(task_id).await?;
            let handle = self.handle_for(task_id).await?;
            let mut h = handle.lock().await;
            h.writer.write_all(line_bytes).await?;
            h.current_size += line_len;
            h.pending_lines += 1;
            h.maybe_flush().await?;
            return Ok(());
        }

        let mut h = handle.lock().await;
        h.writer.write_all(line_bytes).await?;
        h.current_size += line_len;
        h.pending_lines += 1;
        h.maybe_flush().await?;
        Ok(())
    }

    async fn rotate(&self, task_id: &TaskId) -> anyhow::Result<()> {
        let (log_dir, max_rotated_files) = {
            let cfg = self.config.lock().await;
            (cfg.log_dir.clone(), cfg.max_rotated_files)
        };
        let log_path = log_dir.join(format!("{}.log", task_id));

        if !log_path.exists() {
            return Ok(());
        }

        for i in (1..max_rotated_files).rev() {
            let old_path = log_dir.join(format!("{}.log.{}", task_id, i));
            let new_path = log_dir.join(format!("{}.log.{}", task_id, i + 1));

            if old_path.exists() {
                if i == max_rotated_files - 1 {
                    fs::remove_file(old_path).await.ok();
                } else {
                    fs::rename(old_path, new_path).await.ok();
                }
            }
        }

        let rotated_path = log_dir.join(format!("{}.log.1", task_id));
        fs::rename(&log_path, &rotated_path).await.ok();

        Ok(())
    }

    /// Remove a task's writer from the map and flush whatever is buffered.
    /// Called by the exit watcher on task exit, so the durability of the
    /// final/error lines is guaranteed even though `write` flushes on a policy.
    pub async fn close(&self, task_id: &TaskId) {
        let handle = {
            let mut writers = self.writers.lock().await;
            writers.remove(task_id)
        };
        if let Some(handle) = handle {
            let mut h = handle.lock().await;
            let _ = h.writer.flush().await;
        }
    }

    /// Close any open writer for a task, then best-effort delete its live log
    /// file and all rotated siblings ({id}.log, {id}.log.1..=max). Used when a
    /// task is deleted so its log files do not linger as orphans. Per-file
    /// removal errors are logged, not fatal.
    pub async fn delete_task_logs(&self, task_id: &TaskId) {
        self.close(task_id).await;

        let (log_dir, max_rotated_files) = {
            let cfg = self.config.lock().await;
            (cfg.log_dir.clone(), cfg.max_rotated_files)
        };
        let live = log_dir.join(format!("{}.log", task_id));
        let mut paths = vec![live];
        for i in 1..=max_rotated_files {
            paths.push(log_dir.join(format!("{}.log.{}", task_id, i)));
        }

        for path in paths {
            if path.exists() {
                if let Err(e) = fs::remove_file(&path).await {
                    tracing::warn!("Failed to remove log file {}: {}", path.display(), e);
                }
            }
        }
    }

    /// Flush and close every open writer. Used on app shutdown so buffered log
    /// lines are not lost when the process exits. Best-effort: per-writer flush
    /// errors are ignored.
    pub async fn close_all(&self) {
        let handles: Vec<Arc<Mutex<WriterHandle>>> = {
            let mut writers = self.writers.lock().await;
            writers.drain().map(|(_, h)| h).collect()
        };
        for handle in handles {
            let mut h = handle.lock().await;
            let _ = h.writer.flush().await;
        }
    }
}

impl Clone for LogFileWriter {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            writers: Arc::clone(&self.writers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use labalaba_shared::api::LogStream;

    fn entry(line: &str) -> LogEntry {
        LogEntry {
            task_id: String::new(),
            timestamp: "t".to_string(),
            stream: LogStream::Stdout,
            line: line.to_string(),
        }
    }

    async fn read_log(dir: &std::path::Path, id: &TaskId) -> String {
        tokio::fs::read_to_string(dir.join(format!("{}.log", id)))
            .await
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn close_flushes_buffered_lines_below_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let w = LogFileWriter::new(dir.path().to_path_buf(), 10, 5);
        w.init_dir().await.unwrap();

        // A handful of lines — below FLUSH_EVERY_LINES — may sit in the buffer.
        for i in 0..3 {
            w.write(&id, &entry(&format!("line{}", i))).await.unwrap();
        }

        // close() must flush whatever is buffered so it reaches disk.
        w.close(&id).await;

        let contents = read_log(dir.path(), &id).await;
        assert!(contents.contains("line0"));
        assert!(contents.contains("line1"));
        assert!(contents.contains("line2"));
    }

    #[tokio::test]
    async fn close_all_flushes_buffered_lines() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let w = LogFileWriter::new(dir.path().to_path_buf(), 10, 5);
        w.init_dir().await.unwrap();

        w.write(&id, &entry("survivor")).await.unwrap();
        w.close_all().await;

        let contents = read_log(dir.path(), &id).await;
        assert!(contents.contains("survivor"));
    }

    #[tokio::test]
    async fn flush_policy_persists_after_many_lines() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let w = LogFileWriter::new(dir.path().to_path_buf(), 10, 5);
        w.init_dir().await.unwrap();

        // Exceed FLUSH_EVERY_LINES so an automatic flush happens mid-stream,
        // without any explicit close.
        for i in 0..(FLUSH_EVERY_LINES + 5) {
            w.write(&id, &entry(&format!("l{}", i))).await.unwrap();
        }

        let contents = read_log(dir.path(), &id).await;
        assert!(contents.contains("l0"));
        assert!(contents.contains(&format!("l{}", FLUSH_EVERY_LINES - 1)));
    }

    #[tokio::test]
    async fn rotation_flushes_buffered_lines_into_rotated_file() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        // 1 MB cap; a handful of small lines stay below the flush threshold so
        // they sit buffered (unflushed) until rotation is forced.
        let w = LogFileWriter::new(dir.path().to_path_buf(), 1, 5);
        w.init_dir().await.unwrap();

        // A few lines below FLUSH_EVERY_LINES — buffered, not yet on disk.
        for i in 0..3 {
            w.write(&id, &entry(&format!("buffered{}", i))).await.unwrap();
        }

        // Force a rotation by writing a line larger than the 1 MB cap. The old
        // (buffered) writer must be flushed before its file is renamed, or the
        // three buffered lines would be lost.
        let big = "x".repeat(1024 * 1024 + 1);
        w.write(&id, &entry(&big)).await.unwrap();

        // The previously-buffered lines must now live in the rotated file.
        let rotated = tokio::fs::read_to_string(dir.path().join(format!("{}.log.1", id)))
            .await
            .unwrap();
        assert!(rotated.contains("buffered0"));
        assert!(rotated.contains("buffered1"));
        assert!(rotated.contains("buffered2"));
    }

    #[tokio::test]
    async fn update_config_applies_max_file_size_to_subsequent_writes() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        // Start with a large cap so nothing rotates.
        let w = LogFileWriter::new(dir.path().to_path_buf(), 1024, 5);
        w.init_dir().await.unwrap();

        w.write(&id, &entry("before")).await.unwrap();
        assert!(!dir.path().join(format!("{}.log.1", id)).exists());

        // Shrink the cap at runtime; the next write must trigger a rotation.
        w.update_config(dir.path().to_path_buf(), 1, 5).await;
        let big = "y".repeat(1024 * 1024 + 1);
        w.write(&id, &entry(&big)).await.unwrap();

        let rotated = tokio::fs::read_to_string(dir.path().join(format!("{}.log.1", id)))
            .await
            .unwrap();
        assert!(rotated.contains("before"));
    }

    #[tokio::test]
    async fn update_config_log_dir_applies_to_new_writers() {
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let w = LogFileWriter::new(dir_a.path().to_path_buf(), 1024, 5);
        w.init_dir().await.unwrap();

        // Repoint at dir_b before opening this task's writer.
        w.update_config(dir_b.path().to_path_buf(), 1024, 5).await;
        w.write(&id, &entry("inb")).await.unwrap();
        w.close(&id).await;

        let in_b = read_log(dir_b.path(), &id).await;
        assert!(in_b.contains("inb"));
        assert!(!dir_a.path().join(format!("{}.log", id)).exists());
    }

    #[tokio::test]
    async fn delete_task_logs_removes_live_and_rotated_files() {
        let dir = tempfile::tempdir().unwrap();
        let id = TaskId::new();
        let w = LogFileWriter::new(dir.path().to_path_buf(), 10, 5);
        w.init_dir().await.unwrap();

        // Live file via the writer, plus a couple of rotated siblings by hand.
        w.write(&id, &entry("x")).await.unwrap();
        tokio::fs::write(dir.path().join(format!("{}.log.1", id)), b"old").await.unwrap();
        tokio::fs::write(dir.path().join(format!("{}.log.2", id)), b"older").await.unwrap();

        w.delete_task_logs(&id).await;

        assert!(!dir.path().join(format!("{}.log", id)).exists());
        assert!(!dir.path().join(format!("{}.log.1", id)).exists());
        assert!(!dir.path().join(format!("{}.log.2", id)).exists());
    }
}
