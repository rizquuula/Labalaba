use labalaba_shared::api::UpdateInfo;
use reqwest::Client;
use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API: &str = "https://api.github.com/repos/rizquuula/Labalaba/releases/latest";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

pub struct GithubUpdater {
    client: Client,
}

impl GithubUpdater {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(format!("labalaba/{}", CURRENT_VERSION))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    pub async fn check(&self) -> anyhow::Result<UpdateInfo> {
        let resp = self
            .client
            .get(GITHUB_API)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Update check request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("Update check failed: GitHub returned HTTP {}", status);
        }

        let release: GithubRelease = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse GitHub release response: {}", e))?;

        let latest = release.tag_name.trim_start_matches('v').to_string();
        let available = is_newer(&latest, CURRENT_VERSION);

        Ok(UpdateInfo {
            available,
            current_version: CURRENT_VERSION.to_string(),
            latest_version: Some(latest),
            release_url: Some(release.html_url),
            release_notes: release.body,
        })
    }
}

/// True only when `latest` is a valid semver strictly greater than `current`.
/// Unparseable tags are treated as "no update" (with a warning) rather than
/// being flagged as an update by a naive string comparison.
fn is_newer(latest: &str, current: &str) -> bool {
    let latest_ver = match semver::Version::parse(latest) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Ignoring unparseable latest version tag '{}': {}", latest, e);
            return false;
        }
    };
    let current_ver = match semver::Version::parse(current) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Ignoring unparseable current version '{}': {}", current, e);
            return false;
        }
    };
    latest_ver > current_ver
}

impl Default for GithubUpdater {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn newer_latest_is_update() {
        assert!(is_newer("1.0.3", "1.0.2"));
        assert!(is_newer("2.0.0", "1.9.9"));
        assert!(is_newer("1.1.0", "1.0.9"));
    }

    #[test]
    fn older_latest_is_not_update() {
        assert!(!is_newer("1.0.1", "1.0.2"));
        assert!(!is_newer("0.9.0", "1.0.0"));
    }

    #[test]
    fn equal_version_is_not_update() {
        assert!(!is_newer("1.0.2", "1.0.2"));
    }

    #[test]
    fn prerelease_is_older_than_release() {
        // A pre-release of the same version is NOT newer than the release.
        assert!(!is_newer("1.0.2-rc.1", "1.0.2"));
        // But a pre-release of a higher version still counts.
        assert!(is_newer("1.1.0-rc.1", "1.0.2"));
    }

    #[test]
    fn garbage_tag_is_not_update() {
        assert!(!is_newer("not-a-version", "1.0.2"));
        assert!(!is_newer("latest", "1.0.2"));
        assert!(!is_newer("", "1.0.2"));
    }
}
