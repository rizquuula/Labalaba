use labalaba_shared::api::UpdateInfo;
use reqwest::Client;
use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API: &str = "https://api.github.com/repos/YOUR_ORG/labalaba/releases/latest";

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
        let resp = self.client.get(GITHUB_API).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let release: GithubRelease = r.json().await?;
                let latest = release.tag_name.trim_start_matches('v').to_string();
                let available = latest != CURRENT_VERSION;
                Ok(UpdateInfo {
                    available,
                    current_version: CURRENT_VERSION.to_string(),
                    latest_version: Some(latest),
                    release_url: Some(release.html_url),
                    release_notes: release.body,
                })
            }
            _ => Ok(UpdateInfo {
                available: false,
                current_version: CURRENT_VERSION.to_string(),
                latest_version: None,
                release_url: None,
                release_notes: None,
            }),
        }
    }
}

impl Default for GithubUpdater {
    fn default() -> Self {
        Self::new()
    }
}
