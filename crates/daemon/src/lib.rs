pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use labalaba_shared::api::{AppSettings, LogEntry, UpdateInfo};
use labalaba_shared::task::TaskId;
use infrastructure::{
    persistence::yaml_repository::YamlTaskRepository,
    process::spawner::OsProcessSpawner,
    process::liveness::{expected_process_stem, is_task_process_alive},
    scheduler::cron_scheduler::CronScheduler,
    updater::github_updater::GithubUpdater,
    state::AppState,
    log::file_writer::LogFileWriter,
};
use application::task::start_task::StartTask;
use application::update::check_update::CheckUpdate;
use domain::scheduler::service::SchedulerService;

/// Base directory for all data files (tasks.yaml, settings.yaml, logs/).
/// Reads `LABALABA_DATA_DIR` env var; falls back to the current working directory.
pub fn data_dir() -> PathBuf {
    std::env::var("LABALABA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolve a path that may be relative (e.g. "./tasks.yaml") against a base dir.
fn resolve(base: &Path, p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(p.trim_start_matches("./"))
    }
}

/// Check if a process with the given PID is still alive AND (best effort)
/// belongs to a task whose process image stem is `expected_stem`.
///
/// `expected_stem` should come from
/// [`infrastructure::process::liveness::expected_process_stem`]. When it is
/// `None` (no usable executable name) this degrades to a plain liveness check.
/// The identity check is conservative: on platforms where identity can be
/// determined, a recycled PID running a *different* process is reported as not
/// running, so recovery marks the task Crashed rather than later killing a
/// stranger.
pub fn is_process_running(pid: u32, expected_stem: Option<&str>) -> bool {
    is_task_process_alive(pid, expected_stem)
}

/// Load settings from `{data_dir}/settings.yaml`, falling back to defaults.
pub async fn load_settings() -> (AppSettings, String) {
    let base = data_dir();
    let settings_path = base.join("settings.yaml");
    let settings_path_str = settings_path.to_string_lossy().to_string();
    
    if settings_path.exists() {
        if let Ok(contents) = tokio::fs::read_to_string(&settings_path).await {
            if let Ok(s) = serde_yaml::from_str::<AppSettings>(&contents) {
                return (s, settings_path_str);
            }
        }
    }
    (AppSettings::default(), settings_path_str)
}

/// Initialize the full daemon AppState, recover task states, and spawn the restart loop.
///
/// `log_event_callback` — when `Some`, called for every log line produced by managed tasks.
/// Pass `None` for the standalone daemon (which streams via WebSocket instead).
/// `update_event_callback` — when `Some`, called when an update is available.
pub async fn init_app_state(
    log_event_callback: Option<Arc<dyn Fn(LogEntry) + Send + Sync + 'static>>,
    update_event_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
) -> anyhow::Result<Arc<AppState>> {
    let (settings, settings_path) = load_settings().await;

    let (restart_tx, restart_rx) = mpsc::channel::<TaskId>(64);

    let base = data_dir();
    let log_writer = LogFileWriter::new(
        resolve(&base, &settings.log_dir),
        settings.log_max_file_size_mb,
        settings.log_max_rotated_files,
    );
    log_writer.init_dir().await?;

    let auth_token = infrastructure::auth::token::load_or_create_token(&base)?;

    let config_path = resolve(&base, &settings.config_path);
    let repo = Arc::new(YamlTaskRepository::new(config_path));
    let spawner = Arc::new(OsProcessSpawner);
    let updater = Arc::new(GithubUpdater::new());
    let state = AppState::new(repo, spawner, updater, settings.clone(), settings_path.clone(), restart_tx, log_writer, log_event_callback, update_event_callback, auth_token);

    let scheduler = Arc::new(
        CronScheduler::new(Arc::downgrade(&state))
    );
    let _ = state.scheduler.set(scheduler);

    // Save settings to file if it doesn't exist
    if !std::path::Path::new(&settings_path).exists() {
        if let Err(e) = state.save_settings().await {
            tracing::warn!("Failed to save initial settings: {}", e);
        }
    }

    recover_task_states(Arc::clone(&state)).await;
    schedule_existing_tasks(&state).await;
    tokio::spawn(restart_loop(restart_rx, Arc::clone(&state)));

    // Always spawn the update checker; it re-reads `auto_check_updates` each
    // cycle and skips (but stays alive) while disabled, so toggling the setting
    // at runtime takes effect without a restart.
    let callback = state.update_event_callback.clone();
    spawn_update_checker(Arc::clone(&state), callback);

    Ok(state)
}

/// Schedule all tasks that have a cron expression defined.
/// Called once after `recover_task_states` so tasks whose cron fires while the
/// daemon was down are not retroactively triggered — they simply wait for the
/// next scheduled instant. A single invalid expression is logged and skipped so
/// it never prevents other tasks from being scheduled.
async fn schedule_existing_tasks(state: &Arc<AppState>) {
    let tasks = match state.task_repo.find_all().await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("schedule_existing_tasks: failed to load tasks: {}", e);
            return;
        }
    };
    let Some(scheduler) = state.scheduler.get() else { return };
    for task in tasks {
        if let Some(sched) = task.schedule {
            if let Err(e) = scheduler.schedule(task.id.clone(), &sched.cron).await {
                tracing::warn!(
                    "schedule_existing_tasks: invalid cron for task {}: {}",
                    task.id,
                    e
                );
            }
        }
    }
}

