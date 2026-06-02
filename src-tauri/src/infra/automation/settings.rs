use crate::{
    app::state::AppState,
    error::AppError,
};

use super::runtime;
use super::types::*;
use super::util::*;

pub fn state_get(state: &AppState) -> Result<AutomationRuntimeState, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let config = state.config_snapshot().map_err(AppError::from)?;
    let settings = read_settings(&paths)?;
    let check = self_check(state)?;
    Ok(AutomationRuntimeState {
        enabled: settings.enabled,
        runtime_version: runtime::runtime_version_label(&config),
        headless_default: settings.headless_default,
        installed: paths.runner.is_file() && runtime::bundled_node_paths(&paths, &config)?.is_some(),
        ready: settings.enabled && check.ok,
        installing: false,
        last_error: if settings.enabled {
            check.error
        } else {
            "automation runtime is disabled".to_string()
        },
        node_version: check.node_version,
        playwright_version: check.playwright_version,
    })
}

pub fn settings_save(
    state: &AppState,
    enabled: bool,
    headless_default: bool,
) -> Result<AutomationRuntimeState, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let settings = AutomationSettings {
        enabled,
        headless_default,
    };
    write_json_pretty(&paths.settings, &settings)?;
    state_get(state)
}

pub fn self_check(state: &AppState) -> Result<AutomationSelfCheck, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let config = state.config_snapshot().map_err(AppError::from)?;
    runtime::write_runtime_package(&paths.runtime)?;
    if !paths.runner.is_file() {
        runtime::write_runner(&paths.runner)?;
    }

    let Some(node_paths) = runtime::bundled_node_paths(&paths, &config)? else {
        return Ok(AutomationSelfCheck {
            ok: false,
            node_source: "bundled".to_string(),
            node_version: String::new(),
            playwright_version: String::new(),
            runner_path: paths.runner.to_string_lossy().to_string(),
            error: "bundled Node.js is not installed. Click prepare runtime first.".to_string(),
        });
    };

    let node_version = command_stdout(
        quiet_command(&node_paths.executable_path).arg("--version"),
    )?;
    let playwright_version =
        runtime::detect_playwright_version(&node_paths.executable_path, &paths.runtime).unwrap_or_default();
    let typescript_ready =
        runtime::detect_typescript_version(&node_paths.executable_path, &paths.runtime).is_ok_and(|value| !value.is_empty());
    let ok = !playwright_version.is_empty() && typescript_ready;
    let error = if ok {
        String::new()
    } else if playwright_version.is_empty() {
        "playwright-core is not available to the automation runner".to_string()
    } else {
        "typescript is not available to the automation runner".to_string()
    };
    Ok(AutomationSelfCheck {
        ok,
        node_source: "bundled".to_string(),
        node_version,
        playwright_version,
        runner_path: paths.runner.to_string_lossy().to_string(),
        error,
    })
}

pub fn read_settings(paths: &AutomationPaths) -> Result<AutomationSettings, AppError> {
    if !paths.settings.is_file() {
        let settings = AutomationSettings::default();
        write_json_pretty(&paths.settings, &settings)?;
        return Ok(settings);
    }
    read_json(&paths.settings)
}
