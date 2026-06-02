use crate::{
    app::state::AppState,
    domain::runtime::SnapshotInfo,
    error::AppError,
    infra::browser,
};

pub struct BrowserSnapshotService;

impl BrowserSnapshotService {
    pub fn list(&self, state: &AppState, profile_id: &str) -> Result<Vec<SnapshotInfo>, AppError> {
        browser::snapshot::list_snapshots(&state.app_root, profile_id)
    }

    pub fn create(
        &self,
        state: &AppState,
        profile_id: &str,
        name: &str,
    ) -> Result<SnapshotInfo, AppError> {
        let profile = state
            .repositories
            .database
            .get_profile(profile_id)?
            .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;
        browser::snapshot::create_snapshot(&state.app_root, profile_id, name, &profile.user_data_dir)
    }

    pub fn delete(
        &self,
        state: &AppState,
        profile_id: &str,
        snapshot_id: &str,
    ) -> Result<(), AppError> {
        browser::snapshot::delete_snapshot(&state.app_root, profile_id, snapshot_id)
    }

    pub fn restore(
        &self,
        state: &AppState,
        profile_id: &str,
        snapshot_id: &str,
    ) -> Result<(), AppError> {
        let profile = state
            .repositories
            .database
            .get_profile(profile_id)?
            .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;
        browser::snapshot::restore_snapshot(
            &state.app_root,
            &profile.user_data_dir,
            profile_id,
            snapshot_id,
        )
    }
}
