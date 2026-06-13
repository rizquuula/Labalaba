use std::io::{self, Write};
use std::path::Path;

/// Load the daemon auth token from `{data_dir}/daemon.token`, creating it if
/// absent or empty. The generated token is two concatenated v4 UUID simple
/// (hex-only, no hyphens) strings — 64 lowercase hex chars total.
pub fn load_or_create_token(data_dir: &Path) -> io::Result<String> {
    let token_path = data_dir.join("daemon.token");

    if token_path.exists() {
        let contents = std::fs::read_to_string(&token_path)?;
        let trimmed = contents.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );

    let mut file = std::fs::File::create(&token_path)?;
    file.write_all(token.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&token_path, perms)?;
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_load_returns_same_token() {
        let dir = tempfile::tempdir().unwrap();
        let token1 = load_or_create_token(dir.path()).unwrap();
        assert_eq!(token1.len(), 64);
        let token2 = load_or_create_token(dir.path()).unwrap();
        assert_eq!(token1, token2);
    }

    #[test]
    fn second_call_with_existing_file_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let token1 = load_or_create_token(dir.path()).unwrap();
        let token2 = load_or_create_token(dir.path()).unwrap();
        let token3 = load_or_create_token(dir.path()).unwrap();
        assert_eq!(token1, token2);
        assert_eq!(token2, token3);
    }
}
