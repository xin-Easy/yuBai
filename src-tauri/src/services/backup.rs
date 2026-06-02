use crate::{app::state::AppState, error::AppError, infra::backup};

pub use crate::infra::backup::{BackupExportResult, BackupImportResult, BackupManifest};

pub struct BackupService;

impl BackupService {
    pub fn manifest_template(&self, state: &AppState) -> BackupManifest {
        backup::backup_manifest_template(state)
    }

    pub fn export_package(
        &self,
        state: &AppState,
        zip_path: String,
    ) -> Result<BackupExportResult, AppError> {
        backup::export_backup(state, &zip_path)
    }

    pub fn import_package(
        &self,
        state: &AppState,
        zip_path: String,
    ) -> Result<BackupImportResult, AppError> {
        backup::import_backup(state, &zip_path)
    }
}
