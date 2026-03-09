use sysinfo::{ProcessExt, System, SystemExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use labalaba_shared::task::TaskId;
use std::collections::HashMap;

pub struct ResourceMonitor {
    system: RwLock<System>,
    monitored_tasks: RwLock<HashMap<TaskId, u32>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system: RwLock::new(System::new_all()),
            monitored_tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_task(&self, task_id: TaskId, pid: u32) {
        let mut tasks = self.monitored_tasks.write().await;
        tasks.insert(task_id, pid);
        tracing::debug!("Registered task {} for resource monitoring (PID {})", task_id, pid);
    }

    pub async fn unregister_task(&self, task_id: &TaskId) {
        let mut tasks = self.monitored_tasks.write().await;
        if tasks.remove(task_id).is_some() {
            tracing::debug!("Unregistered task {} from resource monitoring", task_id);
        }
    }

    pub async fn get_usage(&self, task_id: &TaskId) -> Option<(f32, u64)> {
        let tasks = self.monitored_tasks.read().await;
        let pid = tasks.get(task_id)?;
        
        let mut sys = self.system.write().await;
        sys.refresh_process((*pid).into());
        
        if let Some(process) = sys.process((*pid).into()) {
            Some((process.cpu_usage(), process.memory()))
        } else {
            None
        }
    }

    pub async fn get_all_usage(&self) -> HashMap<TaskId, (f32, u64)> {
        let tasks = self.monitored_tasks.read().await;
        let mut sys = self.system.write().await;
        sys.refresh_processes();
        
        let mut result = HashMap::new();
        for (task_id, pid) in tasks.iter() {
            if let Some(process) = sys.process((*pid).into()) {
                result.insert(task_id.clone(), (process.cpu_usage(), process.memory()));
            }
        }
        result
    }

    pub async fn start_background_refresh(self: Arc<Self>, interval_secs: u64) {
        let mut interval = interval(Duration::from_secs(interval_secs));
        tracing::info!("Resource monitor background refresh started ({}s interval)", interval_secs);
        
        loop {
            interval.tick().await;
            let mut sys = self.system.write().await;
            sys.refresh_processes();
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
