use tauri::Manager;

use crate::{app::state::AppState, infra};

pub fn build() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data directory");

            let state = AppState::new(app_data_dir);
            let _ = infra::logging::init_global(state.app_root.join("data").join("app.log"));
            if let Err(err) = infra::proxy::extension_installer::ensure_installed(&state.app_root) {
                eprintln!("failed to install proxy checker extension: {err}");
            }
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
}
