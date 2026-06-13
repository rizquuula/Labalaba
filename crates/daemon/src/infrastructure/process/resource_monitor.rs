use sysinfo::{Pid, ProcessExt, System, SystemExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use labalaba_shared::task::TaskId;
use std::collections::{HashMap, HashSet};

pub struct ResourceMonitor {
    system: RwLock<System>,
    monitored_tasks: RwLock<HashMap<TaskId, u32>>,
    /// Last sampled `(cpu_percent, memory_bytes)` per task, aggregated over the
    /// tracked process **and all its descendants**. Populated only by
    /// `refresh_now` (driven by the background loop); readers consume this
    /// snapshot without ever touching `system`. That separation is deliberate:
    /// CPU usage is a delta between two refreshes, so if every concurrent stats
    /// poll refreshed the shared `System` on its own, polls landing milliseconds
    /// apart would measure over a near-zero window and report 0%. One periodic
    /// refresher keeps the sampling window stable.
    cache: RwLock<HashMap<TaskId, (f32, u64)>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system: RwLock::new(System::new_all()),
            monitored_tasks: RwLock::new(HashMap::new()),
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_task(&self, task_id: TaskId, pid: u32) {
        let mut tasks = self.monitored_tasks.write().await;
        tasks.insert(task_id.clone(), pid);
        tracing::debug!("Registered task {} for resource monitoring (PID {})", task_id, pid);
    }

    pub async fn unregister_task(&self, task_id: &TaskId) {
        let mut tasks = self.monitored_tasks.write().await;
        if tasks.remove(task_id).is_some() {
            self.cache.write().await.remove(task_id);
            tracing::debug!("Unregistered task {} from resource monitoring", task_id);
        }
    }

    /// Read the last sampled usage for a task. Returns `None` only when the task
    /// isn't being monitored; a monitored task with no sample yet (within the
    /// first refresh interval, or whose process just died) reads `(0.0, 0)`.
    pub async fn get_usage(&self, task_id: &TaskId) -> Option<(f32, u64)> {
        if !self.monitored_tasks.read().await.contains_key(task_id) {
            return None;
        }
        Some(self.cache.read().await.get(task_id).copied().unwrap_or((0.0, 0)))
    }

    pub async fn get_all_usage(&self) -> HashMap<TaskId, (f32, u64)> {
        self.cache.read().await.clone()
    }

    /// Refresh process info once and recompute the cached usage for every
    /// monitored task, summing CPU and memory across each tracked process and
    /// its full descendant tree. Public so the background loop and tests can
    /// drive it; CPU values are only meaningful from the second call onward
    /// (the first establishes the baseline the delta is measured against).
    pub async fn refresh_now(&self) {
        let targets: Vec<(TaskId, u32)> = self
            .monitored_tasks
            .read()
            .await
            .iter()
            .map(|(id, pid)| (id.clone(), *pid))
            .collect();

        let snapshot = {
            let mut sys = self.system.write().await;
            sys.refresh_processes();
            targets
                .into_iter()
                .map(|(id, pid)| (id, aggregate_subtree(&sys, pid)))
                .collect()
        };

        *self.cache.write().await = snapshot;
    }

    pub async fn start_background_refresh(self: Arc<Self>, interval_secs: u64) {
        let mut interval = interval(Duration::from_secs(interval_secs));
        tracing::info!("Resource monitor background refresh started ({}s interval)", interval_secs);

        loop {
            interval.tick().await;
            self.refresh_now().await;
        }
    }
}

/// Sum `(cpu_usage, memory)` over `root_pid` and every process descended from
/// it. Walking the whole tree is what makes stats correct for tasks whose
/// tracked PID is an idle launcher — a `cmd`/shell wrapper, a `runner_prefix`
/// like `uv run`/`python -m`, or a Windows elevation stub — while the real
/// workload runs in child processes the old single-PID read ignored (reporting
/// 0% CPU). An unknown/dead root simply contributes nothing.
fn aggregate_subtree(sys: &System, root_pid: u32) -> (f32, u64) {
    let root = Pid::from(root_pid as usize);
    let procs = sys.processes();

    // Build parent -> children adjacency once, then collect the subtree.
    let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
    for (pid, process) in procs {
        if let Some(parent) = process.parent() {
            children.entry(parent).or_default().push(*pid);
        }
    }

    let mut subtree: HashSet<Pid> = HashSet::new();
    subtree.insert(root);
    let mut stack = vec![root];
    while let Some(pid) = stack.pop() {
        if let Some(kids) = children.get(&pid) {
            for &kid in kids {
                // The `insert` guard also breaks any pid-reuse cycle.
                if subtree.insert(kid) {
                    stack.push(kid);
                }
            }
        }
    }

    let mut cpu = 0.0f32;
    let mut mem = 0u64;
    for pid in subtree {
        if let Some(process) = procs.get(&pid) {
            cpu += process.cpu_usage();
            mem += process.memory();
        }
    }
    (cpu, mem)
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ResourceMonitor {
    /// Test-only: the CPU usage of a single PID from the last refresh, ignoring
    /// descendants. This is what the old implementation reported; the tree-aware
    /// path must beat it whenever a child is doing the work.
    async fn single_pid_cpu(&self, pid: u32) -> f32 {
        let sys = self.system.read().await;
        sys.process(Pid::from(pid as usize))
            .map(|p| p.cpu_usage())
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spawn a child that pegs a CPU core, register the *current* process as the
    /// task root (it stays near-idle), then assert the tree-aggregated CPU
    /// exceeds the root-only CPU by the child's share. The differential is the
    /// point: the old single-PID read returned only the idle root's ~0%, so any
    /// margin here can come only from walking into the child. The burner is a
    /// direct child killed right after sampling — no orphan, no timing tricks,
    /// and no fragile nested shell quoting.
    #[tokio::test]
    async fn aggregates_cpu_across_descendant_processes() {
        use std::process::Command;

        let mut burner = if cfg!(windows) {
            Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", "while(1){}"])
                .spawn()
                .expect("spawn burner")
        } else {
            Command::new("sh")
                .args(["-c", "while :; do :; done"])
                .spawn()
                .expect("spawn burner")
        };

        let monitor = ResourceMonitor::new();
        let task_id = TaskId::new();
        let root_pid = std::process::id();
        monitor.register_task(task_id.clone(), root_pid).await;

        // CPU is a delta between two refreshes, so the child has to be spinning
        // across both. Give it a moment to start, then bracket a burn window.
        tokio::time::sleep(Duration::from_millis(800)).await;
        monitor.refresh_now().await; // baseline
        tokio::time::sleep(Duration::from_millis(1200)).await;
        monitor.refresh_now().await; // measurement

        let (cpu, mem) = monitor
            .get_usage(&task_id)
            .await
            .expect("a monitored task always yields a sample");
        let root_only = monitor.single_pid_cpu(root_pid).await;

        // Kill before asserting so a failure can never leak the burner.
        let _ = burner.kill();
        let _ = burner.wait();

        assert!(
            cpu > root_only + 1.0,
            "tree-aggregated CPU ({cpu}%) should exceed root-only CPU ({root_only}%) \
             by the busy child's share"
        );
        assert!(mem > 0, "aggregated memory should be non-zero, got {mem} bytes");
    }

    #[tokio::test]
    async fn unregistered_task_has_no_usage() {
        let monitor = ResourceMonitor::new();
        assert!(monitor.get_usage(&TaskId::new()).await.is_none());
    }

    #[tokio::test]
    async fn unregister_clears_cached_sample() {
        let monitor = ResourceMonitor::new();
        let task_id = TaskId::new();
        // Track the current test process (definitely alive) and sample it.
        monitor.register_task(task_id.clone(), std::process::id()).await;
        monitor.refresh_now().await;
        assert!(monitor.get_usage(&task_id).await.is_some());

        monitor.unregister_task(&task_id).await;
        assert!(monitor.get_usage(&task_id).await.is_none());
        assert!(monitor.get_all_usage().await.is_empty());
    }
}
