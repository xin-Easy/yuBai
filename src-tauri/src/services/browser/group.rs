use crate::{
    app::state::AppState,
    domain::browser::{BrowserGroup, BrowserGroupInput, BrowserGroupWithCount},
    error::AppError,
};

pub struct BrowserGroupService;

impl BrowserGroupService {
    pub fn list(&self, state: &AppState) -> Result<Vec<BrowserGroupWithCount>, AppError> {
        state.repositories.database.list_groups()
    }

    pub fn save(&self, state: &AppState, input: BrowserGroupInput) -> Result<BrowserGroup, AppError> {
        state.repositories.database.save_group(input)
    }

    pub fn delete(&self, state: &AppState, group_id: String) -> Result<(), AppError> {
        state.repositories.database.delete_group(group_id)
    }

    pub fn move_profiles(
        &self,
        state: &AppState,
        profile_ids: Vec<String>,
        group_id: String,
    ) -> Result<(), AppError> {
        state.repositories.database.move_profiles_to_group(profile_ids, group_id)
    }
}
