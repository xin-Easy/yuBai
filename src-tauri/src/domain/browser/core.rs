use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCore {
    pub core_id: String,
    pub core_name: String,
    pub core_path: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreInput {
    pub core_id: Option<String>,
    pub core_name: String,
    pub core_path: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreValidateResult {
    pub valid: bool,
    pub message: String,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreExtendedInfo {
    pub core_id: String,
    pub chrome_version: String,
    pub instance_count: i32,
    pub path_valid: bool,
    pub path_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreDownloadOption {
    pub version: String,
    pub channel: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreDownloadInput {
    pub core_name: String,
    pub version: String,
    pub url: String,
    pub proxy_config: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserCoreDownloadProgress {
    pub phase: String,
    pub progress: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemBrowserCandidate {
    pub name: String,
    pub path: String,
    pub version: String,
    pub source: String,
    pub registered: bool,
}
