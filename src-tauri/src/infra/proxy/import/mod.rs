mod health;
mod ingest;
mod subscription;

pub use health::{batch_test_proxies, ip_health, test_proxy};
pub use ingest::import_text;
pub use subscription::fetch_clash_subscription;

use crate::{
    app::state::AppState,
    domain::automation::{BrowserProxyImportInput, BrowserProxyImportResult},
    error::AppError,
};

pub fn refresh_subscription(
    state: &AppState,
    source_id: String,
) -> Result<BrowserProxyImportResult, AppError> {
    let existing = state.repositories.database.list_proxies_by_source(&source_id)?;
    let first = existing
        .iter()
        .find(|p| p.source_url.is_some())
        .ok_or_else(|| AppError::not_found(format!("subscription not found: {source_id}")))?;
    let source_url = first
        .source_url
        .clone()
        .ok_or_else(|| AppError::validation("subscription has no source URL"))?;

    let fetch_result = fetch_clash_subscription(source_url.clone())?;
    state.repositories.database.delete_proxies_by_source(&source_id)?;

    import_text(
        state,
        BrowserProxyImportInput {
            content: fetch_result.content,
            group_name: first.group_name.clone(),
            name_prefix: first.source_name_prefix.clone(),
            dns_servers: first.dns_servers.clone(),
            source_id: Some(source_id),
            source_url: Some(source_url),
            source_name_prefix: first.source_name_prefix.clone(),
        },
    )
}
