use tauri::{AppHandle, State};

use crate::{
    app::state::AppState,
    domain::{
        automation::{
            BrowserProxyBatchTestResult, BrowserProxyImportInput, BrowserProxyImportResult,
            BrowserProxySubscriptionFetchResult, BrowserProxyTestResult,
        },
        browser::{BrowserProxy, BrowserProxyInput},
    },
};

#[tauri::command]
pub fn browser_proxy_list(state: State<'_, AppState>) -> Result<Vec<BrowserProxy>, String> {
    state
        .services
        .proxy
        .list(state.inner())
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_proxy_save(
    state: State<'_, AppState>,
    input: BrowserProxyInput,
) -> Result<BrowserProxy, String> {
    state
        .services
        .proxy
        .save(state.inner(), input)
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_proxy_delete(state: State<'_, AppState>, proxy_id: String) -> Result<(), String> {
    state
        .services
        .proxy
        .delete(state.inner(), proxy_id)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_fetch_clash_by_url(
    url: String,
) -> Result<BrowserProxySubscriptionFetchResult, String> {
    crate::services::proxy::ProxyService::fetch_clash_by_url(url)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_import_text(
    app: AppHandle,
    input: BrowserProxyImportInput,
) -> Result<BrowserProxyImportResult, String> {
    crate::services::proxy::ProxyService::import_text(app, input)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_test(
    app: AppHandle,
    proxy_id: String,
) -> Result<BrowserProxyTestResult, String> {
    crate::services::proxy::ProxyService::test(app, proxy_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_batch_test(
    app: AppHandle,
    proxy_ids: Vec<String>,
) -> Result<BrowserProxyBatchTestResult, String> {
    crate::services::proxy::ProxyService::batch_test(app, proxy_ids)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_ip_health(
    app: AppHandle,
    proxy_id: String,
) -> Result<BrowserProxyTestResult, String> {
    crate::services::proxy::ProxyService::ip_health(app, proxy_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn browser_proxy_refresh_subscription(
    app: AppHandle,
    source_id: String,
) -> Result<BrowserProxyImportResult, String> {
    crate::services::proxy::ProxyService::refresh_subscription(app, source_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub fn browser_proxy_list_by_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<Vec<BrowserProxy>, String> {
    state
        .services
        .proxy
        .list_by_source(state.inner(), &source_id)
        .map_err(Into::into)
}
