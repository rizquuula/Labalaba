use async_trait::async_trait;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::process::Command;
use crate::domain::process::entity::{ProcessHandle, ProcessWaiter};
use crate::domain::process::service::ProcessSpawner;
use crate::domain::task::entity::Task;

#[cfg(test)]
use labalaba_shared::task::TaskId;
#[cfg(test)]
use std::collections::HashMap;

pub struct OsProcessSpawner;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_prefix_parsing() {
        // Test simple runner
        let runner = "python";
        let mut parts: Vec<&str> = runner.split_whitespace().collect();
        let exe = parts.remove(0).to_string();
        let args: Vec<String> = parts.into_iter().map(String::from).collect();
        
        assert_eq!(exe, "python");
        assert!(args.is_empty());

        // Test runner with arguments
        let runner = "uv run";
        let mut parts: Vec<&str> = runner.split_whitespace().collect();
        let exe = parts.remove(0).to_string();
        let args: Vec<String> = parts.into_iter().map(String::from).collect();
        
        assert_eq!(exe, "uv");
        assert_eq!(args, vec!["run"]);

        // Test complex runner
        let runner = "pipenv run python";
        let mut parts: Vec<&str> = runner.split_whitespace().collect();
        let exe = parts.remove(0).to_string();
        let args: Vec<String> = parts.into_iter().map(String::from).collect();
        
