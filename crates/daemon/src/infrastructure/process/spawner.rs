use async_trait::async_trait;
use tokio::process::Command;
use crate::domain::process::entity::ProcessHandle;
use crate::domain::process::service::ProcessSpawner;
use crate::domain::task::entity::Task;

pub struct OsProcessSpawner;

#[async_trait]
impl ProcessSpawner for OsProcessSpawner {
    async fn spawn(&self, task: &Task) -> anyhow::Result<ProcessHandle> {
        #[cfg(target_os = "windows")]
        if task.run_as_admin {
            return super::admin::spawn_as_admin(task).await;
        }

        let mut cmd = Command::new(&task.executable);
        cmd.args(&task.arguments);
        cmd.envs(&task.environment);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        if let Some(ref wd) = task.working_directory {
            cmd.current_dir(wd);
        }

        // Prevent child process from inheriting parent's console on Windows
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn()?;
        let pid = child.id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;

        Ok(ProcessHandle { pid, child })
    }

    async fn kill(&self, pid: u32) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
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
}
