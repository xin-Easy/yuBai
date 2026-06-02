use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSettings {
    pub user_data_root: String,
    pub default_fingerprint_args: Vec<String>,
    pub default_launch_args: Vec<String>,
    pub default_start_urls: Vec<String>,
    pub restore_last_session: bool,
    pub start_ready_timeout_ms: i32,
    pub start_stable_window_ms: i32,
    #[serde(default = "default_download_source")]
    pub download_source: String,
    #[serde(default = "default_mihomo_download_source")]
    pub mihomo_download_source: String,
    #[serde(default = "default_proxy_mode")]
    pub proxy_mode: String,
    #[serde(default)]
    pub mihomo_downloaded: bool,
    #[serde(default)]
    pub mihomo_binary_path: String,
}

fn default_download_source() -> String {
    "auto".to_string()
}

fn default_mihomo_download_source() -> String {
    "auto".to_string()
}

fn default_proxy_mode() -> String {
    "auto".to_string()
}
