use crate::{
    app::state::AppState,
    domain::{app::AppLogEntry, runtime::BrowserSettings},
    error::AppError,
    infra::{logging, proxy::mihomo},
};

pub struct SettingsService;

impl SettingsService {
    pub fn get_browser_settings(&self, state: &AppState) -> Result<BrowserSettings, AppError> {
        let config = state.config_snapshot().map_err(AppError::other)?;
        let mihomo_binary_path = mihomo::installed_binary_path(&state.app_root);
        Ok(BrowserSettings {
            user_data_root: config.browser.user_data_root,
            default_fingerprint_args: config.browser.default_fingerprint_args,
            default_launch_args: config.browser.default_launch_args,
            default_start_urls: config.browser.default_start_urls,
            restore_last_session: config.browser.restore_last_session,
            start_ready_timeout_ms: config.browser.start_ready_timeout_ms,
            start_stable_window_ms: config.browser.start_stable_window_ms,
            download_source: config.browser.download_source,
            mihomo_download_source: config.browser.mihomo_download_source,
            proxy_mode: normalize_proxy_mode(config.browser.proxy_mode),
            mihomo_downloaded: mihomo_binary_path.is_file(),
            mihomo_binary_path: mihomo_binary_path.to_string_lossy().to_string(),
        })
    }

    pub fn save_browser_settings(
        &self,
        state: &AppState,
        settings: BrowserSettings,
    ) -> Result<(), AppError> {
        let mut config = state.config_snapshot().map_err(AppError::other)?;
        config.browser.user_data_root = settings.user_data_root;
        config.browser.default_fingerprint_args = settings.default_fingerprint_args;
        config.browser.default_launch_args = settings.default_launch_args;
        config.browser.default_start_urls = settings.default_start_urls;
        config.browser.restore_last_session = settings.restore_last_session;
        config.browser.start_ready_timeout_ms = settings.start_ready_timeout_ms;
        config.browser.start_stable_window_ms = settings.start_stable_window_ms;
        config.browser.download_source = normalize_download_source(settings.download_source);
        config.browser.mihomo_download_source =
            normalize_download_source(settings.mihomo_download_source);
        config.browser.proxy_mode = normalize_proxy_mode(settings.proxy_mode);
        state.replace_config(config).map_err(AppError::other)
    }

    pub async fn download_mihomo(state: &AppState) -> Result<String, AppError> {
        let app_root = state.app_root.clone();
        let config = state.config_snapshot().map_err(AppError::other)?;
        tokio::task::spawn_blocking(move || {
            let path = mihomo::download_binary(&app_root, &config)?;
            Ok(path.to_string_lossy().to_string())
        })
        .await
        .map_err(|err| AppError::other(err.to_string()))?
    }

    pub fn app_logs(&self) -> Vec<AppLogEntry> {
        logging::global()
            .map(|logger| logger.list())
            .unwrap_or_default()
    }

    pub fn clear_app_logs(&self) {
        if let Some(logger) = logging::global() {
            logger.clear();
        }
    }
}

fn normalize_download_source(value: String) -> String {
    match value.trim().to_lowercase().as_str() {
        "official" => "official".to_string(),
        "npmmirror" | "mirror" => "npmmirror".to_string(),
        _ => "auto".to_string(),
    }
}

fn normalize_proxy_mode(value: String) -> String {
    match value.trim().to_lowercase().as_str() {
        "auto" => "auto".to_string(),
        "mihomo" => "mihomo".to_string(),
        "extension" | "plugin" => "extension".to_string(),
        _ => "auto".to_string(),
    }
}
