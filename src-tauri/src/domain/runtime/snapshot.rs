use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub snapshot_id: String,
    pub profile_id: String,
    pub name: String,
    #[serde(rename = "sizeMB")]
    pub size_mb: f64,
    pub created_at: String,
}
