use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

pub fn resolve_app_path(app_root: &Path, path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return app_root
            .absolutize()
            .map(|path| path.into_owned())
            .unwrap_or_else(|_| app_root.to_path_buf());
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        candidate
            .absolutize()
            .map(|path| path.into_owned())
            .unwrap_or(candidate)
    } else {
        app_root
            .join(candidate)
            .absolutize()
            .map(|path| path.into_owned())
            .unwrap_or_else(|_| app_root.join(trimmed))
    }
}
