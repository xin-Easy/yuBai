use tauri::{AppHandle, Manager};

use crate::{
    app::state::AppState,
    domain::{
        automation::{
            BrowserProxyBatchTestResult, BrowserProxyImportInput, BrowserProxyImportResult,
            BrowserProxySubscriptionFetchResult, BrowserProxyTestResult,
        },
        browser::{BrowserProxy, BrowserProxyInput},
    },
    error::AppError,
    infra::proxy,
};

pub struct ProxyService;

impl ProxyService {
    pub fn list(&self, state: &AppState) -> Result<Vec<BrowserProxy>, AppError> {
        let mut proxies = state.repositories.database.list_proxies()?;
        for proxy in &mut proxies {
            proxy.bound_profile_names = state
                .repositories
                .database
                .find_profile_names_by_proxy_id(&proxy.proxy_id)
                .unwrap_or_default();
        }
        Ok(proxies)
    }

    pub fn save(&self, state: &AppState, input: BrowserProxyInput) -> Result<BrowserProxy, AppError> {
        state.repositories.database.save_proxy(input)
    }

    pub fn delete(&self, state: &AppState, proxy_id: String) -> Result<(), AppError> {
        state.repositories.database.delete_proxy(proxy_id)
    }

    pub fn list_by_source(&self, state: &AppState, source_id: &str) -> Result<Vec<BrowserProxy>, AppError> {
        state.repositories.database.list_proxies_by_source(source_id)
    }

    pub async fn fetch_clash_by_url(
        url: String,
    ) -> Result<BrowserProxySubscriptionFetchResult, AppError> {
        tokio::task::spawn_blocking(move || proxy::import::fetch_clash_subscription(url))
            .await
            .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }

    pub async fn import_text(
        app: AppHandle,
        input: BrowserProxyImportInput,
    ) -> Result<BrowserProxyImportResult, AppError> {
        tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            proxy::import::import_text(state.inner(), input)
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }

    pub async fn test(
        app: AppHandle,
        proxy_id: String,
    ) -> Result<BrowserProxyTestResult, AppError> {
        tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            proxy::import::test_proxy(state.inner(), proxy_id)
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }

    pub async fn batch_test(
        app: AppHandle,
        proxy_ids: Vec<String>,
    ) -> Result<BrowserProxyBatchTestResult, AppError> {
        tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            proxy::import::batch_test_proxies(state.inner(), proxy_ids)
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }

    pub async fn ip_health(
        app: AppHandle,
        proxy_id: String,
    ) -> Result<BrowserProxyTestResult, AppError> {
        tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            proxy::import::ip_health(state.inner(), proxy_id)
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }

    pub async fn refresh_subscription(
        app: AppHandle,
        source_id: String,
    ) -> Result<BrowserProxyImportResult, AppError> {
        tokio::task::spawn_blocking(move || {
            let state = app.state::<AppState>();
            proxy::import::refresh_subscription(state.inner(), source_id)
        })
        .await
        .map_err(|err| AppError::other(format!("task join error: {err}")))?
    }
}
