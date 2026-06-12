use std::path::PathBuf;
use async_trait::async_trait;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use labalaba_shared::task::{TaskConfig, TaskId};
use crate::application::dto::{config_to_task, task_to_config};
use crate::domain::task::entity::Task;
use crate::domain::task::repository::TaskRepository;

#[derive(Serialize, Deserialize, Default)]
struct YamlStore {
    #[serde(default)]
    tasks: Vec<TaskConfig>,
}

/// Persists tasks as a YAML file in the working directory.
/// Uses a Mutex to serialize concurrent writes.
pub struct YamlTaskRepository {
    path: PathBuf,
    lock: Mutex<()>,
}

impl YamlTaskRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), lock: Mutex::new(()) }
    }

    async fn load(&self) -> anyhow::Result<YamlStore> {
        if !self.path.exists() {
            return Ok(YamlStore::default());
        }
        let contents = tokio::fs::read_to_string(&self.path).await?;

        // Fast path: a well-formed file parses strictly.
        match serde_yaml::from_str::<YamlStore>(&contents) {
            Ok(store) => Ok(store),
            Err(strict_err) => {
                // Before touching anything, copy the unparseable file aside so the
                // user's data is never silently destroyed.
                let backup = self.path.with_extension("yaml.corrupt-backup");
                if let Err(e) = tokio::fs::copy(&self.path, &backup).await {
                    tracing::warn!(
                        "Failed to back up unparseable tasks file to {}: {}",
                        backup.display(),
                        e
                    );
                }

                // Fall back to parsing as a generic Value and recovering each task
                // entry individually, skipping entries that fail.
                let recovered = recover_tasks(&contents);
                if recovered.is_empty() {
                    // Nothing salvageable: surface the original error, but only after
                    // the backup copy above has been attempted.
                    Err(anyhow::anyhow!(
                        "Failed to parse tasks file {}: {}",
                        self.path.display(),
                        strict_err
                    ))
                } else {
                    tracing::warn!(
                        "Recovered {} task(s) from malformed tasks file {} (original backed up)",
                        recovered.len(),
                        self.path.display()
                    );
                    Ok(YamlStore { tasks: recovered })
                }
            }
        }
    }

    async fn persist(&self, store: &YamlStore) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(store)?;
        // Write to a temp file in the SAME directory, then atomically rename over
        // the target. A crash mid-write leaves the original intact rather than a
        // truncated file. Rename is atomic on the same filesystem on Unix, and on
        // Windows std/tokio rename replaces an existing destination.
        let tmp = self.path.with_extension("yaml.tmp");
        tokio::fs::write(&tmp, yaml).await?;
        tokio::fs::rename(&tmp, &self.path).await?;
        Ok(())
    }
}

/// Recover individual task entries from a malformed tasks file. Parses the YAML
/// as a generic value, then attempts to deserialize each entry under `tasks`
/// independently, skipping (with a warning) any entry that fails.
fn recover_tasks(contents: &str) -> Vec<TaskConfig> {
    let value: serde_yaml::Value = match serde_yaml::from_str(contents) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Could not parse tasks file as YAML during recovery: {}", e);
            return Vec::new();
        }
    };

    let entries = match value.get("tasks").and_then(|t| t.as_sequence()) {
        Some(seq) => seq,
        None => return Vec::new(),
    };

    let mut recovered = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        match serde_yaml::from_value::<TaskConfig>(entry.clone()) {
            Ok(config) => recovered.push(config),
            Err(e) => {
                tracing::warn!("Skipping unrecoverable task entry #{}: {}", idx, e);
            }
        }
    }
    recovered
}