/// Receives restart requests from crashed tasks and re-executes them.
pub async fn restart_loop(mut rx: mpsc::Receiver<TaskId>, state: Arc<AppState>) {
    while let Some(id) = rx.recv().await {
        tracing::info!("Auto-restarting task {}", id);
        let uc = StartTask { state: Arc::clone(&state) };
        if let Err(e) = uc.execute(id.clone()).await {
            tracing::warn!("Auto-restart of {} failed: {}", id, e);
        }
    }
}

/// Interval at which a recovered task's liveness watcher polls the OS.
const RECOVERY_POLL_INTERVAL_SECS: u64 = 2;

/// Recover runtime states for tasks with persisted PIDs on daemon startup.
///
/// For each task with persisted PIDs we re-check liveness *with identity*
/// (guarding against recycled PIDs), prune dead/foreign PIDs from the persisted
/// list, then mark the task Running (with a polling exit watcher attached) or
/// Crashed. Recovered processes' stdout/stderr cannot be re-attached — those
/// pipes died with the previous parent — so logs are not re-streamed.
pub async fn recover_task_states(state: Arc<AppState>) {
    let tasks = match state.task_repo.find_all().await {
        Ok(tasks) => tasks,
        Err(e) => {
            tracing::warn!("Failed to load tasks for recovery: {}", e);
            return;
        }
    };

    for task in tasks {
        if task.pids.is_empty() {
            continue;
        }

        let expected_stem = expected_process_stem(&task);
        let original_pids = task.pids.clone();
        let alive_pids = prune_pids(&original_pids, |pid| {
            is_process_running(pid, expected_stem.as_deref())
        });

        // Prune the persisted PID list down to verified-alive PIDs so a later
        // Stop never targets a dead or recycled (foreign) PID. Tasks with no
        // survivors get pids = [] persisted.
        if alive_pids != original_pids {
            let pruned = alive_pids.clone();
            if let Err(e) = state
                .task_repo
                .update_pids(&task.id, Box::new(move |_| pruned))
                .await
            {
                tracing::warn!("Failed to prune stale PIDs for task {}: {}", task.id, e);
            }
        }

        {
            let mut states = state.runtime_states.write().await;
            let rt = states.entry(task.id.clone()).or_default();
            if let Some(&pid) = alive_pids.first() {
                rt.mark_running(pid);
            } else {
                rt.mark_crashed(None);
            }
        }

        if let Some(&pid) = alive_pids.first() {
            tracing::info!(
                "Recovered task {} as Running (pid {}, {} alive PID(s))",
                task.id,
                pid,
                alive_pids.len()
            );
            tokio::spawn(recovery_exit_watcher(
                Arc::clone(&state),
                task.id.clone(),
                pid,
                expected_stem,
                task.auto_restart,
            ));
        } else {
            tracing::info!(
                "Recovered task {} as Crashed (no alive PIDs from: {:?})",
                task.id,
                original_pids
            );
        }
    }
}

