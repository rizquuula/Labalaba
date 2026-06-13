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
/// `kind` selects the candidate interpreters, ordered by preference and
/// matched to the script extension where it matters (a `.zsh` prefers `zsh`),
/// always falling back to a more universal shell. Returns the resolved
/// interpreter path, or `None` when the kind is unknown or none is installed.
pub fn detect_interpreter(kind: &str) -> Option<String> {
    match kind {
        "sh" => which(&["bash", "zsh", "sh"]),
        "bash" => which(&["bash", "sh"]),
        "zsh" => which(&["zsh", "bash", "sh"]),
        "ps1" => which(&["pwsh", "powershell"]),
        "bat" => std::env::var("ComSpec")
            .ok()
            .filter(|p| !p.is_empty())
            .or_else(|| which(&["cmd"])),
        _ => None,
    }
}
