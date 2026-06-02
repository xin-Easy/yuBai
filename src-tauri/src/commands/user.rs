// 用户相关的本地逻辑（如读取本地配置文件）
use crate::app::state::AppState;
use tauri::State;

#[tauri::command]
pub fn user_config_get(
    state: State<'_, AppState>,
) -> Result<crate::domain::config::Config, String> {
    state
        .services
        .user
        .config_get(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn user_config_save(
    state: State<'_, AppState>,
    config: crate::domain::config::Config,
) -> Result<(), String> {
    state
        .services
        .user
        .config_save(state.inner(), config)
        .map_err(Into::into)
}
