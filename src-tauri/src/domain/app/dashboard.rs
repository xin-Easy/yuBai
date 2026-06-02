use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_instances: usize,
    pub running_instances: usize,
    pub proxy_count: usize,
    pub core_count: usize,
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub app_root: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbench {
    pub stats: DashboardWorkbenchStats,
    pub system: DashboardWorkbenchSystem,
    pub default_core: Option<DashboardWorkbenchDefaultCore>,
    pub automation: DashboardWorkbenchAutomation,
    pub running_profiles: Vec<DashboardWorkbenchRunningProfile>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbenchStats {
    pub total_instances: usize,
    pub running_instances: usize,
    pub unbound_instances: usize,
    pub proxy_count: usize,
    pub healthy_proxy_count: usize,
    pub untested_proxy_count: usize,
    pub core_count: usize,
    pub invalid_core_count: usize,
    pub script_count: usize,
    pub run_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbenchSystem {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub app_root: String,
    pub proxy_mode: String,
    pub mihomo_downloaded: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbenchDefaultCore {
    pub core_id: String,
    pub core_name: String,
    pub path_valid: bool,
    pub path_message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbenchAutomation {
    pub ready: bool,
    pub installed: bool,
    pub last_error: String,
    pub node_version: String,
    pub playwright_version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWorkbenchRunningProfile {
    pub profile_id: String,
    pub profile_name: String,
    pub debug_port: i32,
    pub runtime_proxy_summary: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigPayload {
    pub name: String,
    pub app_root: String,
    pub version: String,
}
