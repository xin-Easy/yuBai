use crate::error::AppError;
use std::{
    fs::{self, File},
    io,
    path::Path,
};
use zip::ZipArchive;

pub fn extract_zip_safely(
    archive: &mut ZipArchive<File>,
    target_dir: &Path,
) -> Result<(), AppError> {
    fs::create_dir_all(target_dir)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed_name) = entry.enclosed_name() else {
            return Err(AppError::validation(format!(
                "archive contains unsafe path: {}",
                entry.name()
            )));
        };
        let output_path = target_dir.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(&output_path)?;
        io::copy(&mut entry, &mut output)?;
    }

    Ok(())
}
