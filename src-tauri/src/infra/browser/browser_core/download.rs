use crate::{
    app::state::AppState,
    domain::{
        browser::{BrowserCoreDownloadInput, BrowserCoreDownloadOption, BrowserCoreInput},
        config::Config,
        text::non_empty_ref,
    },
};
use reqwest::blocking::Client;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    time::Duration,
};
use tauri::{AppHandle, Manager};
use tempfile::NamedTempFile;
use uuid::Uuid;
use zip::read::root_dir_common_filter;
use zip::ZipArchive;

use super::{
    shared::{
        cft_platform, emit_progress, rewrite_download_url, sanitize_core_folder_name,
        resolve_source_config, KnownGoodResponse, LastKnownGoodResponse,
    },
    validate::validate_core_path,
};

pub fn download_options_with_config(
    limit: usize,
    config: Option<&Config>,
) -> Result<Vec<BrowserCoreDownloadOption>, String> {
    let platform = cft_platform()?;
    let source = resolve_source_config(config);
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("yubai/0.1 ChromeCoreManager")
        .build()
        .map_err(|err| format!("failed to create http client: {err}"))?;

    let mut options = Vec::new();
    let mut last_known: Option<LastKnownGoodResponse> = None;
    for url in &source.last_known_good_urls {
        if let Ok(resp) = client
            .get(*url)
            .send()
            .and_then(|resp| resp.error_for_status())
            .and_then(|resp| resp.json::<LastKnownGoodResponse>())
        {
            last_known = Some(resp);
            break;
        }
    }

    if let Some(last_known) = last_known {
        for (channel, item) in last_known.channels {
            if let Some(download) = item
                .downloads
                .chrome
                .into_iter()
                .find(|download| download.platform == platform)
            {
                options.push(BrowserCoreDownloadOption {
                    version: item.version,
                    channel,
                    platform: platform.clone(),
                    url: rewrite_download_url(&download.url, source.download_prefix),
                });
            }
        }
    }

    let mut known_good: Option<KnownGoodResponse> = None;
    for url in &source.known_good_urls {
        if let Ok(resp) = client
            .get(*url)
            .send()
            .and_then(|resp| resp.error_for_status())
            .and_then(|resp| resp.json::<KnownGoodResponse>())
        {
            known_good = Some(resp);
            break;
        }
    }

    let known_good =
        known_good.ok_or_else(|| "failed to fetch Chrome versions from all sources".to_string())?;

    let mut latest_by_major: BTreeMap<i32, BrowserCoreDownloadOption> = BTreeMap::new();
    for item in known_good.versions {
        let Some(download) = item
            .downloads
            .chrome
            .into_iter()
            .find(|download| download.platform == platform)
        else {
            continue;
        };
        let major = item
            .version
            .split('.')
            .next()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        latest_by_major.insert(
            major,
            BrowserCoreDownloadOption {
                version: item.version,
                channel: "known-good".to_string(),
                platform: platform.clone(),
                url: rewrite_download_url(&download.url, source.download_prefix),
            },
        );
    }

    for option in latest_by_major.into_iter().rev().map(|(_, option)| option) {
        if !options.iter().any(|item| item.version == option.version) {
            options.push(option);
        }
        if options.len() >= limit.max(1) {
            break;
        }
    }

    Ok(options)
}

pub fn start_download_blocking(app: AppHandle, input: BrowserCoreDownloadInput) -> Result<(), String> {
    if input.core_name.trim().is_empty() {
        return Err("Core name cannot be empty".to_string());
    }
    if input.url.trim().is_empty() {
        return Err("Download URL cannot be empty".to_string());
    }

    if let Err(err) = download_and_register(&app, input) {
        emit_progress(&app, "error", 0, format!("Download failed: {err}"));
    }
    Ok(())
}