/// Poll a recovered process until it disappears, then mirror the StartTask exit
/// watcher's crash/restart semantics. Unlike StartTask (which `child.wait()`s a
/// pipe it owns) this can only poll the OS, since the recovered child is not our
/// direct descendant.
///
/// Termination conditions (the watcher stops polling when any holds):
///   * the process is no longer alive/ours — handle exit, then return;
///   * the task's persisted PIDs no longer contain `pid` — another path
///     (Stop/Restart) took over; leave its state alone and return;
///   * the in-memory status moved away from Running and the recorded pid no
///     longer matches `pid` — same reasoning.
async fn recovery_exit_watcher(
    state: Arc<AppState>,
    id: TaskId,
    pid: u32,
    expected_stem: Option<String>,
    auto_restart: bool,
) {
    let interval = std::time::Duration::from_secs(RECOVERY_POLL_INTERVAL_SECS);
    loop {
        tokio::time::sleep(interval).await;

        // Liveness first: the common case is the process is still running, and a
        // plain OS check is far cheaper than re-reading and parsing tasks.yaml.
        // The repo is only consulted once the process is found gone (below).
        if is_process_running(pid, expected_stem.as_deref()) {
            continue;
        }

        // The process is gone. Before acting, confirm we still own this PID in
        // the persisted list — if another code path (Stop/Restart) replaced it,
        // this watcher is stale and must stop without touching any state.
        match state.task_repo.find_by_id(&id).await {
            Ok(Some(t)) if t.pids.contains(&pid) => {}
            _ => return,
        }

        // Clear its PID, then apply the same intentional-vs-crash decision
        // StartTask uses.
        let _ = state
            .task_repo
            .update_pids(&id, Box::new(move |pids| {
                pids.into_iter().filter(|&p| p != pid).collect()
            }))
            .await;

        enum Action {
            Intentional,
            Restart(u64),
            CrashedNoRestart,
        }

        let action = {
            let mut states = state.runtime_states.write().await;
            let rt = states.entry(id.clone()).or_default();
            if rt.is_stopping_or_stopped() {
                rt.mark_stopped(None);
                Action::Intentional
            } else if !auto_restart {
                rt.mark_crashed(None);
                Action::CrashedNoRestart
            } else if rt.restart_cap_reached() {
                rt.mark_crashed(None);
                Action::CrashedNoRestart
            } else {
                let delay = rt.register_restart_attempt();
                rt.mark_crashed(None);
                Action::Restart(delay)
            }
        };

        let _ = state.log_writer.close(&id).await;

        match action {
            Action::Intentional => {
                tracing::info!("Recovered task {} exited intentionally", id);
            }
            Action::CrashedNoRestart => {
                tracing::warn!(
                    "Recovered task {} disappeared; auto-restart disabled or retry cap reached",
                    id
                );
            }
            Action::Restart(delay_secs) => {
                tracing::info!(
                    "Recovered task {} disappeared, queuing auto-restart in {}s",
                    id,
                    delay_secs
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                let _ = state.restart_tx.send(id).await;
            }
        }
        return;
    }
}

/// Compute the pruned PID list from an original list and a liveness predicate.
/// Extracted so the recovery pruning logic is unit-testable without spawning
/// real processes. Only PIDs the predicate reports alive are kept, order
/// preserved.
fn prune_pids<F: Fn(u32) -> bool>(original: &[u32], is_alive: F) -> Vec<u32> {
    original.iter().copied().filter(|&pid| is_alive(pid)).collect()
}

/// Delay before the first update check runs, giving the frontend time to start.
const UPDATE_CHECK_INITIAL_DELAY_SECS: u64 = 5;
/// Lower bound on the configured update-check interval, so a misconfigured
/// settings value can never hammer GitHub.
const UPDATE_CHECK_MIN_INTERVAL_HOURS: u64 = 1;

/// Spawn a background loop that periodically checks for updates.
///
/// Runs an initial check shortly after startup, then sleeps for
/// `update_check_interval_hours` (re-read from settings each cycle, clamped to a
/// sane minimum) before checking again. The `auto_check_updates` flag is also
/// re-read each cycle: while disabled the loop just sleeps and skips the check,
/// so toggling it at runtime takes effect without a restart. Startup/background
/// check failures only
/// log a warning — they never nag the user. The update callback fires only when
/// the latest version differs from the one already notified this session, so the
/// user is not pestered every interval for the same release.
fn spawn_update_checker(
    state: Arc<AppState>,
    update_callback: Option<Arc<dyn Fn(UpdateInfo) + Send + Sync + 'static>>,
) {
    tokio::spawn(async move {
        let mut last_notified: Option<String> = None;
        tokio::time::sleep(std::time::Duration::from_secs(UPDATE_CHECK_INITIAL_DELAY_SECS)).await;

        loop {
            // Re-read the enable flag each cycle so toggling auto-update off at
            // runtime stops the polling (and toggling it back on resumes it)
            // without a restart. Don't hold the settings lock across the await.
            let (enabled, interval_hours) = {
                let s = state.settings.read().await;
                (
                    s.auto_check_updates,
                    s.update_check_interval_hours.max(UPDATE_CHECK_MIN_INTERVAL_HOURS),
                )
            };

            if !enabled {
                tokio::time::sleep(std::time::Duration::from_secs(interval_hours * 3600)).await;
                continue;
            }

            let uc = CheckUpdate { state: Arc::clone(&state) };
            match uc.execute().await {
                Ok(info) => {
                    if info.available {
                        tracing::info!(
                            "Update available: {} (current: {})",
                            info.latest_version.as_ref().unwrap_or(&"unknown".to_string()),
                            info.current_version
                        );
                        // Store so the frontend can pull it on mount even if it
                        // registered its listener after the event fired.
                        *state.pending_update.write().await = Some(info.clone());

                        let is_new = info.latest_version != last_notified;
                        if is_new {
                            last_notified = info.latest_version.clone();
                            if let Some(cb) = &update_callback {
                                cb(info);
                            }
                        }
                    } else {
                        tracing::debug!("No update available (current: {})", info.current_version);
                    }
                }
                Err(e) => {
                    // Background check: log only, never surface to the user.
                    tracing::warn!("Failed to check for updates: {}", e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval_hours * 3600)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::repository::TaskRepository;

    #[test]
    fn prune_keeps_only_alive_pids_in_order() {
        let original = vec![10, 20, 30, 40];
        // Treat even PIDs as alive.
        let pruned = prune_pids(&original, |pid| pid % 20 == 0);
        assert_eq!(pruned, vec![20, 40]);
    }

    #[test]
    fn prune_all_dead_yields_empty() {
        let original = vec![1, 2, 3];
        let pruned = prune_pids(&original, |_| false);
        assert!(pruned.is_empty());
    }

    #[test]
    fn prune_all_alive_is_identity() {
        let original = vec![5, 6, 7];
        let pruned = prune_pids(&original, |_| true);
        assert_eq!(pruned, original);
    }

    fn task_with_pids(pids: Vec<u32>) -> labalaba_shared::task::TaskConfig {
        labalaba_shared::task::TaskConfig {
            id: TaskId::new(),
            description: "t".to_string(),
            executable: "/bin/true".to_string(),
            arguments: vec![],
            working_directory: None,
            environment: std::collections::HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: None,
            pids,
        }
    }

    #[tokio::test]
    async fn pruned_pids_are_persisted_via_update_pids() {
        use crate::application::dto::config_to_task;

        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);
        let repo = YamlTaskRepository::new(path);

        let config = task_with_pids(vec![100, 200, 300]);
        let id = config.id.clone();
        repo.save(&config_to_task(config)).await.unwrap();

        // Simulate recovery deciding only 200 is alive+ours.
        let pruned = prune_pids(&[100, 200, 300], |pid| pid == 200);
        let to_persist = pruned.clone();
        repo.update_pids(&id, Box::new(move |_| to_persist)).await.unwrap();

        let reloaded = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(reloaded.pids, vec![200]);
    }

    #[tokio::test]
    async fn no_survivors_persists_empty_pids() {
        use crate::application::dto::config_to_task;

        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);
        let repo = YamlTaskRepository::new(path);

        let config = task_with_pids(vec![1, 2]);
        let id = config.id.clone();
        repo.save(&config_to_task(config)).await.unwrap();

        let pruned = prune_pids(&[1, 2], |_| false);
        let to_persist = pruned.clone();
        repo.update_pids(&id, Box::new(move |_| to_persist)).await.unwrap();

        let reloaded = repo.find_by_id(&id).await.unwrap().unwrap();
        assert!(reloaded.pids.is_empty());
    }
}
