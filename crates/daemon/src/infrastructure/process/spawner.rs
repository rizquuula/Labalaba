use async_trait::async_trait;
#[cfg(unix)]
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

/// Resolve a task into the executable and argument vector to actually spawn.
///
/// A `runner_prefix` wraps the executable: `"uv run"` on `app.py` yields
/// `uv ["run", "app.py", ..original args]`.
fn resolve_command(task: &Task) -> (String, Vec<String>) {
    let Some(ref runner) = task.runner_prefix else {
        return (task.executable.clone(), task.arguments.clone());
    };

    let mut parts: Vec<&str> = runner.split_whitespace().collect();
    if parts.is_empty() {
        return (task.executable.clone(), task.arguments.clone());
    }

    let exe = parts.remove(0).to_string();
    let mut args: Vec<String> = parts.into_iter().map(String::from).collect();
    args.push(task.executable.clone());
    args.extend(task.arguments.clone());
    (exe, args)
}

/// Unix: run the child on a pseudo-terminal so it line-buffers its output.
///
/// Most programs switch to block buffering when their stdout is a pipe, which
/// delays log lines until several KB accumulate or the process exits. A PTY
/// makes the child believe it is attached to a terminal, so it flushes per
/// line. stdout and stderr are merged into one stream. portable-pty also sets
/// up a new session (pgid == pid), so the group-kill in kill_tree still reaches
/// the whole process tree.
#[cfg(unix)]
fn spawn_on_pty(task: &Task, executable: &str, args: &[String]) -> anyhow::Result<ProcessHandle> {
    let pair = native_pty_system().openpty(PtySize {
        rows: 24,
        cols: 200,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(executable);
    cmd.args(args);

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

/// Windows: run the child with stdout and stderr joined onto a single OS pipe.
///
/// A pseudo-console is deliberately NOT used here. ConPTY adds a console host
/// between the daemon and the child, and a packaged Python binary (Nuitka) was
/// seen dying under it with STATUS_DLL_INIT_FAILED (0xC0000142) before main()
/// ran, where a plain pipe spawn started the same binary fine. That single
/// observation was never reduced to a proven mechanism, so treat it as a
/// caution rather than a law: what is certain is that this pipe spawn is what
/// shipped and worked before the PTY was introduced, and it keeps Windows off
/// a fragile code path for no functional loss.
///
/// The cost is buffering: without a terminal, programs that block-buffer will
/// batch their log output. PYTHONUNBUFFERED covers the common Python case;
/// other programs may lag. Unix keeps the PTY (see spawn_on_pty).
///
/// Both writer ends target one pipe so the two streams stay interleaved in the
/// single reader `ProcessHandle` exposes.
#[cfg(windows)]
fn spawn_on_pipe(task: &Task, executable: &str, args: &[String]) -> anyhow::Result<ProcessHandle> {
    use filedescriptor::Pipe;

    // Keep the child's console off-screen; the daemon itself runs headless.
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new(executable);
    cmd.args(args);

    // Without a terminal Python block-buffers stdout, which would strand log
    // lines in the child until it exits. The task's own env wins.
    if !task.environment.contains_key("PYTHONUNBUFFERED") {
        cmd.env("PYTHONUNBUFFERED", "1");
    }
    if !task.environment.contains_key("NO_COLOR") {
        cmd.env("NO_COLOR", "1");
    }
    cmd.envs(&task.environment);

    if let Some(ref wd) = task.working_directory {
        cmd.current_dir(wd);
    }
    cmd.creation_flags(CREATE_NO_WINDOW);

    // `as_stdio` dups the handle, so the same write end can back both streams.
    let Pipe { read, write } = Pipe::new()?;
    cmd.stdout(write.as_stdio()?);
    cmd.stderr(write.as_stdio()?);

    let child = cmd.spawn()?;

    // Drop our own writer end: while it stays open the pipe never reaches EOF,
    // and the log-reader thread would block forever after the child exits.
    drop(write);

    let pid = child
        .id()
        .ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;

    Ok(ProcessHandle {
        pid,
        output: Some(Box::new(read)),
        waiter: ProcessWaiter::Tokio(child),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_with(runner_prefix: Option<&str>) -> Task {
        Task {
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
            runner_prefix: runner_prefix.map(String::from),
            pids: vec![],
        }
    }

    #[test]
    fn test_runner_prefix_parsing() {
        let (exe, args) = resolve_command(&task_with(Some("python")));
        assert_eq!(exe, "python");
        assert_eq!(args, vec!["C:\\project\\app.py", "--port", "8080"]);

        let (exe, args) = resolve_command(&task_with(Some("pipenv run python")));
        assert_eq!(exe, "pipenv");
        assert_eq!(
            args,
            vec!["run", "python", "C:\\project\\app.py", "--port", "8080"]
        );

        // A whitespace-only prefix is treated as no prefix at all.
        let (exe, args) = resolve_command(&task_with(Some("   ")));
        assert_eq!(exe, "C:\\project\\app.py");
        assert_eq!(args, vec!["--port", "8080"]);
    }

    #[test]
    fn test_command_construction_with_runner_prefix() {
        let (executable, args) = resolve_command(&task_with(Some("uv run")));
        assert_eq!(executable, "uv");
        assert_eq!(args, vec!["run", "C:\\project\\app.py", "--port", "8080"]);

        let (executable, args) = resolve_command(&task_with(None));
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

    /// Windows spawns onto a pipe rather than a ConPTY (see spawn_on_pipe), so
    /// cover that path too: both streams must reach the single reader, and the
    /// reader must see EOF once the child exits.
    #[cfg(windows)]
    #[tokio::test]
    async fn test_spawn_captures_merged_output() {
        use std::io::Read;

        let mut task = task_with(None);
        task.executable = "cmd".to_string();
        task.arguments = vec![
            "/c".to_string(),
            "echo to-stdout& echo to-stderr 1>&2".to_string(),
        ];
        task.working_directory = None;

        let spawner = OsProcessSpawner;
        let mut handle = spawner.spawn(&task).await.expect("spawn should succeed");
        let reader = handle.output.take().expect("pipe reader should be present");

        // Blocking read to EOF on a dedicated thread; EOF arrives only because
        // spawn_on_pipe drops its own copy of the write end.
        let captured = tokio::task::spawn_blocking(move || {
            let mut reader = reader;
            let mut buf = String::new();
            let _ = reader.read_to_string(&mut buf);
            buf
        })
        .await
        .expect("reader thread should join");

        assert!(
            captured.contains("to-stdout"),
            "captured output missing stdout: {captured:?}"
        );
        assert!(
            captured.contains("to-stderr"),
            "stderr was not merged into the reader: {captured:?}"
        );

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

        let (executable, args) = resolve_command(task);

        #[cfg(unix)]
        {
            spawn_on_pty(task, &executable, &args)
        }
        #[cfg(windows)]
        {
            spawn_on_pipe(task, &executable, &args)
        }
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