fn download_and_register(app: &AppHandle, input: BrowserCoreDownloadInput) -> Result<(), String> {
    let state = app.state::<AppState>();
    let app_root = state.app_root.clone();
    let core_name = sanitize_core_folder_name(&input.core_name);
    let display_name = input.core_name.trim().to_string();

    let chrome_dir = app_root.join("chrome");
    fs::create_dir_all(&chrome_dir).map_err(|err| format!("failed to create chrome dir: {err}"))?;

    let target_dir = chrome_dir.join(&core_name);
    if target_dir.exists() {
        return Err(format!(
            "target folder already exists: {}",
            target_dir.to_string_lossy()
        ));
    }

    let temp_file = NamedTempFile::new_in(&chrome_dir)
        .map_err(|err| format!("failed to create temp download file: {err}"))?;
    let temp_path = temp_file.path().to_path_buf();
    emit_progress(app, "downloading", 0, "Preparing download");
    download_zip(app, &input.url, &temp_path, input.proxy_config.as_deref())?;

    emit_progress(app, "extracting", 0, "Extracting browser core");
    if let Err(err) = extract_zip_strip_root(app, &temp_path, &target_dir) {
        let _ = temp_file.close();
        let _ = fs::remove_dir_all(&target_dir);
        return Err(err);
    }
    let _ = temp_file.close();

    let relative_path = format!("chrome/{}", core_name);
    let validation = validate_core_path(&app_root, &relative_path);
    if !validation.valid {
        let _ = fs::remove_dir_all(&target_dir);
        return Err(format!("extracted core is invalid: {}", validation.message));
    }

    let is_first_core = state
        .repositories
        .database
        .list_cores()
        .map(|cores| cores.is_empty())
        .unwrap_or(false);
    state.repositories.database.save_core(BrowserCoreInput {
        core_id: Some(format!("downloaded-chrome-{}", Uuid::new_v4())),
        core_name: display_name,
        core_path: relative_path,
        is_default: input.is_default || is_first_core,
    })?;

    emit_progress(app, "done", 100, "Browser core downloaded and registered");
    Ok(())
}

fn download_zip(
    app: &AppHandle,
    url: &str,
    target: &Path,
    proxy_config: Option<&str>,
) -> Result<(), String> {
    let config = app.state::<AppState>().config_snapshot().ok();
    let source = resolve_source_config(config.as_ref());
    let fallback_url = if url.starts_with(source.download_prefix) {
        Some(url.replacen(source.download_prefix, source.fallback_download_prefix, 1))
    } else if url.starts_with(source.fallback_download_prefix) {
        Some(url.replacen(source.fallback_download_prefix, source.download_prefix, 1))
    } else {
        None
    };

    match download_zip_from(app, url, target, proxy_config) {
        Ok(()) => Ok(()),
        Err(err) if fallback_url.as_ref().is_some() => {
            let fallback = fallback_url.as_deref().unwrap_or_default();
            emit_progress(
                app,
                "downloading",
                0,
                "Primary source failed, retrying from fallback source".to_string(),
            );
            download_zip_from(app, fallback, target, proxy_config)
                .map_err(|fallback_err| format!("{err}; fallback also failed: {fallback_err}"))
        }
        Err(err) => Err(err),
    }
}

fn download_zip_from(
    app: &AppHandle,
    url: &str,
    target: &Path,
    proxy_config: Option<&str>,
) -> Result<(), String> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(60 * 60))
        .user_agent("yubai/0.1 ChromeCoreDownloader");

    if let Some(proxy) = proxy_config.and_then(non_empty_ref) {
        if proxy != "__direct__" && proxy != "direct://" {
            if proxy != "__system__" {
                builder = builder.proxy(
                    reqwest::Proxy::all(proxy).map_err(|err| format!("invalid proxy: {err}"))?,
                );
            }
        } else {
            builder = builder.no_proxy();
        }
    }

    let client = builder
        .build()
        .map_err(|err| format!("failed to create http client: {err}"))?;
    let mut resp = client
        .get(url)
        .send()
        .and_then(|resp| resp.error_for_status())
        .map_err(|err| format!("request failed: {err}"))?;
    let total = resp.content_length().unwrap_or(0);
    let mut file = File::create(target).map_err(|err| format!("failed to create temp file: {err}"))?;
    let mut downloaded = 0u64;
    let mut buf = [0u8; 1024 * 256];
    loop {
        let read = resp
            .read(&mut buf)
            .map_err(|err| format!("failed to read download: {err}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buf[..read])
            .map_err(|err| format!("failed to write download: {err}"))?;
        downloaded += read as u64;
        if total > 0 {
            let progress = ((downloaded as f64 / total as f64) * 100.0).round() as i32;
            emit_progress(
                app,
                "downloading",
                progress.clamp(0, 99),
                format!(
                    "Downloaded {:.1} MB / {:.1} MB",
                    downloaded as f64 / 1024.0 / 1024.0,
                    total as f64 / 1024.0 / 1024.0
                ),
            );
        }
    }
    Ok(())
}

fn extract_zip_strip_root(app: &AppHandle, zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|err| format!("failed to open zip: {err}"))?;
    let mut archive = ZipArchive::new(file).map_err(|err| format!("failed to read zip: {err}"))?;
    if archive.is_empty() {
        return Err("zip archive is empty".to_string());
    }

    fs::create_dir_all(dest).map_err(|err| format!("failed to create target dir: {err}"))?;
    emit_progress(app, "extracting", 5, "Reading archive");
    archive
        .extract_unwrapped_root_dir(dest, root_dir_common_filter)
        .map_err(|err| format!("failed to extract zip: {err}"))?;
    emit_progress(app, "extracting", 100, "Archive extracted");
    Ok(())
}
