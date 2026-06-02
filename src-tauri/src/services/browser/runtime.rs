use tauri::AppHandle;

use crate::{
    app::state::AppState,
    domain::{
        browser::BrowserProfile,
        runtime::{BrowserTabInfo, CookieInfo},
    },
    error::AppError,
    infra::{browser, logging},
};

pub struct BrowserRuntimeService;

impl BrowserRuntimeService {
    pub fn get_cookies(&self, state: &AppState, profile_id: &str) -> Result<Vec<CookieInfo>, AppError> {
        browser::runtime::get_cookies(state, profile_id)
    }

    pub fn clear_cookies(&self, state: &AppState, profile_id: &str) -> Result<(), AppError> {
        browser::runtime::clear_cookies(state, profile_id)
    }

    pub fn export_cookies(&self, state: &AppState, profile_id: &str) -> Result<String, AppError> {
        browser::runtime::export_cookies(state, profile_id)
    }

    pub fn tabs_list(&self, state: &AppState, profile_id: &str) -> Result<Vec<BrowserTabInfo>, AppError> {
        browser::runtime::tabs_list(state, profile_id)
    }

    pub fn status(&self, state: &AppState, profile_id: &str) -> Result<BrowserProfile, AppError> {
        browser::runtime::status_profile(state, profile_id)
    }

    pub fn start(
        &self,
        app: AppHandle,
        state: &AppState,
        profile_id: String,
    ) -> Result<BrowserProfile, AppError> {
        let profile = browser::runtime::start_profile(state, profile_id)?;
        browser::runtime::watch_profile_exit(app, profile.profile_id.clone())?;
        logging::log_target(
            "info",
            "browser.instance",
            format!("browser started: {}", profile.profile_name),
            format!("profileId={} debugPort={}", profile.profile_id, profile.debug_port),
        );
        Ok(profile)
    }

    pub fn stop(&self, state: &AppState, profile_id: String) -> Result<BrowserProfile, AppError> {
        let profile = browser::runtime::stop_profile(state, profile_id)?;
        logging::log_target(
            "info",
            "browser.instance",
            format!("browser stopped: {}", profile.profile_name),
            format!("profileId={}", profile.profile_id),
        );
        Ok(profile)
    }
}
