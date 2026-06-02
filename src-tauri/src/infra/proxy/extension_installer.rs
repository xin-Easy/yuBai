use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    error::AppError,
    infra::logging,
};
use super::extension::ExtensionProxyConfig;

const EXTENSION_DIR: &str = "proxy-checker";
const REQUIRED_FILES: &[&str] = &[
    "manifest.json",
    "service_worker.js",
    "popup.html",
    "popup.css",
    "popup.js",
];

pub fn ensure_installed(app_root: &Path) -> Result<PathBuf, AppError> {
    let fallback_dir = app_root.join("resources").join("extensions-runtime");
    ensure_installed_with_config(app_root, &fallback_dir, &ExtensionProxyConfig::default())
}

pub fn ensure_installed_with_config(
    app_root: &Path,
    profile_user_data_dir: &Path,
    proxy_config: &ExtensionProxyConfig,
) -> Result<PathBuf, AppError> {
    let source_dir = source_extension_dir(app_root)?;
    let target_dir = profile_user_data_dir
        .join("extensions")
        .join(EXTENSION_DIR);

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(AppError::from)?;
    }
    fs::create_dir_all(&target_dir).map_err(AppError::from)?;

    copy_dir_contents(&source_dir, &target_dir)?;
    write_proxy_config(&target_dir, proxy_config)?;
    validate_extension_dir(&target_dir)?;
    logging::log_target(
        "info",
        "proxy.extension",
        "proxy checker extension installed",
        format!(
            "targetDir={} enabled={} scheme={} host={} port={} source={}",
            target_dir.to_string_lossy(),
            proxy_config.enabled,
            proxy_config.scheme,
            proxy_config.host,
            proxy_config.port,
            proxy_config.source,
        ),
    );
    Ok(target_dir)
}

fn source_extension_dir(app_root: &Path) -> Result<PathBuf, AppError> {
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("extensions")
            .join(EXTENSION_DIR),
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| app_root.to_path_buf())
            .join("resources")
            .join("extensions")
            .join(EXTENSION_DIR),
    ];

    candidates
        .into_iter()
        .find(|path| validate_extension_dir(path).is_ok())
        .ok_or_else(|| AppError::not_found("proxy checker extension resources not found"))
}

fn validate_extension_dir(dir: &Path) -> Result<(), AppError> {
    for file in REQUIRED_FILES {
        let path = dir.join(file);
        if !path.is_file() {
            return Err(AppError::not_found(format!(
                "missing proxy checker extension file: {}",
                path.to_string_lossy()
            )));
        }
    }
    Ok(())
}

fn copy_dir_contents(source: &Path, target: &Path) -> Result<(), AppError> {
    for entry in fs::read_dir(source).map_err(AppError::from)? {
        let entry = entry.map_err(AppError::from)?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().map_err(AppError::from)?;

        if file_type.is_dir() {
            fs::create_dir_all(&target_path).map_err(AppError::from)?;
            copy_dir_contents(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path).map_err(AppError::from)?;
        }
    }
    Ok(())
}

fn write_proxy_config(dir: &Path, proxy_config: &ExtensionProxyConfig) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(proxy_config)?;
    fs::write(dir.join("proxy_config.json"), json).map_err(AppError::from)
}
