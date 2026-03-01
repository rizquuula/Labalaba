use tokio::process::Child;

/// A live OS process handle managed by the daemon
pub struct ProcessHandle {
    pub pid: u32,
    pub child: Child,
}

impl std::fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessHandle").field("pid", &self.pid).finish()
    }
}

/// Lightweight info about a process (safe to clone/send)
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
}
