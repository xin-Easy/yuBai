use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserGroup {
    pub group_id: String,
    pub group_name: String,
    pub parent_id: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserGroupInput {
    pub group_id: Option<String>,
    pub group_name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserGroupWithCount {
    pub group_id: String,
    pub group_name: String,
    pub parent_id: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    pub instance_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveProfilesToGroupInput {
    pub profile_ids: Vec<String>,
    pub group_id: String,
}
