use crate::{
    app::state::AppState,
    domain::browser::{BrowserProfile, BrowserProfileInput},
    error::AppError,
    infra::browser,
};

pub struct BrowserProfileService;

impl BrowserProfileService {
    pub fn list(&self, state: &AppState) -> Result<Vec<BrowserProfile>, AppError> {
        browser::runtime::list_profiles(state)
    }

    pub fn create(
        &self,
        state: &AppState,
        input: BrowserProfileInput,
    ) -> Result<BrowserProfile, AppError> {
        let config = state.config_snapshot().map_err(AppError::other)?;
        state.repositories.database.create_profile(input, &config)
    }

    pub fn update(
        &self,
        state: &AppState,
        profile_id: String,
        input: BrowserProfileInput,
    ) -> Result<BrowserProfile, AppError> {
        state.repositories.database.update_profile(profile_id, input)
    }

    pub fn delete(&self, state: &AppState, profile_id: String) -> Result<(), AppError> {
        state.repositories.database.delete_profile(profile_id)
    }
}
