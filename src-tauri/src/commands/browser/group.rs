use tauri::State;

use crate::{
    app::state::AppState,
    domain::browser::{BrowserGroup, BrowserGroupInput, BrowserGroupWithCount, MoveProfilesToGroupInput},
};

#[tauri::command]
pub fn browser_group_list(
    state: State<'_, AppState>,
) -> Result<Vec<BrowserGroupWithCount>, String> {
    state
        .services
        .browser_group
        .list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_group_save(
    state: State<'_, AppState>,
    input: BrowserGroupInput,
) -> Result<BrowserGroup, String> {
    state
        .services
        .browser_group
        .save(state.inner(), input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_group_delete(state: State<'_, AppState>, group_id: String) -> Result<(), String> {
    state
        .services
        .browser_group
        .delete(state.inner(), group_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_group_move_profiles(
    state: State<'_, AppState>,
    input: MoveProfilesToGroupInput,
) -> Result<(), String> {
    state
        .services
        .browser_group
        .move_profiles(state.inner(), input.profile_ids, input.group_id)
        .map_err(Into::into)
}
