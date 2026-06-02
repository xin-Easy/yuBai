use crate::{
    app::state::AppState,
    domain::{
        browser::{BrowserCoreDownloadProgress, BrowserCoreValidateResult},
        config::Config,
    },
    infra::download_source,
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

pub const CFT_KNOWN_GOOD_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";
pub const CFT_LAST_KNOWN_GOOD_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";
pub const MIRROR_KNOWN_GOOD_URL: &str =
    "https://registry.npmmirror.com/-/binary/chrome-for-testing/known-good-versions-with-downloads.json";
pub const MIRROR_LAST_KNOWN_GOOD_URL: &str =
    "https://registry.npmmirror.com/-/binary/chrome-for-testing/last-known-good-versions-with-downloads.json";
pub const OFFICIAL_DOWNLOAD_PREFIX: &str = "https://storage.googleapis.com/chrome-for-testing-public/";
pub const MIRROR_DOWNLOAD_PREFIX: &str = "https://cdn.npmmirror.com/binaries/chrome-for-testing/";

pub struct SourceConfig {
    pub known_good_urls: [&'static str; 2],
    pub last_known_good_urls: [&'static str; 2],
    pub download_prefix: &'static str,
    pub fallback_download_prefix: &'static str,
}

pub fn resolve_source_config(config: Option<&Config>) -> SourceConfig {
    let source = config
        .map(|config| config.browser.download_source.trim().to_lowercase())
        .unwrap_or_else(|| "auto".to_string());
    let prefer_mirror = match source.as_str() {
        "official" => false,
        "npmmirror" | "mirror" => true,
        _ => download_source::prefer_mirror(config),
    };

    if prefer_mirror {
        SourceConfig {
            known_good_urls: [MIRROR_KNOWN_GOOD_URL, CFT_KNOWN_GOOD_URL],
            last_known_good_urls: [MIRROR_LAST_KNOWN_GOOD_URL, CFT_LAST_KNOWN_GOOD_URL],
            download_prefix: MIRROR_DOWNLOAD_PREFIX,
            fallback_download_prefix: OFFICIAL_DOWNLOAD_PREFIX,
        }
    } else {
        SourceConfig {
            known_good_urls: [CFT_KNOWN_GOOD_URL, MIRROR_KNOWN_GOOD_URL],
            last_known_good_urls: [CFT_LAST_KNOWN_GOOD_URL, MIRROR_LAST_KNOWN_GOOD_URL],
            download_prefix: OFFICIAL_DOWNLOAD_PREFIX,
            fallback_download_prefix: MIRROR_DOWNLOAD_PREFIX,
        }
    }
}

pub fn executable_candidates() -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!["chrome.exe".to_string(), "chromium.exe".to_string()]
    } else if cfg!(target_os = "macos") {
        vec![
            "Google Chrome.app/Contents/MacOS/Google Chrome".to_string(),
            "Chromium.app/Contents/MacOS/Chromium".to_string(),
            "chrome".to_string(),
            "chromium".to_string(),
        ]
    } else {
        vec![
            "chrome".to_string(),
            "chrome-bin".to_string(),
            "chromium".to_string(),
            "chromium-browser".to_string(),
        ]
    }
}

pub fn system_browser_paths() -> Vec<(String, PathBuf, String)> {
    let mut paths = Vec::new();
    if cfg!(target_os = "windows") {
        for (env_key, relative, name) in [
            ("PROGRAMFILES", "Google/Chrome/Application/chrome.exe", "Google Chrome"),
            (
                "PROGRAMFILES(X86)",
                "Google/Chrome/Application/chrome.exe",
                "Google Chrome x86",
            ),
            (
                "LOCALAPPDATA",
                "Google/Chrome/Application/chrome.exe",
                "Google Chrome User",
            ),
            ("PROGRAMFILES", "Google/Chrome Beta/Application/chrome.exe", "Chrome Beta"),
            ("PROGRAMFILES", "Google/Chrome Dev/Application/chrome.exe", "Chrome Dev"),
            ("LOCALAPPDATA", "Google/Chrome SxS/Application/chrome.exe", "Chrome Canary"),
        ] {
            if let Ok(root) = env::var(env_key) {
                paths.push((
                    name.to_string(),
                    PathBuf::from(root).join(relative),
                    env_key.to_string(),
                ));
            }
        }
    } else if cfg!(target_os = "macos") {
        paths.push((
            "Google Chrome".to_string(),
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            "Applications".to_string(),
        ));
        if let Ok(home) = env::var("HOME") {
            paths.push((
                "Google Chrome User".to_string(),
                PathBuf::from(home).join("Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                "HOME".to_string(),
            ));
        }
    } else {
        for path in [
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ] {
            paths.push((
                "System Chromium".to_string(),
                PathBuf::from(path),
                "system".to_string(),
            ));
        }
    }

    for exe in executable_candidates() {
        let Some(file_name) = Path::new(&exe).file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Ok(found) = which::which_all(file_name) {
            for path in found {
                paths.push(("PATH Chrome".to_string(), path, "PATH".to_string()));
            }
        }
    }

    paths
}

pub fn chrome_version_from_executable(path: &Path) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let escaped = path.display().to_string().replace('\'', "''");
        let script = format!("(Get-Item '{escaped}').VersionInfo.ProductVersion");
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !raw.is_empty() {
            return Some(raw);
        }
    }

    let output = Command::new(path).arg("--version").output().ok()?;
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    raw.split_whitespace()
        .find(|part| {
            part.chars()
                .next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        })
        .map(|part| part.to_string())
}

