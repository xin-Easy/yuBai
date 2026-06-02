use crate::app::state::AppState;
use tauri::State;

#[tauri::command]
pub fn file_resolve_path(state: State<'_, AppState>, path: String) -> String {
    state.services.file.resolve_path(state.inner(), path)
}

#[tauri::command]
pub fn file_exists(state: State<'_, AppState>, path: String) -> bool {
    state.services.file.exists(state.inner(), path)
}
