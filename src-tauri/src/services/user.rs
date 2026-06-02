use crate::{app::state::AppState, domain::config::Config, error::AppError};

pub struct UserService;

impl UserService {
    pub fn config_get(&self, state: &AppState) -> Result<Config, AppError> {
        state.config_snapshot().map_err(AppError::other)
    }

    pub fn config_save(&self, state: &AppState, config: Config) -> Result<(), AppError> {
        state.replace_config(config).map_err(AppError::other)
    }
}
