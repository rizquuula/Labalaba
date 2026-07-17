//! PID liveness checks with best-effort identity verification.
//!
//! A bare `kill(pid, 0)` (Unix) or substring match on `tasklist` output
//! (Windows) is unsafe: PIDs are recycled, so a dead task's stale PID can
//! match an unrelated live process. Recovery would then mark the task Running
//! and a later Stop would kill the stranger. These helpers compare the live
//! process against the task's *expected* executable name and, when identity
//! cannot be established, treat the process as NOT ours (conservative: better
//! to mark Crashed than to later kill an unrelated process).

use crate::domain::task::entity::Task;

/// The executable name we expect the OS process backing a task to report.
///
/// For a runner-prefixed task (e.g. `runner_prefix = "uv run"`) the actual
/// process image is the runner's first token (`uv`), not the script. Otherwise
/// it is the task's `executable`. Returned value is the file *stem* (no
/// directory, no extension) for cross-platform comparison.
pub fn expected_process_stem(task: &Task) -> Option<String> {
    let raw = match task.runner_prefix.as_deref() {
        Some(prefix) => prefix.split_whitespace().next().map(str::to_string),
        None => None,
    }
    .unwrap_or_else(|| task.executable.clone());

    file_stem(&raw)
}

/// Extract the file stem from a path-like string, handling both `/` and `\`
/// separators and stripping any extension (so `C:\bin\app.exe` -> `app`).
fn file_stem(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let last_component = trimmed
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(trimmed);
    let stem = match last_component.rsplit_once('.') {
        Some((before, _ext)) if !before.is_empty() => before,
        _ => last_component,
    };
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}

/// Check whether `pid` is alive AND (best effort) belongs to `task`.
///
/// `expected_stem` is the value of [`expected_process_stem`] for the task,
/// computed once by the caller. When it is `None` (no usable name) we fall
/// back to a plain liveness check, since there is nothing to compare against.
///
/// Windows answers liveness and identity from a single `tasklist` spawn: the
/// query already returns the image name. Spawning it twice doubled the cost of
/// startup recovery — which runs before the daemon binds its port — and of every
/// recovery watcher poll thereafter.
#[cfg(target_os = "windows")]
pub fn is_task_process_alive(pid: u32, expected_stem: Option<&str>) -> bool {
    let Some(image_name) = query_tasklist(pid) else {
        return false;
    };
    match expected_stem {
        Some(stem) => image_name_matches(&image_name, stem),
        None => true,
    }
}

/// Check whether `pid` is alive AND (best effort) belongs to `task`.
///
/// `expected_stem` is the value of [`expected_process_stem`] for the task,
/// computed once by the caller. When it is `None` (no usable name) we fall
/// back to a plain liveness check, since there is nothing to compare against.
#[cfg(not(target_os = "windows"))]
pub fn is_task_process_alive(pid: u32, expected_stem: Option<&str>) -> bool {
    if !pid_is_alive(pid) {
        return false;
    }
    match expected_stem {
        Some(stem) => process_identity_matches(pid, stem),
        None => true,
    }
}

#[cfg(not(target_os = "windows"))]
fn pid_is_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Best-effort identity check: does the live process named `pid` look like it
/// is running `expected_stem`? Conservative — returns `false` when identity
/// cannot be determined on a platform that supports the check.
#[cfg(target_os = "linux")]
fn process_identity_matches(pid: u32, expected_stem: &str) -> bool {
    match read_proc_identity(pid) {
        Some(idents) => idents
            .iter()
            .any(|name| comm_matches(name, expected_stem)),
        // /proc should exist on Linux; if we genuinely can't read it, be
        // conservative and disown the process.
        None => false,
    }
}

// macOS (and any other Unix without /proc): we have no cheap identity source,
// so liveness alone must stand. kept as a separate helper per the brief.
#[cfg(all(unix, not(target_os = "linux")))]
fn process_identity_matches(_pid: u32, _expected_stem: &str) -> bool {
    true
}

