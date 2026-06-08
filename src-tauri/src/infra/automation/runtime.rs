use crate::{
    app::state::AppState,
    domain::config::Config,
    error::AppError,
    infra::{automation::util::quiet_command, download_source},
};
use chrono::Utc;
use reqwest::blocking::Client;
use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use tar::Archive;
use tauri::{AppHandle, Emitter, Manager};
use xz2::read::XzDecoder;
use zip::ZipArchive;

use super::types::*;
use super::util::*;

const NODE_OFFICIAL_DIST_BASE: &str = "https://nodejs.org/dist";
const NODE_NPMMIRROR_DIST_BASE: &str = "https://npmmirror.com/mirrors/node";

#[allow(dead_code)]
pub fn install_runtime(state: &AppState) -> Result<AutomationRuntimeState, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let config = state.config_snapshot().map_err(AppError::from)?;
    ensure_bundled_node(&paths, &config)?;
    write_runtime_package(&paths.runtime)?;
    write_runner(&paths.runner)?;
    ensure_playwright_installed(&paths, &config)?;
    super::settings::state_get(state)
}

pub fn install_runtime_with_events(app: AppHandle) -> Result<AutomationRuntimeState, AppError> {
    emit_runtime_progress(&app, "preparing", "Preparing automation runtime", 5);
    let state = app.state::<AppState>();
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let config = state.config_snapshot().map_err(AppError::from)?;
    emit_runtime_progress(&app, "downloading_node", "Checking bundled Node.js", 15);
    ensure_bundled_node_with_events(&paths, &config, Some(&app))?;
    emit_runtime_progress(&app, "writing_runner", "Writing automation runner", 55);
    write_runtime_package(&paths.runtime)?;
    write_runner(&paths.runner)?;
    emit_runtime_progress(&app, "installing_playwright", "Installing playwright-core", 70);
    ensure_playwright_installed_with_events(&paths, &config, Some(&app))?;
    emit_runtime_progress(&app, "ready", "Automation runtime is ready", 100);
    super::settings::state_get(state.inner())
}

pub fn emit_runtime_progress(app: &AppHandle, phase: &str, message: &str, progress: i32) {
    let _ = app.emit(
        "automation:runtime:progress",
        AutomationRuntimeProgressEvent {
            phase: phase.to_string(),
            message: message.to_string(),
            progress,
        },
    );
}

pub fn write_runner(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, RUNNER_CJS)?;
    Ok(())
}

pub fn write_runtime_package(runtime_dir: &Path) -> Result<(), AppError> {
    let package_json = runtime_dir.join("package.json");
    fs::write(
        package_json,
        r#"{"private":true,"dependencies":{"playwright-core":"^1.59.0","typescript":"^5.9.3"}}"#,
    )?;
    Ok(())
}

pub fn runtime_version_label(config: &Config) -> String {
    format!(
        "node-{}-playwright-core",
        normalize_node_version(&config.automation.node.version)
    )
}

pub fn detect_playwright_version(node: &Path, runtime_dir: &Path) -> Result<String, AppError> {
    let mut command = quiet_command(node);
    command
        .args([
            "-e",
            "const p=require('playwright-core/package.json'); process.stdout.write(p.version)",
        ])
        .current_dir(runtime_dir)
        .env("NODE_PATH", runtime_dir.join("node_modules"));
    command_stdout(&mut command)
}

pub fn detect_typescript_version(node: &Path, runtime_dir: &Path) -> Result<String, AppError> {
    let mut command = quiet_command(node);
    command
        .args([
            "-e",
            "const p=require('typescript/package.json'); process.stdout.write(p.version)",
        ])
        .current_dir(runtime_dir)
        .env("NODE_PATH", runtime_dir.join("node_modules"));
    command_stdout(&mut command)
}

#[allow(dead_code)]
fn ensure_playwright_installed(paths: &AutomationPaths, config: &Config) -> Result<(), AppError> {
    ensure_playwright_installed_with_events(paths, config, None)
}

