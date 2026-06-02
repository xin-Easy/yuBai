use tauri::State;

use crate::{
    app::state::AppState,
    domain::runtime::SnapshotInfo,
};

#[tauri::command]
pub fn browser_snapshot_list(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<Vec<SnapshotInfo>, String> {
    state
        .services
        .browser_snapshot
        .list(state.inner(), &profile_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_snapshot_create(
    state: State<'_, AppState>,
    profile_id: String,
    name: String,
) -> Result<SnapshotInfo, String> {
    state
        .services
        .browser_snapshot
        .create(state.inner(), &profile_id, &name)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_snapshot_delete(
    state: State<'_, AppState>,
    profile_id: String,
    snapshot_id: String,
) -> Result<(), String> {
    state
        .services
        .browser_snapshot
        .delete(state.inner(), &profile_id, &snapshot_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_snapshot_restore(
    state: State<'_, AppState>,
    profile_id: String,
    snapshot_id: String,
) -> Result<(), String> {
    state
        .services
        .browser_snapshot
        .restore(state.inner(), &profile_id, &snapshot_id)
        .map_err(Into::into)
}
