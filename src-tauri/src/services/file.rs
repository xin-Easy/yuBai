use crate::{app::state::AppState, domain::paths::resolve_app_path};

pub struct FileService;

impl FileService {
    pub fn resolve_path(&self, state: &AppState, path: String) -> String {
        resolve_app_path(&state.app_root, &path)
            .to_string_lossy()
            .to_string()
    }

    pub fn exists(&self, state: &AppState, path: String) -> bool {
        resolve_app_path(&state.app_root, &path).exists()
    }
}
