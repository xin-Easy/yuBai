use tauri::{AppHandle, State};

use crate::{
    app::state::AppState,
    domain::browser::{
        BrowserCore, BrowserCoreDownloadInput, BrowserCoreDownloadOption,
        BrowserCoreExtendedInfo, BrowserCoreInput, BrowserCoreValidateResult,
        SystemBrowserCandidate,
    },
};

#[tauri::command]
pub fn browser_core_list(state: State<'_, AppState>) -> Result<Vec<BrowserCore>, String> {
    state
        .services
        .browser_core
        .list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_save(
    state: State<'_, AppState>,
    input: BrowserCoreInput,
) -> Result<BrowserCore, String> {
    state
        .services
        .browser_core
        .save(state.inner(), input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_delete(state: State<'_, AppState>, core_id: String) -> Result<(), String> {
    state
        .services
        .browser_core
        .delete(state.inner(), core_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_validate(
    state: State<'_, AppState>,
    core_path: String,
) -> BrowserCoreValidateResult {
    state
        .services
        .browser_core
        .validate(state.inner(), core_path)
}

#[tauri::command]
pub fn browser_core_extended_info(
    state: State<'_, AppState>,
) -> Result<Vec<BrowserCoreExtendedInfo>, String> {
    state
        .services
        .browser_core
        .extended_info(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_scan_local(state: State<'_, AppState>) -> Result<Vec<BrowserCore>, String> {
    state
        .services
        .browser_core
        .scan_local(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_detect_system(
    state: State<'_, AppState>,
) -> Result<Vec<SystemBrowserCandidate>, String> {
    state
        .services
        .browser_core
        .detect_system(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_core_register_system(
    state: State<'_, AppState>,
    path: String,
    core_name: Option<String>,
    is_default: bool,
) -> Result<BrowserCore, String> {
    state
        .services
        .browser_core
        .register_system(state.inner(), path, core_name, is_default)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_core_download_options(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<BrowserCoreDownloadOption>, String> {
    state
        .services
        .browser_core
        .download_options(state.inner(), limit)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_core_download(
    app: AppHandle,
    input: BrowserCoreDownloadInput,
) -> Result<(), String> {
    crate::services::browser::core::BrowserCoreService::download(app, input)
        .await
        .map_err(Into::into)
}
