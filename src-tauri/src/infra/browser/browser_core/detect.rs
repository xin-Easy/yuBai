use crate::{
    app::state::AppState,
    domain::{
        browser::{
            BrowserCore, BrowserCoreExtendedInfo, BrowserCoreInput, SystemBrowserCandidate,
        },
        paths::resolve_app_path,
        text::non_empty_owned,
    },
};
use std::{collections::HashSet, fs, path::Path};
use uuid::Uuid;

use super::{
    shared::{
        app_root, chrome_version_from_executable, chrome_version_from_manifest, normalize_path_key,
        system_browser_paths,
    },
    validate::validate_core_path,
};

pub fn extended_info(state: &AppState) -> Result<Vec<BrowserCoreExtendedInfo>, String> {
    let cores = state.repositories.database.list_cores()?;
    let profiles = state.repositories.database.list_profiles()?;
    let default_core_id = cores
        .iter()
        .find(|core| core.is_default)
        .or_else(|| cores.first())
        .map(|core| core.core_id.clone())
        .unwrap_or_default();

    let mut result = Vec::with_capacity(cores.len());
    for core in cores {
        let validation = validate_core_path(app_root(state), &core.core_path);
        let instance_count = profiles
            .iter()
            .filter(|profile| {
                if profile.core_id.trim().is_empty() || profile.core_id == "default" {
                    core.core_id == default_core_id
                } else {
                    profile.core_id == core.core_id
                }
            })
            .count() as i32;
        let version = validation
            .executable_path
            .as_deref()
            .and_then(|path| chrome_version_from_executable(Path::new(path)))
            .or_else(|| chrome_version_from_manifest(&resolve_app_path(app_root(state), &core.core_path)))
            .unwrap_or_default();

        result.push(BrowserCoreExtendedInfo {
            core_id: core.core_id,
            chrome_version: version,
            instance_count,
            path_valid: validation.valid,
            path_message: validation.message,
        });
    }
    Ok(result)
}

pub fn scan_local_cores(state: &AppState) -> Result<Vec<BrowserCore>, String> {
    let chrome_root = state.app_root.join("chrome");
    let mut registered = state.repositories.database.list_cores()?;
    let existing_paths: HashSet<String> = registered
        .iter()
        .map(|core| normalize_path_key(&resolve_app_path(app_root(state), &core.core_path)))
        .collect();

    if !chrome_root.exists() {
        return Ok(registered);
    }

    let mut candidates = Vec::new();
    if super::validate::find_core_executable(&chrome_root).is_some() {
        candidates.push(("Default Chrome".to_string(), "chrome".to_string()));
    }

    let entries =
        fs::read_dir(&chrome_root).map_err(|err| format!("failed to scan chrome dir: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read chrome dir entry: {err}"))?;
        if !entry.file_type().map(|file_type| file_type.is_dir()).unwrap_or(false) {
            continue;
        }
        let abs = entry.path();
        if super::validate::find_core_executable(&abs).is_none() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        candidates.push((format!("Chrome {}", folder), format!("chrome/{}", folder)));
    }

    for (name, relative_path) in candidates {
        let abs = resolve_app_path(app_root(state), &relative_path);
        if existing_paths.contains(&normalize_path_key(&abs)) {
            continue;
        }
        let core = state.repositories.database.save_core(BrowserCoreInput {
            core_id: Some(format!("core-{}", Uuid::new_v4())),
            core_name: name,
            core_path: relative_path,
            is_default: registered.is_empty(),
        })?;
        registered.push(core);
    }

    state.repositories.database.list_cores().map_err(Into::into)
}

pub fn detect_system_browsers(state: &AppState) -> Result<Vec<SystemBrowserCandidate>, String> {
    let registered = state.repositories.database.list_cores()?;
    let registered_paths: HashSet<String> = registered
        .iter()
        .map(|core| normalize_path_key(&resolve_app_path(app_root(state), &core.core_path)))
        .collect();

    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for (name, path, source) in system_browser_paths() {
        if !path.is_file() {
            continue;
        }
        let key = normalize_path_key(&path);
        if !seen.insert(key.clone()) {
            continue;
        }
        result.push(SystemBrowserCandidate {
            name,
            path: path.to_string_lossy().to_string(),
            version: chrome_version_from_executable(&path).unwrap_or_default(),
            source,
            registered: registered_paths.contains(&key),
        });
    }
    Ok(result)
}

pub fn register_system_browser(
    state: &AppState,
    path: String,
    core_name: Option<String>,
    is_default: bool,
) -> Result<BrowserCore, String> {
    let path = path.trim();
    let validation = validate_core_path(app_root(state), path);
    if !validation.valid {
        return Err(validation.message);
    }

    let version = chrome_version_from_executable(Path::new(path)).unwrap_or_default();
    let name = core_name.and_then(non_empty_owned).unwrap_or_else(|| {
        if version.is_empty() {
            "System Chrome".to_string()
        } else {
            format!("System Chrome {}", version)
        }
    });

    state
        .repositories
        .database
        .save_core(BrowserCoreInput {
            core_id: Some(format!("system-chrome-{}", Uuid::new_v4())),
            core_name: name,
            core_path: path.to_string(),
            is_default,
        })
        .map_err(Into::into)
}
