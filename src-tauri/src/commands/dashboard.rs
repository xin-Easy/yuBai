use tauri::State;

use crate::{app::state::AppState, domain::app::{AppConfigPayload, DashboardStats, DashboardWorkbench}};

#[tauri::command]
pub fn dashboard_stats_get(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    state
        .services
        .dashboard
        .stats(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn dashboard_workbench_get(state: State<'_, AppState>) -> Result<DashboardWorkbench, String> {
    state
        .services
        .dashboard
        .workbench(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn app_config_get(state: State<'_, AppState>) -> AppConfigPayload {
    state.services.dashboard.app_config(state.inner())
}
