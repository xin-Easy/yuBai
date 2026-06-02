use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLogEntry {
    pub level: String,
    pub message: String,
    pub target: String,
    pub details: String,
    pub created_at: String,
}
