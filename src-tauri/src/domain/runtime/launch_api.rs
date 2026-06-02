use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchApiLogEntry {
    pub method: String,
    pub path: String,
    pub client_ip: String,
    pub launch_code: String,
    pub selector: serde_json::Value,
    pub request: serde_json::Value,
    pub success: bool,
    pub status_code: i32,
    pub error: String,
    pub profile_id: String,
    pub profile_name: String,
    pub created_at: String,
}
