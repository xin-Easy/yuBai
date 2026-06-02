use tauri::AppHandle;

use crate::{app::state::AppState, error::AppError, infra::automation};

pub use crate::infra::automation::{
    AutomationRun, AutomationRunInput, AutomationRuntimeState, AutomationScript,
    AutomationScriptInput, AutomationSelfCheck,
};

pub struct AutomationService;

impl AutomationService {
    pub fn state(&self, state: &AppState) -> Result<AutomationRuntimeState, AppError> {
        automation::state_get(state)
    }

    pub fn settings_save(
        &self,
        state: &AppState,
        enabled: bool,
        headless_default: bool,
    ) -> Result<AutomationRuntimeState, AppError> {
        automation::settings_save(state, enabled, headless_default)
    }

    pub fn runtime_install(&self, app: AppHandle) -> Result<AutomationRuntimeState, AppError> {
        automation::install_runtime_with_events(app)
    }

    pub fn self_check(&self, state: &AppState) -> Result<AutomationSelfCheck, AppError> {
        automation::self_check(state)
    }

    pub fn script_list(&self, state: &AppState) -> Result<Vec<AutomationScript>, AppError> {
        automation::list_scripts(state)
    }

    pub fn script_create(
        &self,
        state: &AppState,
        input: AutomationScriptInput,
    ) -> Result<AutomationScript, AppError> {
        automation::create_script(state, input)
    }

    pub fn script_update(
        &self,
        state: &AppState,
        script_id: String,
        input: AutomationScriptInput,
    ) -> Result<AutomationScript, AppError> {
        automation::update_script(state, script_id, input)
    }

    pub fn script_delete(&self, state: &AppState, script_id: String) -> Result<(), AppError> {
        automation::delete_script(state, script_id)
    }

    pub fn run_list(&self, state: &AppState, limit: i32) -> Result<Vec<AutomationRun>, AppError> {
        automation::list_runs(state, limit)
    }

    pub fn run(&self, app: AppHandle, input: AutomationRunInput) -> Result<AutomationRun, AppError> {
        automation::enqueue_script_run(app, input)
    }

    pub fn cancel(&self, app: AppHandle, run_id: String) -> Result<AutomationRun, AppError> {
        automation::cancel_run(app, run_id)
    }
}
