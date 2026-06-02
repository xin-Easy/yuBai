use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserBookmark {
    pub name: String,
    pub url: String,
    pub open_on_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkSyncResult {
    pub total: i32,
    pub synced: i32,
    pub skipped: i32,
    pub failed: i32,
    pub skipped_list: Vec<String>,
    pub failed_list: Vec<String>,
}
