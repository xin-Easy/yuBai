use tauri::{AppHandle, State};

use crate::{app::state::AppState, services::automation::{
    AutomationRun, AutomationRunInput, AutomationRuntimeState, AutomationScript,
    AutomationScriptInput, AutomationSelfCheck,
}};

#[tauri::command]
pub fn automation_state_get(
    state: State<'_, AppState>,
) -> Result<AutomationRuntimeState, String> {
    state.services.automation.state(state.inner()).map_err(Into::into)
}

#[tauri::command]
pub fn automation_settings_save(
    state: State<'_, AppState>,
    enabled: bool,
    headless_default: bool,
) -> Result<AutomationRuntimeState, String> {
    state
        .services
        .automation
        .settings_save(state.inner(), enabled, headless_default)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_runtime_install(
    app: AppHandle,
) -> Result<AutomationRuntimeState, String> {
    crate::services::automation::AutomationService
        .runtime_install(app)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_runtime_self_check(
    state: State<'_, AppState>,
) -> Result<AutomationSelfCheck, String> {
    state
        .services
        .automation
        .self_check(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_list(
    state: State<'_, AppState>,
) -> Result<Vec<AutomationScript>, String> {
    state
        .services
        .automation
        .script_list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_create(
    state: State<'_, AppState>,
    input: AutomationScriptInput,
) -> Result<AutomationScript, String> {
    state
        .services
        .automation
        .script_create(state.inner(), input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_update(
    state: State<'_, AppState>,
    script_id: String,
    input: AutomationScriptInput,
) -> Result<AutomationScript, String> {
    state
        .services
        .automation
        .script_update(state.inner(), script_id, input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_delete(
    state: State<'_, AppState>,
    script_id: String,
) -> Result<(), String> {
    state
        .services
        .automation
        .script_delete(state.inner(), script_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_run_list(
    state: State<'_, AppState>,
    limit: i32,
) -> Result<Vec<AutomationRun>, String> {
    state
        .services
        .automation
        .run_list(state.inner(), limit)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_run(
    app: AppHandle,
    input: AutomationRunInput,
) -> Result<AutomationRun, String> {
    crate::services::automation::AutomationService
        .run(app, input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn automation_script_run_cancel(
    app: AppHandle,
    run_id: String,
) -> Result<AutomationRun, String> {
    crate::services::automation::AutomationService
        .cancel(app, run_id)
        .map_err(Into::into)
}
