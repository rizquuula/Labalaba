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
        Ok(serde_yaml::from_str(&contents)?)
    }

    async fn persist(&self, store: &YamlStore) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(store)?;
        tokio::fs::write(&self.path, yaml).await?;
        Ok(())
    }
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
}
