use serde::{Deserialize, Serialize};

use super::fingerprint::BrowserFingerprint;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProfile {
    pub profile_id: String,
    pub profile_name: String,
    pub user_data_dir: String,
    pub core_id: String,
    pub fingerprint_args: Vec<String>,
    pub fingerprint: Option<BrowserFingerprint>,
    pub proxy_id: String,
    pub proxy_config: String,
    pub proxy_bind_source_id: Option<String>,
    pub proxy_bind_source_url: Option<String>,
    pub proxy_bind_name: Option<String>,
    pub proxy_bind_updated_at: Option<String>,
    pub launch_args: Vec<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub group_id: Option<String>,
    pub running: bool,
    pub debug_port: i32,
    pub debug_ready: bool,
    pub pid: i32,
    pub automation_run_id: Option<String>,
    pub runtime_proxy_summary: String,
    pub runtime_warning: String,
    pub last_error: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_start_at: Option<String>,
    pub last_stop_at: Option<String>,
    pub launch_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProfileInput {
    pub profile_name: String,
    pub user_data_dir: String,
    pub core_id: String,
    pub fingerprint_args: Vec<String>,
    pub fingerprint: Option<BrowserFingerprint>,
    pub proxy_id: String,
    pub proxy_config: String,
    pub launch_args: Vec<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub group_id: Option<String>,
}
