use crate::{
    infra::{archive::extract_zip_safely, logging},
    domain::{runtime::SnapshotInfo, paths::resolve_app_path},
    error::AppError,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub snapshot_id: String,
    pub profile_id: String,
    pub name: String,
    pub created_at: String,
    pub format: String,
    pub source: String,
    pub size_mb: f64,
}

pub fn snapshot_dir(app_root: &Path, profile_id: &str) -> PathBuf {
    app_root.join("data").join("snapshots").join(profile_id)
}

pub fn list_snapshots(app_root: &Path, profile_id: &str) -> Result<Vec<SnapshotInfo>, AppError> {
    let root = snapshot_dir(app_root, profile_id);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("zip") {
            match read_snapshot_meta(&path) {
                Ok(meta) => items.push(SnapshotInfo {
                    name: meta.name,
                    snapshot_id: meta.snapshot_id,
                    profile_id: meta.profile_id,
                    size_mb: meta.size_mb,
                    created_at: meta.created_at,
                }),
                Err(err) => {
                    logging::log_target(
                        "warn",
                        "snapshot",
                        "failed to read snapshot metadata",
                        format!("path={} error={err}", path.to_string_lossy()),
                    );
                }
            }
        }
    }
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

pub fn create_snapshot(
    app_root: &Path,
    profile_id: &str,
    name: &str,
    user_data_dir: &str,
) -> Result<SnapshotInfo, AppError> {
    let source = resolve_app_path(app_root, user_data_dir);
    if !source.exists() {
        return Err(AppError::not_found(format!(
            "profile user data dir not found: {}",
            source.to_string_lossy()
        )));
    }

    let snapshot_id = format!("snapshot-{}", uuid::Uuid::new_v4());
    let root = snapshot_dir(app_root, profile_id);
    fs::create_dir_all(&root)?;
    let archive_path = root.join(format!("{snapshot_id}.zip"));
    let created_at = Utc::now().to_rfc3339();
    let size_mb = write_snapshot_archive(&source, &archive_path)?;
    let meta = SnapshotMeta {
        snapshot_id: snapshot_id.clone(),
        profile_id: profile_id.to_string(),
        name: if name.trim().is_empty() {
            "Snapshot".to_string()
        } else {
            name.trim().to_string()
        },
        created_at: created_at.clone(),
        format: "zip+meta".to_string(),
        source: source.to_string_lossy().to_string(),
        size_mb,
    };
    write_snapshot_meta(&archive_path, &meta)?;

    Ok(SnapshotInfo {
        snapshot_id,
        profile_id: profile_id.to_string(),
        name: meta.name,
        size_mb,
        created_at,
    })
}

pub fn restore_snapshot(
    app_root: &Path,
    profile_user_data_dir: &str,
    profile_id: &str,
    snapshot_id: &str,
) -> Result<(), AppError> {
    let root = snapshot_dir(app_root, profile_id);
    let zip_path = root.join(format!("{snapshot_id}.zip"));
    let target = resolve_app_path(app_root, profile_user_data_dir);

    if target.exists() {
        let backup = target.with_extension(format!("restore-backup-{}", Utc::now().timestamp()));
        fs::rename(&target, backup)?;
    }
    fs::create_dir_all(&target)?;

    if zip_path.exists() {
        extract_snapshot_archive(&zip_path, &target)?;
        return Ok(());
    }
    Err(AppError::not_found("snapshot not found"))
}

pub fn delete_snapshot(
    app_root: &Path,
    profile_id: &str,
    snapshot_id: &str,
) -> Result<(), AppError> {
    let root = snapshot_dir(app_root, profile_id);
    let zip_path = root.join(format!("{snapshot_id}.zip"));
    if zip_path.exists() {
        fs::remove_file(zip_path)?;
    }
    Ok(())
}

fn write_snapshot_archive(source: &Path, archive_path: &Path) -> Result<f64, AppError> {
    let file = File::create(archive_path)?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    let mut total_bytes = 0u64;
    add_dir_to_zip(
        source,
        source,
        &mut writer,
        options,
        &mut buffer,
        &mut total_bytes,
    )?;
    writer.finish()?;
    Ok(total_bytes as f64 / 1024.0 / 1024.0)
}

fn add_dir_to_zip(
    base: &Path,
    path: &Path,
    writer: &mut ZipWriter<File>,
    options: FileOptions<()>,
    buffer: &mut Vec<u8>,
    total_bytes: &mut u64,
) -> Result<(), AppError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let rel = entry_path.strip_prefix(base).unwrap_or(&entry_path);
        let name = rel.to_string_lossy().replace('\\', "/");
        if entry_path.is_dir() {
            writer.add_directory(format!("{name}/"), options)?;
            add_dir_to_zip(base, &entry_path, writer, options, buffer, total_bytes)?;
        } else {
            writer.start_file(name, options)?;
            let mut file = File::open(&entry_path)?;
            let bytes = file.read_to_end(buffer)?;
            writer.write_all(buffer)?;
            writer.flush()?;
            *total_bytes += bytes as u64;
            buffer.clear();
        }
    }
    Ok(())
}

fn extract_snapshot_archive(archive_path: &Path, target: &Path) -> Result<(), AppError> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    extract_zip_safely(&mut archive, target)?;
    Ok(())
}

fn write_snapshot_meta(archive_path: &Path, meta: &SnapshotMeta) -> Result<(), AppError> {
    let meta_path = archive_path.with_extension("zip.meta.json");
    let raw = serde_json::to_string_pretty(meta)?;
    fs::write(meta_path, raw)?;
    Ok(())
}

fn read_snapshot_meta(archive_path: &Path) -> Result<SnapshotMeta, AppError> {
    let meta_path = archive_path.with_extension("zip.meta.json");
    let raw = fs::read_to_string(meta_path)?;
    Ok(serde_json::from_str(&raw)?)
}
