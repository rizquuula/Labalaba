use async_trait::async_trait;
use tokio::process::Command;
use crate::domain::process::entity::ProcessHandle;
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

        let mut cmd = Command::new(&executable);
        cmd.args(&args);
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

    async fn kill_tree(&self, pid: u32) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F", "/T"])
                .output()
                .await?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("kill")
                .args(["-TERM", "-"])
                .arg(pid.to_string())
                .output()
                .await?;
        }
        Ok(())
    }
}
