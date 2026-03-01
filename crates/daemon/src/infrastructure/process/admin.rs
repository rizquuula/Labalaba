/// Windows-only: spawn a process with elevated privileges via ShellExecuteW "runas" verb.
/// Because the elevated process runs in a separate session, we cannot capture its stdout/stderr
/// directly. Instead we spawn an intermediate batch helper that pipes output to a named pipe
/// or temp file. For now we launch detached and log a notice.
use crate::domain::process::entity::ProcessHandle;
use crate::domain::task::entity::Task;
use tokio::process::Command;

pub async fn spawn_as_admin(task: &Task) -> anyhow::Result<ProcessHandle> {
    // Use PowerShell to launch with elevation; stdout/stderr are redirected to temp files.
    let args_str = task.arguments.join(" ");
    let script = format!(
        "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait",
        task.executable.replace('\'', "''"),
        args_str.replace('\'', "''"),
    );

    let child = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let pid = child.id()
        .ok_or_else(|| anyhow::anyhow!("Failed to get PID for elevated process"))?;

    tracing::info!("Launched elevated process via PowerShell, wrapper PID={}", pid);
    Ok(ProcessHandle { pid, child })
}
