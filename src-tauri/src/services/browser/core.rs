use tauri::AppHandle;

use crate::{
    app::state::AppState,
    domain::browser::{
        BrowserCore, BrowserCoreDownloadInput, BrowserCoreDownloadOption,
        BrowserCoreExtendedInfo, BrowserCoreInput, BrowserCoreValidateResult,
        SystemBrowserCandidate,
    },
    error::AppError,
    infra::browser,
};

pub struct BrowserCoreService;

impl BrowserCoreService {
    pub fn list(&self, state: &AppState) -> Result<Vec<BrowserCore>, AppError> {
        state.repositories.database.list_cores()
    }

    pub fn save(&self, state: &AppState, input: BrowserCoreInput) -> Result<BrowserCore, AppError> {
        state.repositories.database.save_core(input)
    }

    pub fn delete(&self, state: &AppState, core_id: String) -> Result<(), AppError> {
        state.repositories.database.delete_core(core_id)
    }

    pub fn validate(&self, state: &AppState, core_path: String) -> BrowserCoreValidateResult {
        browser::browser_core::validate_core_path(&state.app_root, &core_path)
    }

    pub fn extended_info(&self, state: &AppState) -> Result<Vec<BrowserCoreExtendedInfo>, AppError> {
        browser::browser_core::extended_info(state).map_err(AppError::other)
    }

    pub fn scan_local(&self, state: &AppState) -> Result<Vec<BrowserCore>, AppError> {
        browser::browser_core::scan_local_cores(state).map_err(AppError::other)
    }

    pub fn detect_system(&self, state: &AppState) -> Result<Vec<SystemBrowserCandidate>, AppError> {
        browser::browser_core::detect_system_browsers(state).map_err(AppError::other)
    }

    pub fn register_system(
        &self,
        state: &AppState,
        path: String,
        core_name: Option<String>,
        is_default: bool,
    ) -> Result<BrowserCore, AppError> {
        browser::browser_core::register_system_browser(state, path, core_name, is_default)
            .map_err(AppError::other)
    }

    pub async fn download_options(
        &self,
        state: &AppState,
        limit: Option<usize>,
    ) -> Result<Vec<BrowserCoreDownloadOption>, AppError> {
        let limit = limit.unwrap_or(24);
        let config = state.config_snapshot().map_err(AppError::other)?;
        tokio::task::spawn_blocking(move || {
            browser::browser_core::download_options_with_config(limit, Some(&config))
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
        .map_err(AppError::other)
    }

    pub async fn download(app: AppHandle, input: BrowserCoreDownloadInput) -> Result<(), AppError> {
        tokio::task::spawn_blocking(move || browser::browser_core::start_download_blocking(app, input))
            .await
            .map_err(|err| AppError::other(format!("task join error: {err}")))?
            .map_err(AppError::other)
    }
}
