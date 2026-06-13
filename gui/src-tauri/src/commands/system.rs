//! System/environment probing commands used by the GUI to make smart
//! suggestions — e.g. which interpreter to launch a script with.

use std::path::PathBuf;

/// Search `PATH` for the first available binary among `candidates`, returning
/// its resolved absolute path. On Windows, common executable extensions are
/// also tried so bare names like `pwsh` resolve to `pwsh.exe`.
fn which(candidates: &[&str]) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path_var).collect();
    for cand in candidates {
        for dir in &dirs {
            let direct = dir.join(cand);
            if direct.is_file() {
                return Some(direct.to_string_lossy().into_owned());
            }
            #[cfg(windows)]
            {
                for ext in ["exe", "cmd", "bat"] {
                    let with_ext = dir.join(format!("{cand}.{ext}"));
                    if with_ext.is_file() {
                        return Some(with_ext.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    None
}

/// Resolve the interpreter used to launch a script of the given `kind`.
///
/// `kind` is `"sh"` (POSIX shell scripts on macOS/Linux) or `"ps1"`
/// (PowerShell scripts on Windows). Returns the resolved interpreter path, or
/// `None` when the kind is unknown or no matching interpreter is installed.
#[tauri::command]
pub fn detect_interpreter(kind: String) -> Option<String> {
    match kind.as_str() {
        // Prefer a full-featured shell, fall back to the POSIX baseline.
        "sh" => which(&["bash", "zsh", "sh"]),
        // Prefer cross-platform PowerShell 7+ (pwsh), fall back to Windows PowerShell.
        "ps1" => which(&["pwsh", "powershell"]),
        _ => None,
    }
}
