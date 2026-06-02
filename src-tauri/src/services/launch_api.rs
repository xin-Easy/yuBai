use tauri::AppHandle;

use crate::{app::state::AppState, error::AppError, infra::launch_api::LaunchApiConfig};

pub struct LaunchApiAppService;

impl LaunchApiAppService {
    pub fn config(&self, state: &AppState) -> LaunchApiConfig {
        LaunchApiConfig {
            enabled: state.launch_api.service.enabled(),
            port: state.launch_api.service.port(),
            api_key: String::new(),
        }
    }

    pub fn start(
        &self,
        app: AppHandle,
        state: &AppState,
        port: Option<u16>,
    ) -> Result<LaunchApiConfig, AppError> {
        state.launch_api.service.start(app, port.unwrap_or(0))?;
        Ok(self.config(state))
    }

    pub fn stop(&self, state: &AppState) -> LaunchApiConfig {
        state.launch_api.service.stop();
        self.config(state)
    }
}