fn ensure_playwright_installed_with_events(
    paths: &AutomationPaths,
    config: &Config,
    app: Option<&AppHandle>,
) -> Result<(), AppError> {
    if paths
        .runtime
        .join("node_modules")
        .join("playwright-core")
        .join("package.json")
        .is_file()
        && paths
            .runtime
            .join("node_modules")
            .join("typescript")
            .join("package.json")
            .is_file()
    {
        return Ok(());
    }

    let node_paths = bundled_node_paths(paths, config)?
        .ok_or_else(|| AppError::validation("bundled Node.js is not installed"))?;
    if let Some(app) = app {
        emit_runtime_progress(app, "installing_playwright", "Running npm install", 75);
    }
    let output = npm_install_command(&node_paths, config)
        .current_dir(&paths.runtime)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn npm_install_command(node_paths: &NodePackage, config: &Config) -> Command {
    if node_paths
        .npm_path
        .file_name()
        .and_then(|name| name.to_str())
        == Some("npm-cli.js")
    {
        let mut command = quiet_command(&node_paths.executable_path);
        command.arg(&node_paths.npm_path);
        command.arg("install");
        command.arg("--omit=dev");
        append_npm_registry(&mut command, config);
        command
    } else {
        let mut command = quiet_command(&node_paths.npm_path);
        command.arg("install");
        command.arg("--omit=dev");
        append_npm_registry(&mut command, config);
        command
    }
}

fn append_npm_registry(command: &mut Command, config: &Config) {
    if npm_prefer_mirror(config) {
        command.arg("--registry=https://registry.npmmirror.com");
    }
}

fn npm_prefer_mirror(config: &Config) -> bool {
    match config
        .automation
        .node
        .download_source
        .trim()
        .to_lowercase()
        .as_str()
    {
        "official" => false,
        "npmmirror" | "mirror" => true,
        _ => download_source::prefer_mirror(Some(config)),
    }
}

#[allow(dead_code)]
pub fn ensure_bundled_node(paths: &AutomationPaths, config: &Config) -> Result<NodePackage, AppError> {
    ensure_bundled_node_with_events(paths, config, None)
}

fn ensure_bundled_node_with_events(
    paths: &AutomationPaths,
    config: &Config,
    app: Option<&AppHandle>,
) -> Result<NodePackage, AppError> {
    let package = node_package(paths, config)?;
    if package.executable_path.is_file() && package.npm_path.is_file() {
        write_node_manifest(paths, &package)?;
        return Ok(package);
    }

    if package.version_dir.exists() {
        fs::remove_dir_all(&package.version_dir)?;
    }
    fs::create_dir_all(&paths.node_downloads)?;
    fs::create_dir_all(&paths.node_versions)?;

    if let Some(app) = app {
        emit_runtime_progress(app, "downloading_node", "Downloading bundled Node.js", 25);
    }
    let download_urls = node_download_urls(&package.url);
    download_node_archive(&download_urls, &package.archive_path)?;
    if let Some(app) = app {
        emit_runtime_progress(app, "extracting_node", "Extracting bundled Node.js", 45);
    }
    extract_node_archive(&package.archive_path, &paths.node_versions)?;

    if !package.executable_path.is_file() {
        return Err(AppError::other(format!(
            "Node executable not found after extract: {}",
            package.executable_path.to_string_lossy()
        )));
    }
    let npm_path = resolve_installed_npm_path(&package.version_dir).ok_or_else(|| {
        AppError::other(format!(
            "npm executable not found after extract: {}",
            package.version_dir.to_string_lossy()
        ))
    })?;
    let package = NodePackage {
        npm_path,
        ..package
    };

    if !package.npm_path.is_file() {
        return Err(AppError::other(format!(
            "npm executable not found after extract: {}",
            package.npm_path.to_string_lossy()
        )));
    }
    write_node_manifest(paths, &package)?;
    Ok(package)
}

pub fn bundled_node_paths(
    paths: &AutomationPaths,
    config: &Config,
) -> Result<Option<NodePackage>, AppError> {
    let package = node_package(paths, config)?;
    if package.executable_path.is_file() && package.npm_path.is_file() {
        Ok(Some(package))
    } else {
        Ok(None)
    }
}

fn node_package(paths: &AutomationPaths, config: &Config) -> Result<NodePackage, AppError> {
    let version = normalize_node_version(&config.automation.node.version);
    let platform = node_platform()?;
    let archive_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.xz"
    };
    let package_name = format!("node-v{version}-{platform}");
    let archive_name = format!("{package_name}.{archive_ext}");
    let base_url = node_download_base_url(config);
    let url = format!("{base_url}/v{version}/{archive_name}");
    let version_dir = paths.node_versions.join(&package_name);
    let executable_path = if cfg!(target_os = "windows") {
        version_dir.join("node.exe")
    } else {
        version_dir.join("bin").join("node")
    };
    let npm_path = resolve_npm_path(&version_dir);

    Ok(NodePackage {
        version,
        package_name,
        url,
        version_dir,
        archive_path: paths.node_downloads.join(archive_name),
        executable_path,
        npm_path,
    })
}