/// Run `tasklist /FI "PID eq {pid}" /FO CSV /NH` and return the image name for
/// an exact PID match, or `None` if the PID is absent.
#[cfg(target_os = "windows")]
fn query_tasklist(pid: u32) -> Option<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let output = Command::new("tasklist")
        .args([
            "/FI",
            &format!("PID eq {}", pid),
            "/FO",
            "CSV",
            "/NH",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_tasklist_csv(&stdout, pid)
}

/// Parse `tasklist` CSV output, returning the image name whose PID column
/// *exactly* equals `pid`. Returns `None` if no row matches (e.g. the
/// "INFO: No tasks..." banner tasklist prints when the filter matches nothing).
///
/// CSV columns: "Image Name","PID","Session Name","Session#","Mem Usage".
#[cfg(target_os = "windows")]
fn parse_tasklist_csv(stdout: &str, pid: u32) -> Option<String> {
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<String> = parse_csv_fields(line);
        if fields.len() < 2 {
            continue;
        }
        if fields[1].trim().parse::<u32>().ok() == Some(pid) {
            return Some(fields[0].trim().to_string());
        }
    }
    None
}

/// Minimal CSV field splitter for tasklist's quoted-comma format. tasklist
/// quotes every field and does not embed quotes, so a simple state machine
/// suffices (no escaped-quote handling needed).
#[cfg(target_os = "windows")]
fn parse_csv_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

/// Compare a Windows image name (e.g. `python.exe`) against an expected stem
/// (e.g. `python`), case-insensitively and ignoring the `.exe` suffix.
#[cfg(target_os = "windows")]
fn image_name_matches(image_name: &str, expected_stem: &str) -> bool {
    let image_stem = file_stem(image_name).unwrap_or_else(|| image_name.to_string());
    image_stem.eq_ignore_ascii_case(expected_stem)
}

/// Read `/proc/{pid}/comm` and the first token of `/proc/{pid}/cmdline` as
/// candidate identity strings. Returns `None` only if neither can be read.
#[cfg(target_os = "linux")]
fn read_proc_identity(pid: u32) -> Option<Vec<String>> {
    let mut idents = Vec::new();

    if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
        let comm = comm.trim();
        if !comm.is_empty() {
            idents.push(comm.to_string());
        }
    }

    if let Ok(cmdline) = std::fs::read(format!("/proc/{}/cmdline", pid)) {
        // cmdline is NUL-separated; the first segment is argv[0].
        if let Some(argv0) = cmdline.split(|&b| b == 0).next() {
            if let Ok(s) = std::str::from_utf8(argv0) {
                let s = s.trim();
                if !s.is_empty() {
                    idents.push(s.to_string());
                }
            }
        }
    }

    if idents.is_empty() {
        None
    } else {
        Some(idents)
    }
}

