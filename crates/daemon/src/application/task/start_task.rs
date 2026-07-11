use std::sync::Arc;
use labalaba_shared::task::TaskId;
use labalaba_shared::api::LogStream;
use crate::domain::log::entity::{make_log_entry, new_log_channel};
use crate::infrastructure::state::AppState;

/// Strip ANSI escape sequences (CSI/SGR color codes, OSC, and simple two-byte
/// escapes) from a line. Children switch to emitting these once a PTY makes them
/// believe they are on a terminal; the log viewer wants plain text.
fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0x1b {
            // ESC
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'[' => {
                    // CSI: parameter/intermediate bytes until a final byte in @..~
                    i += 1;
                    while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1; // consume the final byte
                    }
                }
                b']' => {
                    // OSC: terminated by BEL (0x07) or ST (ESC \)
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == 0x07 {
                            i += 1;
                            break;
                        }
                        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {
                    // Other escape (e.g. ESC c): skip the single following byte.
                    i += 1;
                }
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
    // `out` only ever contains bytes copied verbatim from valid UTF-8 input
    // (escape sequences are pure ASCII), so this is lossless in practice.
    String::from_utf8_lossy(&out).into_owned()
}

pub struct StartTask {
    pub state: Arc<AppState>,
}

impl StartTask {
    pub async fn execute(&self, id: TaskId) -> anyhow::Result<u32> {
        let task = self.state.task_repo.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", id))?;

        // Atomically claim the Starting state in a single critical section so
        // two concurrent starts can't both pass the not-running check and spawn
        // duplicate processes.
        {
            let mut states = self.state.runtime_states.write().await;
            let claimed = states.entry(id.clone()).or_default().mark_starting_if_stopped();
            if !claimed {
                anyhow::bail!("Task {} is already running", id);
            }
        }

        // Sleep only AFTER claiming Starting, so the slot is reserved while waiting.
        if task.startup_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(task.startup_delay_ms)).await;
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

        // Register PID to task and persist via a locked read-modify-write so a
        // concurrent StopTask clearing pids can't clobber this push (or vice versa).
        self.state.task_repo.update_pids(&id, Box::new(move |mut pids| {
            pids.push(pid);
            pids
        })).await?;

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

        // The PTY merges stdout+stderr into one blocking reader. Read it on a
        // dedicated OS thread (blocking I/O must not run on the async runtime),
        // strip ANSI escapes, and hand each line to the async side over an mpsc
        // channel. The thread exits on EOF/Err so it can't leak across restarts.
        if let Some(reader) = handle.output.take() {
            let (line_tx, mut line_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

            std::thread::spawn(move || {
                use std::io::BufRead;
                let buf = std::io::BufReader::new(reader);
                for line in buf.lines() {
                    match line {
                        Ok(line) => {
                            if line_tx.send(strip_ansi(&line)).is_err() {
                                break; // receiver gone
                            }
                        }
                        Err(_) => break, // EOF (or EIO on PTY close) — stop reading
                    }
                }
                // Dropping `line_tx` here closes the channel and ends the drainer.
            });

            let tx = broadcaster.clone();
            let id_out = id.clone();
            let writer = log_writer.clone();
            let cb = log_cb.clone();
            tokio::spawn(async move {
                // Every captured line is tagged Stdout because the PTY merges the
                // two streams; the per-line broadcast + persist + callback logic
                // is otherwise unchanged.
                while let Some(line) = line_rx.recv().await {
                    let entry = make_log_entry(&id_out, LogStream::Stdout, line);
                    let _ = tx.send(entry.clone());
                    let _ = writer.write(&id_out, &entry).await;
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
        // Move the waiter out of the (now output-drained) handle so the exit
        // watcher owns it. The waiter yields the exit code across both spawn
        // backends (PTY child / tokio child).
        let waiter = handle.waiter;
        tokio::spawn(async move {
            let exit_code = waiter.wait().await;
            let nonzero_exit = exit_code.map(|c| c != 0).unwrap_or(true);

            // Decide how to react under a single critical section: a deliberate
            // StopTask marks the state Stopping/Stopped before/while killing, so
            // an exit during that window is intentional — never a crash, never a
            // restart. Otherwise apply the backoff/cap bookkeeping.
            enum Action {
                Intentional,
                Restart(u64),
                CrashedNoRestart,
            }

            let action = {
                let mut states = state_clone.runtime_states.write().await;
                let rt = states.entry(id_clone.clone()).or_default();

                if rt.is_stopping_or_stopped() {
                    rt.mark_stopped(exit_code);
                    Action::Intentional
                } else if !nonzero_exit {
                    rt.mark_stopped(exit_code);
                    Action::Intentional
                } else if !auto_restart {
                    rt.mark_crashed(exit_code);
                    Action::CrashedNoRestart
                } else if rt.restart_cap_reached() {
                    rt.mark_crashed(exit_code);
                    Action::CrashedNoRestart
                } else {
                    let delay = rt.register_restart_attempt();
                    rt.mark_crashed(exit_code);
                    Action::Restart(delay)
                }
            };

            let _ = log_writer.close(&id_clone).await;

            // The process owning `pid` has exited, so drop it from the persisted
            // PID list (mirrors recovery_exit_watcher). Without this, frequently
            // re-run short-lived tasks — e.g. a cron-scheduled one-shot — would
            // accumulate dead PIDs in tasks.yaml until the next daemon restart.
            let _ = state_clone
                .task_repo
                .update_pids(&id_clone, Box::new(move |pids| {
                    pids.into_iter().filter(|&p| p != pid).collect()
                }))
                .await;

            match action {
                Action::Intentional => {
                    tracing::info!("Task {} exited intentionally (code {:?})", id_clone, exit_code);
                }
                Action::CrashedNoRestart => {
                    tracing::warn!(
                        "Task {} crashed (code {:?}); auto-restart disabled or retry cap reached",
                        id_clone,
                        exit_code
                    );
                }
                Action::Restart(delay_secs) => {
                    tracing::info!(
                        "Task {} crashed (code {:?}), queuing auto-restart in {}s",
                        id_clone,
                        exit_code,
                        delay_secs
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                    let _ = restart_tx.send(id_clone).await;
                }
            }
        });

        Ok(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn test_strip_ansi_sgr() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn test_strip_ansi_plain_passthrough() {
        assert_eq!(strip_ansi("plain text"), "plain text");
    }

    #[test]
    fn test_strip_ansi_multiple_codes() {
        assert_eq!(
            strip_ansi("\x1b[1m\x1b[32mok\x1b[0m done\x1b[0m"),
            "ok done"
        );
    }

    #[test]
    fn test_strip_ansi_osc_and_utf8() {
        // OSC title sequence terminated by BEL, plus a multibyte char preserved.
        assert_eq!(strip_ansi("\x1b]0;title\x07caf\u{e9}"), "caf\u{e9}");
    }
}
