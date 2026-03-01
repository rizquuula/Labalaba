use std::sync::Arc;
use labalaba_shared::api::UpdateInfo;
use crate::infrastructure::state::AppState;

pub struct CheckUpdate {
    pub state: Arc<AppState>,
}

impl CheckUpdate {
    pub async fn execute(&self) -> anyhow::Result<UpdateInfo> {
        self.state.updater.check().await
    }
}
