use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxy {
    pub proxy_id: String,
    pub proxy_name: String,
    pub proxy_config: String,
    pub dns_servers: Option<String>,
    pub group_name: Option<String>,
    pub source_id: Option<String>,
    pub source_url: Option<String>,
    pub source_name_prefix: Option<String>,
    pub source_auto_refresh: bool,
    pub source_refresh_interval_m: i32,
    pub source_last_refresh_at: Option<String>,
    pub last_latency_ms: i64,
    pub last_test_ok: bool,
    pub last_tested_at: Option<String>,
    #[serde(rename = "lastIPHealthJson")]
    pub last_ip_health_json: Option<String>,
    #[serde(default)]
    pub bound_profile_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxyInput {
    pub proxy_id: Option<String>,
    pub proxy_name: String,
    pub proxy_config: String,
    pub dns_servers: Option<String>,
    pub group_name: Option<String>,
    pub source_id: Option<String>,
    pub source_url: Option<String>,
    pub source_name_prefix: Option<String>,
}
