use tauri::State;

use crate::{
    app::state::AppState,
    domain::runtime::{BookmarkSyncResult, BrowserBookmark},
};

#[tauri::command]
pub fn bookmark_list(state: State<'_, AppState>) -> Result<Vec<BrowserBookmark>, String> {
    state
        .services
        .bookmark
        .list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn bookmark_save(
    state: State<'_, AppState>,
    items: Vec<BrowserBookmark>,
) -> Result<(), String> {
    state
        .services
        .bookmark
        .save(state.inner(), items)
        .map_err(Into::into)
}

#[tauri::command]
pub fn bookmark_reset(state: State<'_, AppState>) -> Result<(), String> {
    state
        .services
        .bookmark
        .reset(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn bookmark_sync_to_profiles(state: State<'_, AppState>) -> Result<BookmarkSyncResult, String> {
    state
        .services
        .bookmark
        .sync_to_profiles(state.inner())
        .map_err(Into::into)
}