fn node_download_base_url(config: &Config) -> &'static str {
    match config
        .automation
        .node
        .download_source
        .trim()
        .to_lowercase()
        .as_str()
    {
        "official" => NODE_OFFICIAL_DIST_BASE,
        "npmmirror" | "mirror" => NODE_NPMMIRROR_DIST_BASE,
        _ => {
            if download_source::prefer_mirror(Some(config)) {
                NODE_NPMMIRROR_DIST_BASE
            } else {
                NODE_OFFICIAL_DIST_BASE
            }
        }
    }
}

fn node_download_urls(primary_url: &str) -> Vec<String> {
    let mut urls = vec![primary_url.to_string()];
    let fallback_url = if primary_url.starts_with(NODE_NPMMIRROR_DIST_BASE) {
        Some(primary_url.replacen(NODE_NPMMIRROR_DIST_BASE, NODE_OFFICIAL_DIST_BASE, 1))
    } else if primary_url.starts_with(NODE_OFFICIAL_DIST_BASE) {
        Some(primary_url.replacen(NODE_OFFICIAL_DIST_BASE, NODE_NPMMIRROR_DIST_BASE, 1))
    } else {
        None
    };

    if let Some(fallback_url) = fallback_url {
        if fallback_url != primary_url {
            urls.push(fallback_url);
        }
    }
    urls
}

fn normalize_node_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

fn resolve_npm_path(version_dir: &Path) -> PathBuf {
    resolve_installed_npm_path(version_dir).unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            version_dir.join("npm.cmd")
        } else {
            version_dir.join("bin").join("npm")
        }
    })
}

fn resolve_installed_npm_path(version_dir: &Path) -> Option<PathBuf> {
    let npm_bin_dir = version_dir
        .join("node_modules")
        .join("npm")
        .join("bin");
    let lib_npm_bin_dir = version_dir
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("bin");
    let search_dirs = if cfg!(target_os = "windows") {
        vec![version_dir.to_path_buf(), npm_bin_dir.clone()]
    } else {
        vec![version_dir.join("bin"), lib_npm_bin_dir.clone(), npm_bin_dir.clone()]
    };

    let paths = std::env::join_paths(search_dirs.iter()).ok()?;
    let binary_names = if cfg!(target_os = "windows") {
        ["npm.cmd", "npm.exe", "npm"]
    } else {
        ["npm", "npm-cli.js", "npm.cmd"]
    };

    binary_names
        .iter()
        .find_map(|name| which::which_in(name, Some(&paths), version_dir).ok())
        .or_else(|| {
            [
                npm_bin_dir.join("npm-cli.js"),
                lib_npm_bin_dir.join("npm-cli.js"),
            ]
            .into_iter()
            .find(|path| path.is_file())
        })
}

