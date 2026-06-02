use crate::error::AppError;
use crate::app::state::AppState;
use chrono::Utc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use super::types::*;
use super::util::*;

pub fn list_scripts(state: &AppState) -> Result<Vec<AutomationScript>, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let mut scripts = Vec::new();
    for entry in fs::read_dir(&paths.scripts)? {
        let entry = entry?;
        let metadata = entry.path().join("script.json");
        if metadata.is_file() {
            let mut script = read_json::<AutomationScript>(&metadata)?;
            script.content = read_script_content(&entry.path(), &script.entry_file).unwrap_or_default();
            scripts.push(script);
        }
    }
    scripts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(scripts)
}

pub fn create_script(
    state: &AppState,
    input: AutomationScriptInput,
) -> Result<AutomationScript, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let name = non_empty(input.name, "script name is required")?;
    for entry in fs::read_dir(&paths.scripts)? {
        let entry = entry?;
        let metadata_path = entry.path().join("script.json");
        if metadata_path.is_file() {
            if let Ok(existing) = read_json::<AutomationScript>(&metadata_path) {
                if existing.name == name {
                    return Err(AppError::validation(format!(
                        "script with name \"{name}\" already exists"
                    )));
                }
            }
        }
    }
    let now = Utc::now().to_rfc3339();
    let script_id = format!("script-{}", Uuid::new_v4());
    let dir = paths.scripts.join(&script_id);
    fs::create_dir_all(&dir)?;
    let script = AutomationScript {
        script_id,
        name,
        description: input.description.unwrap_or_default(),
        entry_file: "main.ts".to_string(),
        version: "1.0.0".to_string(),
        created_at: now.clone(),
        updated_at: now,
        content: input.content.clone(),
    };
    fs::write(dir.join(&script.entry_file), input.content)?;
    write_json_pretty(&dir.join("script.json"), &script)?;
    Ok(script)
}

pub fn update_script(
    state: &AppState,
    script_id: String,
    input: AutomationScriptInput,
) -> Result<AutomationScript, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let dir = script_dir(&paths, &script_id)?;
    let metadata_path = dir.join("script.json");
    let mut script = read_json::<AutomationScript>(&metadata_path)?;
    let name = non_empty(input.name, "script name is required")?;
    for entry in fs::read_dir(&paths.scripts)? {
        let entry = entry?;
        if entry.file_name() == script_id.as_str() { continue; }
        let other_meta = entry.path().join("script.json");
        if other_meta.is_file() {
            if let Ok(existing) = read_json::<AutomationScript>(&other_meta) {
                if existing.name == name {
                    return Err(AppError::validation(format!(
                        "script with name \"{name}\" already exists"
                    )));
                }
            }
        }
    }
    script.name = name;
    script.description = input.description.unwrap_or_default();
    script.updated_at = Utc::now().to_rfc3339();
    script.content = input.content.clone();
    fs::write(dir.join(&script.entry_file), input.content)?;
    write_json_pretty(&metadata_path, &script)?;
    Ok(script)
}

pub fn delete_script(state: &AppState, script_id: String) -> Result<(), AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let dir = script_dir(&paths, &script_id)?;
    fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn get_script_by_id(paths: &AutomationPaths, script_id: &str) -> Result<AutomationScript, AppError> {
    let dir = script_dir(paths, script_id)?;
    let mut script = read_json::<AutomationScript>(&dir.join("script.json"))?;
    script.content = read_script_content(&dir, &script.entry_file).unwrap_or_default();
    Ok(script)
}

pub fn script_dir(paths: &AutomationPaths, script_id: &str) -> Result<std::path::PathBuf, AppError> {
    let safe_id = non_empty(script_id.to_string(), "script id is required")?;
    if safe_id.contains('/') || safe_id.contains('\\') || safe_id.contains("..") {
        return Err(AppError::validation("invalid script id"));
    }
    let dir = paths.scripts.join(safe_id);
    if !dir.is_dir() {
        return Err(AppError::not_found("automation script not found"));
    }
    Ok(dir)
}

pub fn canonical_script_entry(
    paths: &AutomationPaths,
    script: &AutomationScript,
) -> Result<String, AppError> {
    let dir = script_dir(paths, &script.script_id)?;
    let base = dir.canonicalize()?;
    let entry = dir.join(&script.entry_file).canonicalize()?;
    if !entry.starts_with(&base) {
        return Err(AppError::validation("script entry escapes script directory"));
    }
    Ok(entry.to_string_lossy().to_string())
}

fn read_script_content(dir: &Path, entry_file: &str) -> Result<String, AppError> {
    let base = dir.canonicalize()?;
    let entry = dir.join(entry_file).canonicalize()?;
    if !entry.starts_with(base) {
        return Err(AppError::validation("script entry escapes script directory"));
    }
    Ok(fs::read_to_string(entry)?)
}
