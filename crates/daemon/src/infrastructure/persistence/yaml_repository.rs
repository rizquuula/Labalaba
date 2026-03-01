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
}
