use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
    #[serde(default)]
    pub automation: AutomationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_type")]
    pub r#type: String,
    #[serde(default)]
    pub sqlite: SQLiteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SQLiteConfig {
    #[serde(default = "default_sqlite_path")]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_max_profile_limit")]
    pub max_profile_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    #[serde(default = "default_user_data_root")]
    pub user_data_root: String,
    #[serde(default)]
    pub default_fingerprint_args: Vec<String>,
    #[serde(default)]
    pub default_launch_args: Vec<String>,
    #[serde(default)]
    pub default_start_urls: Vec<String>,
    #[serde(default)]
    pub restore_last_session: bool,
    #[serde(default = "default_start_ready_timeout_ms")]
    pub start_ready_timeout_ms: i32,
    #[serde(default = "default_start_stable_window_ms")]
    pub start_stable_window_ms: i32,
    #[serde(default = "default_download_source")]
    pub download_source: String,
    #[serde(default = "default_mihomo_download_source")]
    pub mihomo_download_source: String,
    #[serde(default = "default_proxy_mode")]
    pub proxy_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    #[serde(default)]
    pub node: AutomationNodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationNodeConfig {
    #[serde(default = "default_automation_node_version")]
    pub version: String,
    #[serde(default = "default_automation_node_download_source")]
    pub download_source: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            app: AppConfig::default(),
            browser: BrowserConfig::default(),
            automation: AutomationConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            r#type: default_database_type(),
            sqlite: SQLiteConfig::default(),
        }
    }
}

impl Default for SQLiteConfig {
    fn default() -> Self {
        Self {
            path: default_sqlite_path(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            max_profile_limit: default_max_profile_limit(),
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            user_data_root: default_user_data_root(),
            default_fingerprint_args: vec![
                "--fingerprint-brand=Chrome".to_string(),
                "--fingerprint-platform=windows".to_string(),
            ],
            default_launch_args: vec!["--disable-sync".to_string(), "--no-first-run".to_string()],
            default_start_urls: Vec::new(),
            restore_last_session: false,
            start_ready_timeout_ms: default_start_ready_timeout_ms(),
            start_stable_window_ms: default_start_stable_window_ms(),
            download_source: default_download_source(),
            mihomo_download_source: default_mihomo_download_source(),
            proxy_mode: default_proxy_mode(),
        }
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            node: AutomationNodeConfig::default(),
        }
    }
}

impl Default for AutomationNodeConfig {
    fn default() -> Self {
        Self {
            version: default_automation_node_version(),
            download_source: default_automation_node_download_source(),
        }
    }
}

pub fn load_config(app_root: &Path) -> Config {
    let path = app_root.join("config.yaml");
    let Ok(raw) = fs::read_to_string(&path) else {
        let config = Config::default();
        if let Err(err) = fs::create_dir_all(app_root) {
            eprintln!("failed to create app config directory: {err}");
        }
        if let Ok(yaml) = serde_yaml::to_string(&config) {
            if let Err(err) = fs::write(&path, yaml) {
                eprintln!("failed to write default config.yaml: {err}");
            }
        }
        return config;
    };

    match serde_yaml::from_str::<Config>(&raw) {
        Ok(config) => config,
        Err(err) => {
            write_config_load_error(app_root, &raw, &err);
            Config::default()
        }
    }
}

fn write_config_load_error(app_root: &Path, raw: &str, err: &serde_yaml::Error) {
    let data_dir = app_root.join("data");
    if let Err(create_err) = fs::create_dir_all(&data_dir) {
        eprintln!("failed to create config diagnostic directory: {create_err}");
        return;
    }

    let message = format!(
        "time: {}\nerror: {err}\n\nconfig.yaml:\n{raw}\n",
        Utc::now().to_rfc3339()
    );
    if let Err(write_err) = fs::write(data_dir.join("config-load-error.log"), message) {
        eprintln!("failed to write config-load-error.log: {write_err}");
    }
}

fn default_database_type() -> String {
    "sqlite".to_string()
}

fn default_sqlite_path() -> String {
    "data/app.db".to_string()
}

fn default_app_name() -> String {
    "yubai".to_string()
}

fn default_max_profile_limit() -> usize {
    20
}

fn default_user_data_root() -> String {
    "data/profiles".to_string()
}

fn default_start_ready_timeout_ms() -> i32 {
    3000
}

fn default_start_stable_window_ms() -> i32 {
    1200
}

fn default_proxy_mode() -> String {
    "auto".to_string()
}

fn default_download_source() -> String {
    "auto".to_string()
}

fn default_mihomo_download_source() -> String {
    "auto".to_string()
}

fn default_automation_node_version() -> String {
    "24.16.0".to_string()
}

fn default_automation_node_download_source() -> String {
    "auto".to_string()
}
