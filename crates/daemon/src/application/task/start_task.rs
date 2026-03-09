use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use labalaba_shared::task::TaskId;
use labalaba_shared::api::LogStream;
use crate::domain::log::entity::{make_log_entry, new_log_channel};
use crate::infrastructure::state::AppState;

pub struct StartTask {
    pub state: Arc<AppState>,
}

impl StartTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<u32> {
        let task = self.state.task_repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;

        {
            let states = self.state.runtime_states.read().await;
            if let Some(s) = states.get(&id) {
                if s.is_running() {
                    anyhow::bail!("Task {} is already running", id);
                }
            }
        }

        if task.startup_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(task.startup_delay_ms)).await;
        }

        {
            let mut states = self.state.runtime_states.write().await;
            states.entry(id.clone()).or_default().mark_starting();
        }

        let mut handle = self.state.spawner.spawn(&task).await.inspect_err(|_| {
            let id_clone = id.clone();
            let state_clone = Arc::clone(&self.state);
            tokio::spawn(async move {
                let mut states = state_clone.runtime_states.write().await;
                states.entry(id_clone).or_default().mark_crashed(None);
            });
        })?;

        let pid = handle.pid;

        {
            let mut states = self.state.runtime_states.write().await;
            states.entry(id.clone()).or_default().mark_running(pid);
        }

        // Register PID to task and persist
        let task = {
            let mut task = task;
            task.pids.push(pid);
            self.state.task_repo.save(&task).await?;
            task
        };

        // Register with resource monitor
        self.state.resource_monitor.register_task(id.clone(), pid).await;

        // Ensure a log channel exists for this task
        let broadcaster = {
            let mut logs = self.state.log_channels.write().await;
            logs.entry(id.clone()).or_insert_with(new_log_channel).clone()
        };

        // Open log file writer for this task
        let log_writer = self.state.log_writer.clone();
        log_writer.open(&id).await?;

        let log_cb = self.state.log_event_callback.clone();

        // Spawn stdout collector
        if let Some(stdout) = handle.child.stdout.take() {
            let tx = broadcaster.clone();
            let id_out = id.clone();
            let writer = log_writer.clone();
            let cb = log_cb.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let entry = make_log_entry(&id_out, LogStream::Stdout, line);
                    let _ = tx.send(entry.clone());
                    let _ = writer.write(&id_out, &entry).await;
                    if let Some(ref cb) = cb { cb(entry); }
                }
            });
        }

        // Spawn stderr collector
        if let Some(stderr) = handle.child.stderr.take() {
            let tx = broadcaster.clone();
            let id_err = id.clone();
            let writer = log_writer.clone();
            let cb = log_cb.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let entry = make_log_entry(&id_err, LogStream::Stderr, line);
                    let _ = tx.send(entry.clone());
                    let _ = writer.write(&id_err, &entry).await;
                    if let Some(ref cb) = cb { cb(entry); }
                }
            });
        }

        // Spawn exit watcher — sends on restart_tx instead of recursing
        let state_clone = Arc::clone(&self.state);
        let auto_restart = task.auto_restart;
        let restart_tx = self.state.restart_tx.clone();
        let log_writer = self.state.log_writer.clone();
        let id_clone = id.clone();
        tokio::spawn(async move {
            let exit_status = handle.child.wait().await.ok();
            let exit_code = exit_status.and_then(|s| s.code());
            let crashed = exit_code.map(|c| c != 0).unwrap_or(true);

            {
                let mut states = state_clone.runtime_states.write().await;
                let rt = states.entry(id_clone.clone()).or_default();
                if crashed {
                    rt.mark_crashed(exit_code);
                } else {
                    rt.mark_stopped(exit_code);
                }
            }

            let _ = log_writer.close(&id_clone).await;

            if crashed && auto_restart {
                tracing::info!("Task {} crashed (code {:?}), queuing auto-restart", id_clone, exit_code);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                let _ = restart_tx.send(id_clone).await;
            }
        });

        Ok(pid)
    }
}
