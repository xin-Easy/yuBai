use tauri::{AppHandle, State};

use crate::{app::state::AppState, infra::launch_api::LaunchApiConfig};

#[tauri::command]
pub fn launch_api_config_get(state: State<'_, AppState>) -> LaunchApiConfig {
    state.services.launch_api_app.config(state.inner())
}

#[tauri::command]
pub fn launch_api_start(
    app: AppHandle,
    state: State<'_, AppState>,
    port: Option<u16>,
) -> Result<LaunchApiConfig, String> {
    state
        .services
        .launch_api_app
        .start(app, state.inner(), port)
        .map_err(Into::into)
}

#[tauri::command]
pub fn launch_api_stop(state: State<'_, AppState>) -> LaunchApiConfig {
    state.services.launch_api_app.stop(state.inner())
}