#[async_trait]
impl TaskRepository for YamlTaskRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<Task>> {
        let store = self.load().await?;
        Ok(store.tasks.into_iter().map(config_to_task).collect())
    }

    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Option<Task>> {
        let store = self.load().await?;
        Ok(store.tasks.into_iter()
            .find(|t| &t.id == id)
            .map(config_to_task))
    }

    async fn save(&self, task: &Task) -> anyhow::Result<()> {
        let _guard = self.lock.lock().await;
        let mut store = self.load().await?;
        let config = task_to_config(task);
        if let Some(pos) = store.tasks.iter().position(|t| t.id == task.id) {
            store.tasks[pos] = config;
        } else {
            store.tasks.push(config);
        }
        self.persist(&store).await
    }

    async fn delete(&self, id: &TaskId) -> anyhow::Result<()> {
        let _guard = self.lock.lock().await;
        let mut store = self.load().await?;
        store.tasks.retain(|t| &t.id != id);
        self.persist(&store).await
    }

    async fn update_pids(
        &self,
        id: &TaskId,
        mutate: Box<dyn FnOnce(Vec<u32>) -> Vec<u32> + Send>,
    ) -> anyhow::Result<()> {
        let _guard = self.lock.lock().await;
        let mut store = self.load().await?;
        {
            let config = store
                .tasks
                .iter_mut()
                .find(|t| &t.id == id)
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;
            config.pids = mutate(std::mem::take(&mut config.pids));
        }
        self.persist(&store).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_config(id: TaskId) -> TaskConfig {
        TaskConfig {
            id,
            description: "test".to_string(),
            executable: "/bin/true".to_string(),
            arguments: vec![],
            working_directory: None,
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids: vec![],
        }
    }

    async fn repo_with_one_task() -> (YamlTaskRepository, TaskId) {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        // Drop the file handle but keep the path; the repo creates/writes it.
        drop(file);
        let repo = YamlTaskRepository::new(path);
        let id = TaskId::new();
        let store = YamlStore { tasks: vec![sample_config(id.clone())] };
        repo.persist(&store).await.unwrap();
        (repo, id)
    }

    #[tokio::test]
    async fn test_update_pids_push_then_clear() {
        let (repo, id) = repo_with_one_task().await;

        repo.update_pids(&id, Box::new(|mut pids| { pids.push(42); pids })).await.unwrap();
        let task = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(task.pids, vec![42]);

        repo.update_pids(&id, Box::new(|_pids| Vec::new())).await.unwrap();
        let task = repo.find_by_id(&id).await.unwrap().unwrap();
        assert!(task.pids.is_empty());
    }

    #[tokio::test]
    async fn test_update_pids_preserves_other_fields() {
        let (repo, id) = repo_with_one_task().await;
        repo.update_pids(&id, Box::new(|mut pids| { pids.push(7); pids })).await.unwrap();
        let task = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(task.executable, "/bin/true");
        assert_eq!(task.pids, vec![7]);
    }

    #[tokio::test]
    async fn test_update_pids_unknown_task_errors() {
        let (repo, _id) = repo_with_one_task().await;
        let err = repo
            .update_pids(&TaskId::new(), Box::new(|mut pids| { pids.push(1); pids }))
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_persist_is_atomic_no_temp_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.yaml");
        let repo = YamlTaskRepository::new(path.clone());
        let id = TaskId::new();
        let store = YamlStore { tasks: vec![sample_config(id.clone())] };

        repo.persist(&store).await.unwrap();

        // Target exists and parses back; temp file is cleaned up by the rename.
        assert!(path.exists());
        assert!(!path.with_extension("yaml.tmp").exists());
        let loaded = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(loaded.id, id);
    }

    #[tokio::test]
    async fn test_load_recovers_well_formed_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.yaml");
        let repo = YamlTaskRepository::new(path);
        let store = YamlStore {
            tasks: vec![sample_config(TaskId::new()), sample_config(TaskId::new())],
        };
        repo.persist(&store).await.unwrap();

        let loaded = repo.find_all().await.unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_load_skips_one_bad_entry_and_backs_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.yaml");
        // One valid entry, one entry missing the required `executable` field.
        let yaml = "tasks:\n  - id: 11111111-1111-1111-1111-111111111111\n    description: good\n    executable: /bin/true\n  - id: 22222222-2222-2222-2222-222222222222\n    description: bad\n";
        tokio::fs::write(&path, yaml).await.unwrap();

        let repo = YamlTaskRepository::new(path.clone());
        let loaded = repo.find_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].executable, "/bin/true");

        // The original (unparseable-strictly) file was backed up before recovery.
        assert!(path.with_extension("yaml.corrupt-backup").exists());
    }

    #[tokio::test]
    async fn test_load_garbage_file_errors_after_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.yaml");
        tokio::fs::write(&path, ":\n::not yaml at all: [\n").await.unwrap();

        let repo = YamlTaskRepository::new(path.clone());
        let result = repo.find_all().await;
        assert!(result.is_err());

        // Even on hard failure, the original is preserved as a backup.
        assert!(path.with_extension("yaml.corrupt-backup").exists());
    }
}