fn node_platform() -> Result<&'static str, AppError> {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            Ok("win-x64")
        } else if cfg!(target_arch = "aarch64") {
            Ok("win-arm64")
        } else {
            Err(AppError::validation("unsupported Windows architecture"))
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "x86_64") {
            Ok("darwin-x64")
        } else if cfg!(target_arch = "aarch64") {
            Ok("darwin-arm64")
        } else {
            Err(AppError::validation("unsupported macOS architecture"))
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            Ok("linux-x64")
        } else if cfg!(target_arch = "aarch64") {
            Ok("linux-arm64")
        } else {
            Err(AppError::validation("unsupported Linux architecture"))
        }
    } else {
        Err(AppError::validation("unsupported operating system"))
    }
}

fn download_node_archive(urls: &[String], target: &Path) -> Result<(), AppError> {
    if target.is_file() {
        match validate_node_archive(target) {
            Ok(()) => return Ok(()),
            Err(_) => {
                let _ = fs::remove_file(target);
            }
        }
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(600))
        .build()?;
    let temp_path = target.with_file_name(format!(
        "{}.download",
        target
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default()
    ));
    let mut last_err = None;

    for url in urls {
        let _ = fs::remove_file(&temp_path);
        match download_node_archive_from(&client, url, &temp_path)
            .and_then(|_| validate_node_archive(&temp_path))
        {
            Ok(()) => {
                fs::rename(&temp_path, target)?;
                return Ok(());
            }
            Err(err) => {
                let _ = fs::remove_file(&temp_path);
                last_err = Some(err);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::other("Node download failed")))
}

fn download_node_archive_from(client: &Client, url: &str, target: &Path) -> Result<(), AppError> {
    let mut response = client
        .get(url)
        .send()
        .map_err(|err| AppError::other(format!("Node download failed from {url}: {err}")))?
        .error_for_status()
        .map_err(|err| AppError::other(format!("Node download returned error from {url}: {err}")))?;
    let mut file = File::create(target)?;
    io::copy(&mut response, &mut file)?;
    Ok(())
}

fn extract_node_archive(archive_path: &Path, target_dir: &Path) -> Result<(), AppError> {
    if archive_path.extension().and_then(|value| value.to_str()) == Some("zip") {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file).map_err(|err| {
            AppError::other(format!(
                "invalid Node zip archive {}: {err}",
                archive_path.to_string_lossy()
            ))
        })?;
        crate::infra::archive::extract_zip_safely(&mut archive, target_dir)?;
        return Ok(());
    }

    let file = File::open(archive_path)?;
    let decoder = XzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive
        .unpack(target_dir)
        .map_err(|err| AppError::other(format!("failed to extract Node archive: {err}")))?;
    Ok(())
}

fn validate_node_archive(archive_path: &Path) -> Result<(), AppError> {
    let mut file = File::open(archive_path)?;
    let mut header = [0u8; 6];
    let read = file.read(&mut header)?;
    file.seek(SeekFrom::Start(0))?;

    if archive_path.extension().and_then(|value| value.to_str()) == Some("zip") {
        if read < 4 || &header[..4] != b"PK\x03\x04" {
            return Err(AppError::other(format!(
                "downloaded Node archive is not a zip file: {}",
                archive_preview(archive_path)?
            )));
        }
        ZipArchive::new(file).map(|_| ()).map_err(|err| {
            AppError::other(format!(
                "invalid Node zip archive {}: {err}",
                archive_path.to_string_lossy()
            ))
        })?;
    } else if archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".tar.xz"))
        .unwrap_or(false)
    {
        if read < 6 || header != [0xfd, b'7', b'z', b'X', b'Z', 0x00] {
            return Err(AppError::other(format!(
                "downloaded Node archive is not an xz file: {}",
                archive_preview(archive_path)?
            )));
        }
    }

    Ok(())
}

fn archive_preview(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path)?;
    let mut bytes = [0u8; 120];
    let read = file.read(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes[..read])
        .replace('\r', " ")
        .replace('\n', " "))
}

fn write_node_manifest(paths: &AutomationPaths, package: &NodePackage) -> Result<(), AppError> {
    let manifest = NodeInstallManifest {
        version: package.version.clone(),
        package_name: package.package_name.clone(),
        download_url: package.url.clone(),
        installed_at: Utc::now().to_rfc3339(),
    };
    write_json_pretty(&paths.node_current_version, &manifest)
}
