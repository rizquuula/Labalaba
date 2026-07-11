use tokio::process::Child as TokioChild;

/// Yields the exit code of a spawned process, abstracting over the two spawn
/// backends: a PTY-backed child (blocking `wait`, bridged via `spawn_blocking`)
/// and a plain tokio child (used for the Windows elevated / no-capture path).
pub enum ProcessWaiter {
    /// A child spawned on a pseudo-terminal. `portable_pty::Child::wait` is
    /// blocking, so it is run on a blocking thread.
    Pty(Box<dyn portable_pty::Child + Send + Sync>),
    /// A plain async child.
    Tokio(TokioChild),
}

impl ProcessWaiter {
    /// Wait for the process to exit and return its exit code (`None` if it could
    /// not be determined, e.g. terminated by a signal).
    pub async fn wait(self) -> Option<i32> {
        match self {
            ProcessWaiter::Pty(mut child) => tokio::task::spawn_blocking(move || {
                child.wait().ok().map(|status| status.exit_code() as i32)
            })
            .await
            .ok()
            .flatten(),
            ProcessWaiter::Tokio(mut child) => child.wait().await.ok().and_then(|s| s.code()),
        }
    }
}

/// A live OS process handle managed by the daemon.
pub struct ProcessHandle {
    pub pid: u32,
    /// The merged (stdout+stderr) PTY reader. Blocking I/O, read on a dedicated
    /// thread. `None` for the elevated/no-capture path.
    pub output: Option<Box<dyn std::io::Read + Send>>,
    /// Waiter that yields the process exit code.
    pub waiter: ProcessWaiter,
}

impl std::fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessHandle").field("pid", &self.pid).finish()
    }
}
