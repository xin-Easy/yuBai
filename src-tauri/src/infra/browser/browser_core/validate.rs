use crate::{
    domain::{browser::BrowserCoreValidateResult, paths::resolve_app_path},
};
use std::path::{Path, PathBuf};

use super::shared::{executable_candidates, invalid};

pub fn validate_core_path(app_root: &Path, core_path: &str) -> BrowserCoreValidateResult {
    let core_path = core_path.trim();
    if core_path.is_empty() {
        return invalid("Path cannot be empty");
    }

    let base = resolve_app_path(app_root, core_path);
    if !base.exists() {
        return invalid(format!("Path does not exist: {}", base.to_string_lossy()));
    }

    match find_core_executable(&base) {
        Some(exe) => BrowserCoreValidateResult {
            valid: true,
            message: format!("Valid browser executable: {}", exe.to_string_lossy()),
            executable_path: Some(exe.to_string_lossy().to_string()),
        },
        None => invalid(format!(
            "No browser executable found. Candidates: {}",
            executable_candidates().join(", ")
        )),
    }
}

pub fn find_core_executable(base: &Path) -> Option<PathBuf> {
    if base.is_file() {
        let base_name = base.file_name()?.to_string_lossy().to_lowercase();
        for candidate in executable_candidates() {
            let candidate_name = Path::new(&candidate)
                .file_name()
                .map(|name| name.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if base_name == candidate_name {
                return Some(base.to_path_buf());
            }
        }
        return None;
    }

    for candidate in executable_candidates() {
        let path = base.join(Path::new(&candidate));
        if path.is_file() {
            return Some(path);
        }
    }

    None
}