pub fn chrome_version_from_manifest(core_dir: &Path) -> Option<String> {
    let manifest = core_dir.join("manifest.json");
    if let Ok(raw) = fs::read_to_string(manifest) {
        let value = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
        if let Some(version) = value.get("version").and_then(|item| item.as_str()) {
            return Some(version.to_string());
        }
    }
    let entries = fs::read_dir(core_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(version) = name.strip_suffix(".manifest") {
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}

pub fn emit_progress(app: &AppHandle, phase: &str, progress: i32, message: impl Into<String>) {
    let _ = app.emit(
        "browser-core:download-progress",
        BrowserCoreDownloadProgress {
            phase: phase.to_string(),
            progress,
            message: message.into(),
        },
    );
}

pub fn cft_platform() -> Result<String, String> {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86") {
            Ok("win32".to_string())
        } else {
            Ok("win64".to_string())
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            Ok("mac-arm64".to_string())
        } else {
            Ok("mac-x64".to_string())
        }
    } else if cfg!(target_os = "linux") {
        Ok("linux64".to_string())
    } else {
        Err("current platform is not supported by Chrome for Testing".to_string())
    }
}

pub fn sanitize_core_folder_name(value: &str) -> String {
    let out = sanitize_filename::sanitize(value.trim());
    if out.is_empty() {
        format!("chrome-{}", Uuid::new_v4())
    } else {
        out
    }
}

pub fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

pub fn invalid(message: impl Into<String>) -> BrowserCoreValidateResult {
    BrowserCoreValidateResult {
        valid: false,
        message: message.into(),
        executable_path: None,
    }
}

#[derive(Debug, Deserialize)]
pub struct KnownGoodResponse {
    pub versions: Vec<KnownGoodVersion>,
}

#[derive(Debug, Deserialize)]
pub struct KnownGoodVersion {
    pub version: String,
    pub downloads: KnownGoodDownloads,
}

#[derive(Debug, Deserialize)]
pub struct KnownGoodDownloads {
    #[serde(default)]
    pub chrome: Vec<KnownGoodDownload>,
}

#[derive(Debug, Deserialize)]
pub struct KnownGoodDownload {
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct LastKnownGoodResponse {
    #[serde(rename = "channels")]
    pub channels: BTreeMap<String, LastKnownGoodChannel>,
}

#[derive(Debug, Deserialize)]
pub struct LastKnownGoodChannel {
    pub version: String,
    pub downloads: KnownGoodDownloads,
}

pub fn rewrite_download_url(url: &str, prefix: &str) -> String {
    if let Some(rest) = url.strip_prefix(OFFICIAL_DOWNLOAD_PREFIX) {
        format!("{prefix}{rest}")
    } else if let Some(rest) = url.strip_prefix(MIRROR_DOWNLOAD_PREFIX) {
        format!("{prefix}{rest}")
    } else {
        url.to_string()
    }
}

pub fn app_root(state: &AppState) -> &Path {
    &state.app_root
}