        assert_eq!(exe, "pipenv");
        assert_eq!(args, vec!["run", "python"]);
    }

    #[test]
    fn test_command_construction_with_runner_prefix() {
        let mut task = Task {
            id: TaskId::new(),
            description: "Test Python Script".to_string(),
            executable: "C:\\project\\app.py".to_string(),
            arguments: vec!["--port".to_string(), "8080".to_string()],
            working_directory: Some("C:\\project".to_string()),
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: Some("uv run".to_string()),
            pids: vec![],
        };

        // Simulate the logic from spawn()
        let (executable, args) = if let Some(ref runner) = task.runner_prefix {
            let mut parts: Vec<&str> = runner.split_whitespace().collect();
            if parts.is_empty() {
                (task.executable.clone(), task.arguments.clone())
            } else {
                let exe = parts.remove(0).to_string();
                let runner_args: Vec<String> = parts.into_iter().map(String::from).collect();
                let mut all_args = runner_args;
                all_args.push(task.executable.clone());
                all_args.extend(task.arguments.clone());
                (exe, all_args)
            }
        } else {
            (task.executable.clone(), task.arguments.clone())
        };

        assert_eq!(executable, "uv");
        assert_eq!(args, vec!["run", "C:\\project\\app.py", "--port", "8080"]);

        // Test without runner prefix
        task.runner_prefix = None;
        let (executable, args) = if let Some(ref runner) = task.runner_prefix {
            let mut parts: Vec<&str> = runner.split_whitespace().collect();
            if parts.is_empty() {
                (task.executable.clone(), task.arguments.clone())
            } else {
                let exe = parts.remove(0).to_string();
                let runner_args: Vec<String> = parts.into_iter().map(String::from).collect();
                let mut all_args = runner_args;
                all_args.push(task.executable.clone());
                all_args.extend(task.arguments.clone());
                (exe, all_args)
            }
        } else {
            (task.executable.clone(), task.arguments.clone())
        };

        assert_eq!(executable, "C:\\project\\app.py");
        assert_eq!(args, vec!["--port", "8080"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_spawn_captures_merged_output() {
        use std::io::Read;

        let task = Task {
            id: TaskId::new(),
            description: "printf test".to_string(),
            executable: "sh".to_string(),
            arguments: vec!["-c".to_string(), "printf 'a\\nb\\nc\\n'".to_string()],
            working_directory: None,
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids: vec![],
        };

        let spawner = OsProcessSpawner;
        let mut handle = spawner.spawn(&task).await.expect("spawn should succeed");
        let reader = handle.output.take().expect("PTY reader should be present");

        // Read the merged stream to EOF on a blocking thread. On Linux, reading a
        // PTY master after the slave closes can return EIO instead of a clean EOF;
        // any already-buffered bytes are still delivered, so ignore the final err.
        let captured = tokio::task::spawn_blocking(move || {
            let mut reader = reader;
            let mut buf = String::new();
            let _ = reader.read_to_string(&mut buf);
            buf
        })
        .await
        .expect("reader thread should join");

        assert!(captured.contains('a'), "captured output missing 'a': {captured:?}");
        assert!(captured.contains('b'), "captured output missing 'b': {captured:?}");
        assert!(captured.contains('c'), "captured output missing 'c': {captured:?}");

        let exit_code = handle.waiter.wait().await;
        assert_eq!(exit_code, Some(0));
    }
}

#[async_trait]
impl ProcessSpawner for OsProcessSpawner {
    async fn spawn(&self, task: &Task) -> anyhow::Result<ProcessHandle> {
        #[cfg(target_os = "windows")]
        if task.run_as_admin {
            return super::admin::spawn_as_admin(task).await;
        }

        // Handle runner prefix (e.g., "uv run" → exe="uv", args=["run", script.py, ...])
        let (executable, args) = if let Some(ref runner) = task.runner_prefix {
            let mut parts: Vec<&str> = runner.split_whitespace().collect();
            if parts.is_empty() {
                (task.executable.clone(), task.arguments.clone())
            } else {
                let exe = parts.remove(0).to_string();
                let runner_args: Vec<String> = parts.into_iter().map(String::from).collect();
                let mut all_args = runner_args;
                all_args.push(task.executable.clone()); // Script path
                all_args.extend(task.arguments.clone()); // Original args
                (exe, all_args)
            }
        } else {
            (task.executable.clone(), task.arguments.clone())
        };

        // Run the child on a pseudo-terminal so it line-buffers its output.
        // Most programs switch to block buffering when their stdout is a pipe,
        // which delays log lines until several KB accumulate or the process
        // exits. A PTY makes the child believe it is attached to a terminal, so
        // it flushes per line. stdout and stderr are merged into one stream.
        // portable-pty also sets up a new session (pgid == pid), so the existing
        // group-kill in kill_tree still reaches the whole process tree.
        let pair = native_pty_system().openpty(PtySize {
            rows: 24,
            cols: 200,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(&executable);
        cmd.args(&args);

        // Seeing a TTY, some programs start emitting ANSI color/control codes.
        // Default to a no-color, well-known terminal type to reduce that (lines
        // are additionally ANSI-stripped downstream). The task's own env wins.
        if !task.environment.contains_key("NO_COLOR") {
            cmd.env("NO_COLOR", "1");
        }
        if !task.environment.contains_key("TERM") {
            cmd.env("TERM", "xterm-256color");
        }
        for (k, v) in &task.environment {
            cmd.env(k, v);
        }

        if let Some(ref wd) = task.working_directory {
            cmd.cwd(wd);
        }

        let child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let pid = child
            .process_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;

        // Drop the slave so the master reader receives EOF once the child exits.
        drop(pair.slave);

        Ok(ProcessHandle {
            pid,
            output: Some(reader),
            waiter: ProcessWaiter::Pty(child),
        })
    }

    async fn kill(&self, pid: u32) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output()
                .await?;
        }
        Ok(())
    }

    async fn kill_tree(&self, pid: u32) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await?;
        }
        #[cfg(unix)]
        {
            // Children are spawned in their own process group (pgid == pid),
            // so signalling the negative pid hits the whole tree at once.
            let group_result = unsafe { libc::kill(-(pid as i32), libc::SIGTERM) };
            if group_result != 0 {
                // Group kill failed (e.g. ESRCH because the leader already
                // exited but a child lingers) — fall back to the direct PID.
                let direct_result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
                if direct_result != 0 {
                    let err = std::io::Error::last_os_error();
                    // ESRCH simply means the process is already gone; not an error.
                    if err.raw_os_error() != Some(libc::ESRCH) {
                        anyhow::bail!("Failed to kill PID {}: {}", pid, err);
                    }
                }
            }
        }
        Ok(())
    }
}
