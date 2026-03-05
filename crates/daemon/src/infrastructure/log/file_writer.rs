use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;
use labalaba_shared::task::TaskId;
use labalaba_shared::api::LogEntry;

pub struct LogFileWriter {
    log_dir: PathBuf,
    max_file_size_mb: usize,
    max_rotated_files: usize,
    writers: Arc<Mutex<std::collections::HashMap<TaskId, WriterHandle>>>,
}

struct WriterHandle {
    writer: BufWriter<File>,
    current_size: u64,
}

impl LogFileWriter {
    pub fn new(log_dir: PathBuf, max_file_size_mb: usize, max_rotated_files: usize) -> Self {
        Self {
            log_dir,
            max_file_size_mb,
            max_rotated_files,
            writers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn init_dir(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.log_dir).await?;
        Ok(())
    }

    pub async fn open(&self, task_id: &TaskId) -> anyhow::Result<()> {
        let mut writers = self.writers.lock().await;
        
        let log_path = self.log_dir.join(format!("{}.log", task_id));
        fs::create_dir_all(&self.log_dir).await?;
        
        let metadata = fs::metadata(&log_path).await.ok();
        let current_size = metadata.map(|m| m.len()).unwrap_or(0);
        
        let writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;
        
        let buf_writer = BufWriter::new(writer);
        
        writers.insert(
            task_id.clone(),
            WriterHandle {
                writer: buf_writer,
                current_size,
            },
        );
        
        Ok(())
    }

    pub async fn write(&self, task_id: &TaskId, entry: &LogEntry) -> anyhow::Result<()> {
        let mut writers = self.writers.lock().await;
        
        if let Some(handle) = writers.get_mut(task_id) {
            let line = format!("[{}] [{}] {}\n", entry.timestamp, match entry.stream {
                labalaba_shared::api::LogStream::Stdout => "stdout",
                labalaba_shared::api::LogStream::Stderr => "stderr",
            }, entry.line);
            
            let line_bytes = line.as_bytes();
            let line_len = line_bytes.len() as u64;
            
            handle.current_size += line_len;
            
            if handle.current_size > (self.max_file_size_mb as u64 * 1024 * 1024) {
                drop(writers);
                self.rotate(task_id).await?;
                self.open(task_id).await?;
                writers = self.writers.lock().await;
            }
            
            if let Some(handle) = writers.get_mut(task_id) {
                handle.writer.write_all(line_bytes).await?;
                handle.writer.flush().await?;
            }
        }
        
        Ok(())
    }

    async fn rotate(&self, task_id: &TaskId) -> anyhow::Result<()> {
        let log_path = self.log_dir.join(format!("{}.log", task_id));
        
        if !log_path.exists() {
            return Ok(());
        }
        
        for i in (1..self.max_rotated_files).rev() {
            let old_path = self.log_dir.join(format!("{}.log.{}", task_id, i));
            let new_path = self.log_dir.join(format!("{}.log.{}", task_id, i + 1));
            
            if old_path.exists() {
                if i == self.max_rotated_files - 1 {
                    fs::remove_file(old_path).await.ok();
                } else {
                    fs::rename(old_path, new_path).await.ok();
                }
            }
        }
        
        let rotated_path = self.log_dir.join(format!("{}.log.1", task_id));
        fs::rename(&log_path, &rotated_path).await.ok();
        
        let mut writers = self.writers.lock().await;
        let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .await?;
        
        if let Some(handle) = writers.get_mut(task_id) {
            handle.writer = BufWriter::new(writer);
            handle.current_size = 0;
        }
        
        Ok(())
    }

    pub async fn close(&self, task_id: &TaskId) {
        let mut writers = self.writers.lock().await;
        if let Some(handle) = writers.remove(task_id) {
            let _ = handle.writer.into_inner().flush().await;
        }
    }
}

impl Drop for LogFileWriter {
    fn drop(&mut self) {
        // Cleanup not needed here since we use Arc<Mutex<HashMap>>
    }
}

impl Clone for LogFileWriter {
    fn clone(&self) -> Self {
        Self {
            log_dir: self.log_dir.clone(),
            max_file_size_mb: self.max_file_size_mb,
            max_rotated_files: self.max_rotated_files,
            writers: Arc::clone(&self.writers),
        }
    }
}
