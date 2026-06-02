use tauri::State;

use crate::{
    app::state::AppState,
    domain::browser::{BrowserProfile, BrowserProfileInput},
};

#[tauri::command]
pub fn browser_profile_list(state: State<'_, AppState>) -> Result<Vec<BrowserProfile>, String> {
    state
        .services
        .browser_profile
        .list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_profile_create(
    state: State<'_, AppState>,
    input: BrowserProfileInput,
) -> Result<BrowserProfile, String> {
    state
        .services
        .browser_profile
        .create(state.inner(), input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_profile_update(
    state: State<'_, AppState>,
    profile_id: String,
    input: BrowserProfileInput,
) -> Result<BrowserProfile, String> {
    state
        .services
        .browser_profile
        .update(state.inner(), profile_id, input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_profile_delete(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    state
        .services
        .browser_profile
        .delete(state.inner(), profile_id)
        .map_err(Into::into)
}
