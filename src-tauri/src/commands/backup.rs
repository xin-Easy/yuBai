use tauri::State;

use crate::{app::state::AppState, services::backup::{BackupExportResult, BackupImportResult, BackupManifest}};

#[tauri::command]
pub fn backup_manifest_template(state: State<'_, AppState>) -> BackupManifest {
    state.services.backup.manifest_template(state.inner())
}

#[tauri::command]
pub fn backup_export_package(
    state: State<'_, AppState>,
    zip_path: String,
) -> Result<BackupExportResult, String> {
    state
        .services
        .backup
        .export_package(state.inner(), zip_path)
        .map_err(Into::into)
}

#[tauri::command]
pub fn backup_import_package(
    state: State<'_, AppState>,
    zip_path: String,
) -> Result<BackupImportResult, String> {
    state
        .services
        .backup
        .import_package(state.inner(), zip_path)
        .map_err(Into::into)
}
