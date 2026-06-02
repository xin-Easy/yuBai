use tauri::State;

use crate::{app::state::AppState, domain::{app::AppLogEntry, runtime::BrowserSettings}};

#[tauri::command]
pub fn browser_settings_get(state: State<'_, AppState>) -> Result<BrowserSettings, String> {
    state
        .services
        .settings
        .get_browser_settings(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_settings_save(
    state: State<'_, AppState>,
    settings: BrowserSettings,
) -> Result<(), String> {
    state
        .services
        .settings
        .save_browser_settings(state.inner(), settings)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn mihomo_download(state: State<'_, AppState>) -> Result<String, String> {
    crate::services::settings::SettingsService::download_mihomo(state.inner())
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub fn app_logs_get() -> Vec<AppLogEntry> {
    crate::services::settings::SettingsService.app_logs()
}

#[tauri::command]
pub fn app_logs_clear() {
    crate::services::settings::SettingsService.clear_app_logs();
}
