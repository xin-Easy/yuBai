use crate::{
    app::state::AppState,
    domain::paths::resolve_app_path,
    error::AppError,
    infra::archive::extract_zip_safely,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

pub const BACKUP_FORMAT: &str = "yubai-full-backup";
pub const BACKUP_MANIFEST_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub format: String,
    pub manifest_version: i32,
    pub created_at: String,
    pub app_name: String,
    pub app_version: String,
    pub entries: Vec<BackupManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifestEntry {
    pub id: String,
    pub path: String,
    pub required: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupExportResult {
    pub zip_path: String,
    pub manifest: BackupManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupImportResult {
    pub zip_path: String,
    pub manifest: BackupManifest,
}

pub fn backup_manifest_template(app: &AppState) -> BackupManifest {
    let config = app.config_snapshot().unwrap_or_default();
    let entries = vec![
        BackupManifestEntry {
            id: "config_yaml".to_string(),
            path: "config.yaml".to_string(),
            required: true,
            exists: app_root_exists(&app.app_root, "config.yaml"),
        },
        BackupManifestEntry {
            id: "database_main".to_string(),
            path: config.database.sqlite.path.clone(),
            required: true,
            exists: app_root_exists(&app.app_root, &config.database.sqlite.path),
        },
        BackupManifestEntry {
            id: "data_dir".to_string(),
            path: "data".to_string(),
            required: false,
            exists: app_root_exists(&app.app_root, "data"),
        },
    ];
    BackupManifest {
        format: BACKUP_FORMAT.to_string(),
        manifest_version: BACKUP_MANIFEST_VERSION,
        created_at: Utc::now().to_rfc3339(),
        app_name: config.app.name,
        app_version: app.app_version.clone(),
        entries,
    }
}

pub fn export_backup(app: &AppState, zip_path: &str) -> Result<BackupExportResult, AppError> {
    let zip_path = normalize_output_path(&app.app_root, zip_path);
    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let manifest = backup_manifest_template(app);
    write_backup_zip(&zip_path, &manifest, app)?;
    Ok(BackupExportResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        manifest,
    })
}

pub fn import_backup(app: &AppState, zip_path: &str) -> Result<BackupImportResult, AppError> {
    let zip_path = normalize_output_path(&app.app_root, zip_path);
    let manifest = read_backup_manifest(&zip_path)?;
    apply_backup_archive(app, &zip_path, &manifest)?;
    Ok(BackupImportResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        manifest,
    })
}

pub fn write_backup_zip(
    zip_path: &Path,
    manifest: &BackupManifest,
    app: &AppState,
) -> Result<(), AppError> {
    let file = File::create(zip_path)?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);
    let manifest_json = serde_json::to_vec_pretty(manifest)?;
    writer.start_file("manifest.json", options)?;
    writer.write_all(&manifest_json)?;

    let entries = [
        ("config.yaml", app.app_root.join("config.yaml")),
        ("data", app.app_root.join("data")),
    ];
    for (name, path) in entries {
        if path.is_file() {
            add_file(&mut writer, &path, name)?;
        } else if path.is_dir() {
            add_dir(&mut writer, &path, name)?;
        }
    }

    writer.finish()?;
    Ok(())
}

fn add_file(writer: &mut ZipWriter<File>, path: &Path, archive_path: &str) -> Result<(), AppError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let header = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);
    writer.start_file(archive_path, header)?;
    writer.write_all(&buffer)?;
    Ok(())
}

fn add_dir(writer: &mut ZipWriter<File>, dir: &Path, archive_base: &str) -> Result<(), AppError> {
    let base = archive_base.trim_end_matches('/');
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|err| AppError::other(err.to_string()))?;
        let path = entry.path();
        if path == dir {
            continue;
        }
        let rel = path.strip_prefix(dir).unwrap_or(path);
        let archive_path = format!("{}/{}", base, rel.to_string_lossy().replace('\\', "/"));
        if path.is_dir() {
            writer.add_directory(archive_path, FileOptions::<()>::default())?;
        } else {
            add_file(writer, path, &archive_path)?;
        }
    }
    Ok(())
}

fn read_backup_manifest(zip_path: &Path) -> Result<BackupManifest, AppError> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut manifest = archive.by_name("manifest.json")?;
    let mut raw = String::new();
    manifest.read_to_string(&mut raw)?;
    let parsed: BackupManifest = serde_json::from_str(&raw)?;
    if parsed.format != BACKUP_FORMAT {
        return Err(AppError::validation("unsupported backup format"));
    }
    if parsed.manifest_version != BACKUP_MANIFEST_VERSION {
        return Err(AppError::validation("unsupported backup manifest version"));
    }
    Ok(parsed)
}

fn apply_backup_archive(
    app: &AppState,
    zip_path: &Path,
    _manifest: &BackupManifest,
) -> Result<(), AppError> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let tmp_dir = app.app_root.join("data").join("_backup_import");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;
    extract_zip_safely(&mut archive, &tmp_dir)?;

    let config_src = tmp_dir.join("config.yaml");
    if config_src.exists() {
        fs::copy(&config_src, app.app_root.join("config.yaml"))?;
    }

    let data_src = tmp_dir.join("data");
    if data_src.exists() {
        copy_dir_contents(&data_src, &app.app_root.join("data"))?;
    }

    fs::remove_dir_all(&tmp_dir)?;
    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_contents(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn normalize_output_path(app_root: &Path, input: &str) -> PathBuf {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return app_root.join("backup.zip");
    }
    resolve_app_path(app_root, trimmed)
}

fn app_root_exists(app_root: &Path, path: &str) -> bool {
    resolve_app_path(app_root, path).exists()
}