/// Match a `/proc` identity string against the expected stem. `comm` is
/// truncated to 15 chars by the kernel, and argv[0] may be a full path, so we
/// compare on file stems and also accept a truncation-prefix match.
#[cfg(any(target_os = "linux", test))]
fn comm_matches(candidate: &str, expected_stem: &str) -> bool {
    let cand_stem = file_stem(candidate).unwrap_or_else(|| candidate.to_string());
    if cand_stem.eq_ignore_ascii_case(expected_stem) {
        return true;
    }
    // `comm` is capped at 15 bytes (TASK_COMM_LEN-1); accept a prefix match so
    // a long executable name still verifies against its truncated comm.
    cand_stem.len() == 15 && expected_stem.len() > 15 && expected_stem.starts_with(&cand_stem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use labalaba_shared::task::TaskId;
    use std::collections::HashMap;

    fn task_with(executable: &str, runner_prefix: Option<&str>) -> Task {
        Task {
            id: TaskId::new(),
            description: "t".to_string(),
            executable: executable.to_string(),
            arguments: vec![],
            working_directory: None,
            environment: HashMap::new(),
            run_as_admin: false,
            auto_restart: false,
            schedule: None,
            startup_delay_ms: 0,
            depends_on: vec![],
            runner_prefix: runner_prefix.map(str::to_string),
            pids: vec![],
        }
    }

    #[test]
    fn file_stem_strips_dir_and_ext() {
        assert_eq!(file_stem("/usr/bin/python3").as_deref(), Some("python3"));
        assert_eq!(file_stem("C:\\bin\\app.exe").as_deref(), Some("app"));
        assert_eq!(file_stem("node").as_deref(), Some("node"));
        assert_eq!(file_stem("./run.sh").as_deref(), Some("run"));
        assert_eq!(file_stem("   ").as_deref(), None);
        assert_eq!(file_stem("").as_deref(), None);
        // Leading-dot names keep their name (no extension to strip).
        assert_eq!(file_stem(".bashrc").as_deref(), Some(".bashrc"));
    }

    #[test]
    fn expected_stem_uses_runner_first_token() {
        let t = task_with("/proj/app.py", Some("uv run"));
        assert_eq!(expected_process_stem(&t).as_deref(), Some("uv"));

        let t = task_with("/proj/app.py", Some("python3"));
        assert_eq!(expected_process_stem(&t).as_deref(), Some("python3"));
    }

    #[test]
    fn expected_stem_uses_executable_without_runner() {
        let t = task_with("/usr/local/bin/myserver", None);
        assert_eq!(expected_process_stem(&t).as_deref(), Some("myserver"));

        let t = task_with("C:\\svc\\worker.exe", None);
        assert_eq!(expected_process_stem(&t).as_deref(), Some("worker"));
    }

    #[test]
    fn comm_matches_exact_and_path_forms() {
        assert!(comm_matches("python3", "python3"));
        assert!(comm_matches("/usr/bin/python3", "python3"));
        assert!(comm_matches("Python3", "python3"));
        assert!(!comm_matches("bash", "python3"));
    }

    #[test]
    fn comm_matches_truncated_15_char_prefix() {
        // comm truncates to 15 chars; a 20-char expected name should still match.
        let expected = "averylongbinaryname_v2"; // > 15 chars
        let comm = &expected[..15]; // kernel-truncated comm
        assert_eq!(comm.len(), 15);
        assert!(comm_matches(comm, expected));
        // But a non-prefix 15-char comm must not match.
        assert!(!comm_matches("totallydifferent", expected));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_tasklist_csv_exact_pid_no_substring() {
        let out = "\"python.exe\",\"123\",\"Console\",\"1\",\"12,345 K\"\r\n\
                   \"node.exe\",\"1234\",\"Console\",\"1\",\"9,000 K\"\r\n";
        // Exact match for 123 returns python.exe, NOT node.exe (1234).
        assert_eq!(parse_tasklist_csv(out, 123).as_deref(), Some("python.exe"));
        assert_eq!(parse_tasklist_csv(out, 1234).as_deref(), Some("node.exe"));
        // A pid present only as a substring (e.g. 23 inside 123/1234) must miss.
        assert_eq!(parse_tasklist_csv(out, 23), None);
        // The "no tasks" banner yields no match.
        assert_eq!(
            parse_tasklist_csv("INFO: No tasks are running which match the criteria.", 123),
            None
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn image_name_matches_ignores_exe_and_case() {
        assert!(image_name_matches("python.exe", "python"));
        assert!(image_name_matches("Python.EXE", "python"));
        assert!(image_name_matches("uv.exe", "uv"));
        assert!(!image_name_matches("node.exe", "python"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_csv_fields_splits_quoted_commas() {
        let fields = parse_csv_fields("\"a,b\",\"123\",\"c\"");
        assert_eq!(fields, vec!["a,b".to_string(), "123".to_string(), "c".to_string()]);
    }
}
