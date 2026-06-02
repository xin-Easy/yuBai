use crate::{domain::config::Config, error::AppError, infra::download_source};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::NamedTempFile;
use zip::ZipArchive;

const RELEASE_API: &str = "https://api.github.com/repos/MetaCubeX/mihomo/releases/latest";

#[derive(Debug, Deserialize)]
struct Release {
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn ensure_binary(app_root: &Path, config: &Config) -> Result<PathBuf, AppError> {
    let runtime_dir = runtime_dir(app_root);
    fs::create_dir_all(&runtime_dir)?;

    let binary_path = installed_binary_path(app_root);
    if binary_path.is_file() {
        return Ok(binary_path);
    }

    download_latest_release(&runtime_dir, &binary_path, config)?;
    Ok(binary_path)
}

pub fn download_binary(app_root: &Path, config: &Config) -> Result<PathBuf, AppError> {
    let runtime_dir = runtime_dir(app_root);
    fs::create_dir_all(&runtime_dir)?;

    let binary_path = installed_binary_path(app_root);
    if binary_path.is_file() {
        let _ = fs::remove_file(&binary_path);
    }

    download_latest_release(&runtime_dir, &binary_path, config)?;
    Ok(binary_path)
}

pub fn installed_binary_path(app_root: &Path) -> PathBuf {
    runtime_dir(app_root).join(binary_name())
}

pub fn write_runtime_config(
    workdir: &Path,
    port: u16,
    proxy_yaml: &str,
) -> Result<PathBuf, AppError> {
    let config_path = workdir.join("mihomo-config.yaml");
    let raw = format!(
        "mixed-port: {port}\nallow-lan: false\nmode: rule\nlog-level: warning\nipv6: false\nproxies:\n{proxy_yaml}\nproxy-groups:\n  - name: Proxy\n    type: select\n    proxies:\n      - proxy-out\nrules:\n  - MATCH,Proxy\n"
    );
    fs::write(&config_path, raw)?;
    Ok(config_path)
}

fn download_latest_release(
    runtime_dir: &Path,
    binary_path: &Path,
    config: &Config,
) -> Result<(), AppError> {
    let client = client()?;
    let prefer_mirror = download_source::prefer_mirror(Some(config));
    let source_label = download_source::source_label(Some(config));

    let release = client
        .get(RELEASE_API)
        .send()
        .map_err(|err| {
            AppError::other(format!(
                "failed to request mihomo release metadata: {err}"
            ))
        })?
        .error_for_status()
        .map_err(|err| {
            AppError::other(format!(
                "mihomo release metadata request returned error: {err}"
            ))
        })?
        .json::<Release>()
        .map_err(|err| {
            AppError::other(format!(
                "failed to parse mihomo release metadata: {err}"
            ))
        })?;

    let asset = select_asset(&release).ok_or_else(|| {
        AppError::not_found(format!(
            "mihomo asset not found for {} via {source_label} source",
            platform_prefix()
        ))
    })?;
    download_asset(
        &client,
        runtime_dir,
        binary_path,
        asset,
        prefer_mirror,
        source_label,
    )?;
    Ok(())
}

fn select_asset<'a>(release: &'a Release) -> Option<&'a ReleaseAsset> {
    let prefix = format!("mihomo-{}", platform_prefix());
    let ext = if cfg!(target_os = "windows") {
        ".zip"
    } else {
        ".gz"
    };

    release
        .assets
        .iter()
        .filter(|asset| asset.name.starts_with(&prefix) && asset.name.ends_with(ext))
        .find(|asset| !asset.name.contains("-compatible-"))
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|asset| asset.name.starts_with(&prefix) && asset.name.ends_with(ext))
        })
}

fn download_asset(
    client: &Client,
    runtime_dir: &Path,
    binary_path: &Path,
    asset: &ReleaseAsset,
    prefer_mirror: bool,
    _source_label: &str,
) -> Result<(), AppError> {
    let mirror_urls = if prefer_mirror {
        vec![
            format!("https://ghproxy.net/{}", asset.browser_download_url),
            asset
                .browser_download_url
                .replace( "https://kkgithub.com","https://github.com"),
        ]
    } else {
        vec![]
    };

    let mut last_err = None;

    for url in &mirror_urls {
        match fetch_asset_bytes(client, url) {
            Ok(bytes) => {
                return install_asset(client, runtime_dir, binary_path, asset, bytes, url);
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    let official_url = &asset.browser_download_url;
    match fetch_asset_bytes(client, official_url) {
        Ok(bytes) => install_asset(client, runtime_dir, binary_path, asset, bytes, official_url),
        Err(e) => Err(last_err.unwrap_or(e)),
    }
}

fn fetch_asset_bytes(client: &Client, url: &str) -> Result<Vec<u8>, AppError> {
    client
        .get(url)
        .send()
        .map_err(|err| AppError::other(format!("download failed: {err}")))?
        .error_for_status()
        .map_err(|err| AppError::other(format!("download returned error: {err}")))?
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|err| AppError::other(format!("failed to read bytes: {err}")))
}

fn install_asset(
    _client: &Client,
    runtime_dir: &Path,
    binary_path: &Path,
    asset: &ReleaseAsset,
    bytes: Vec<u8>,
    download_url: &str,
) -> Result<(), AppError> {
    if asset.name.ends_with(".zip") {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor)?;
        extract_zip_binary(&mut archive, runtime_dir, binary_path)?;
    } else if asset.name.ends_with(".gz") {
        let mut decoder = GzDecoder::new(Cursor::new(bytes.to_vec()));
        let temp = NamedTempFile::new_in(runtime_dir)?;
        let temp_path = temp.path().to_path_buf();
        let mut output = File::create(&temp_path)?;
        std::io::copy(&mut decoder, &mut output)?;
        fs::rename(temp_path, binary_path)?;
    } else {
        return Err(AppError::validation(format!(
            "unsupported mihomo asset format: {}",
            asset.name,
        )));
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(binary_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(binary_path, permissions)?;
    }

    fs::write(
        runtime_dir.join("version.txt"),
        &release_version_line(asset, download_url),
    )?;
    Ok(())
}

fn extract_zip_binary<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    runtime_dir: &Path,
    binary_path: &Path,
) -> Result<(), AppError> {
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if file_name != "mihomo.exe" && file_name != "mihomo" {
            continue;
        }

        let temp = NamedTempFile::new_in(runtime_dir)?;
        let temp_path = temp.path().to_path_buf();
        let mut output = File::create(&temp_path)?;
        std::io::copy(&mut entry, &mut output)?;
        fs::rename(temp_path, binary_path)?;
        return Ok(());
    }

    Err(AppError::not_found(
        "mihomo binary not found in downloaded archive",
    ))
}

fn client() -> Result<Client, AppError> {
    Client::builder()
        .user_agent("yubai/0.1.0")
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(AppError::from)
}

fn platform_prefix() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}-{arch}")
}

fn runtime_dir(app_root: &Path) -> PathBuf {
    app_root.join("runtime").join("mihomo")
}

fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "mihomo.exe"
    } else {
        "mihomo"
    }
}

fn release_version_line(asset: &ReleaseAsset, download_url: &str) -> String {
    format!("{}\n{}\n", asset.name, download_url)
}
