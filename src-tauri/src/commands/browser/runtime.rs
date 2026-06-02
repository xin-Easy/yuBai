use tauri::{AppHandle, State};

use crate::{
    app::state::AppState,
    domain::{browser::BrowserProfile, runtime::{BrowserTabInfo, CookieInfo}},
};

#[tauri::command]
pub fn browser_get_cookies(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<Vec<CookieInfo>, String> {
    state
        .services
        .browser_runtime
        .get_cookies(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_clear_cookies(state: State<'_, AppState>, profile_id: String) -> Result<(), String> {
    state
        .services
        .browser_runtime
        .clear_cookies(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_export_cookies(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<String, String> {
    state
        .services
        .browser_runtime
        .export_cookies(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_tabs_list(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<Vec<BrowserTabInfo>, String> {
    state
        .services
        .browser_runtime
        .tabs_list(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_instance_status(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<BrowserProfile, String> {
    state
        .services
        .browser_runtime
        .status(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_instance_start(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<BrowserProfile, String> {
    state
        .services
        .browser_runtime
        .start(app, state.inner(), profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_instance_stop(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<BrowserProfile, String> {
    state
        .services
        .browser_runtime
        .stop(state.inner(), profile_id)
        .map_err(Into::into)
}
