use crate::error::AppError;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    path::Path,
    process::Command,
};

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn non_empty(value: String, message: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::validation(message))
    } else {
        Ok(trimmed.to_string())
    }
}

pub fn command_stdout(command: &mut Command) -> Result<String, AppError> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(AppError::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
