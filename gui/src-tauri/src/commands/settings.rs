use std::sync::Arc;
use labalaba_daemon::infrastructure::state::AppState;
use labalaba_shared::api::AppSettings;
use labalaba_daemon::application::update::check_update::CheckUpdate;
use labalaba_shared::api::UpdateInfo;

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let state = Arc::clone(&*state);
    let result = state.settings.read().await.clone();
    Ok(result)
}

#[tauri::command]
pub async fn update_settings(state: tauri::State<'_, Arc<AppState>>, settings: AppSettings) -> Result<AppSettings, String> {
    let state = Arc::clone(&*state);
    {
        let mut s = state.settings.write().await;
        *s = settings.clone();
    }
    Ok(settings)
}

#[tauri::command]
pub async fn check_update(state: tauri::State<'_, Arc<AppState>>) -> Result<UpdateInfo, String> {
    let state = Arc::clone(&*state);
    let uc = CheckUpdate { state: Arc::clone(&state) };
    uc.execute().await.map_err(|e| e.to_string())
}
