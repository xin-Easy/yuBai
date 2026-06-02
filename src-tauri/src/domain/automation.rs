use super::browser::BrowserProxy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxySubscriptionFetchResult {
    pub url: String,
    pub content: String,
    pub proxy_count: usize,
    pub dns_servers: Option<String>,
    pub suggested_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxyImportInput {
    pub content: String,
    pub group_name: Option<String>,
    pub name_prefix: Option<String>,
    pub dns_servers: Option<String>,
    pub source_id: Option<String>,
    pub source_url: Option<String>,
    pub source_name_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxyImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub items: Vec<BrowserProxy>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxyTestResult {
    pub proxy_id: String,
    pub ok: bool,
    pub latency_ms: i64,
    pub tested_at: String,
    pub ip_health_json: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProxyBatchTestResult {
    pub total: usize,
    pub ok: usize,
    pub failed: usize,
    pub items: Vec<BrowserProxyTestResult>,
}
