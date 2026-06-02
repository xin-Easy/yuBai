use crate::infra::db::schema::{browser_bookmarks, browser_cores, browser_profiles, browser_proxies};
use crate::domain::{
    browser::{BrowserCore, BrowserFingerprint, BrowserProfile, BrowserProxy},
    runtime::BrowserBookmark,
};
use crate::error::AppError;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = browser_cores)]
pub(crate) struct BrowserCoreRow {
    pub core_id: String,
    pub core_name: String,
    pub core_path: String,
    pub is_default: i32,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = browser_cores)]
pub(crate) struct NewBrowserCoreRow<'a> {
    pub core_id: &'a str,
    pub core_name: &'a str,
    pub core_path: &'a str,
    pub is_default: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = browser_proxies)]
pub(crate) struct BrowserProxyRow {
    pub proxy_id: String,
    pub proxy_name: String,
    pub proxy_config: String,
    pub dns_servers: Option<String>,
    pub sort_order: i32,
    pub group_name: Option<String>,
    pub last_latency_ms: i64,
    pub last_test_ok: i32,
    pub last_tested_at: Option<String>,
    pub last_ip_health_json: String,
    pub source_id: Option<String>,
    pub source_url: Option<String>,
    pub source_name_prefix: Option<String>,
    pub source_auto_refresh: i32,
    pub source_refresh_interval_m: i32,
    pub source_last_refresh_at: Option<String>,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = browser_proxies)]
pub(crate) struct NewBrowserProxyRow<'a> {
    pub proxy_id: &'a str,
    pub proxy_name: &'a str,
    pub proxy_config: &'a str,
    pub dns_servers: Option<&'a str>,
    pub group_name: Option<&'a str>,
    pub last_latency_ms: i64,
    pub last_test_ok: i32,
    pub last_tested_at: Option<&'a str>,
    pub last_ip_health_json: &'a str,
    pub source_id: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub source_name_prefix: Option<&'a str>,
    pub source_auto_refresh: i32,
    pub source_refresh_interval_m: i32,
    pub source_last_refresh_at: Option<&'a str>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = browser_bookmarks)]
pub(crate) struct BrowserBookmarkRow {
    pub name: String,
    pub url: String,
    pub open_on_start: i32,
}

#[derive(Insertable)]
#[diesel(table_name = browser_bookmarks)]
pub(crate) struct NewBrowserBookmarkRow<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub open_on_start: i32,
    pub sort_order: i32,
}

#[derive(Queryable)]
#[diesel(table_name = browser_profiles)]
pub(crate) struct BrowserProfileRow {
    pub profile_id: String,
    pub profile_name: String,
    pub user_data_dir: String,
    pub core_id: String,
    pub fingerprint_args: String,
    pub fingerprint_json: String,
    pub proxy_id: String,
    pub proxy_config: String,
    pub launch_args: String,
    pub tags: String,
    pub keywords: String,
    pub group_id: Option<String>,
    pub proxy_bind_source_id: Option<String>,
    pub proxy_bind_source_url: Option<String>,
    pub proxy_bind_name: Option<String>,
    pub proxy_bind_updated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = browser_profiles)]
pub(crate) struct NewBrowserProfileRow<'a> {
    pub profile_id: &'a str,
    pub profile_name: &'a str,
    pub user_data_dir: &'a str,
    pub core_id: &'a str,
    pub fingerprint_args: &'a str,
    pub fingerprint_json: &'a str,
    pub proxy_id: &'a str,
    pub proxy_config: &'a str,
    pub launch_args: &'a str,
    pub tags: &'a str,
    pub keywords: &'a str,
    pub group_id: Option<&'a str>,
    pub proxy_bind_source_id: Option<&'a str>,
    pub proxy_bind_source_url: Option<&'a str>,
    pub proxy_bind_name: Option<&'a str>,
    pub proxy_bind_updated_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

pub(crate) fn to_json<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    Ok(serde_json::to_string(value)?)
}

pub(crate) fn map_core_row(row: BrowserCoreRow) -> BrowserCore {
    let _ = row.sort_order;
    let _ = row.created_at;
    BrowserCore {
        core_id: row.core_id,
        core_name: row.core_name,
        core_path: row.core_path,
        is_default: row.is_default != 0,
    }
}

pub(crate) fn map_proxy_row(row: BrowserProxyRow) -> BrowserProxy {
    let _ = row.sort_order;
    let _ = row.created_at;
    BrowserProxy {
        proxy_id: row.proxy_id,
        proxy_name: row.proxy_name,
        proxy_config: row.proxy_config,
        dns_servers: row.dns_servers,
        group_name: row.group_name,
        source_id: row.source_id,
        source_url: row.source_url,
        source_name_prefix: row.source_name_prefix,
        source_auto_refresh: row.source_auto_refresh != 0,
        source_refresh_interval_m: row.source_refresh_interval_m,
        source_last_refresh_at: row.source_last_refresh_at,
        last_latency_ms: row.last_latency_ms,
        last_test_ok: row.last_test_ok != 0,
        last_tested_at: row.last_tested_at,
        last_ip_health_json: if row.last_ip_health_json.is_empty() {
            None
        } else {
            Some(row.last_ip_health_json)
        },
        bound_profile_names: Vec::new(),
    }
}

pub(crate) fn map_profile_row(row: BrowserProfileRow) -> BrowserProfile {
    BrowserProfile {
        profile_id: row.profile_id,
        profile_name: row.profile_name,
        user_data_dir: row.user_data_dir,
        core_id: row.core_id,
        fingerprint_args: parse_json_list(&row.fingerprint_args),
        fingerprint: parse_fingerprint(&row.fingerprint_json),
        proxy_id: row.proxy_id,
        proxy_config: row.proxy_config,
        proxy_bind_source_id: row.proxy_bind_source_id,
        proxy_bind_source_url: row.proxy_bind_source_url,
        proxy_bind_name: row.proxy_bind_name,
        proxy_bind_updated_at: row.proxy_bind_updated_at,
        launch_args: parse_json_list(&row.launch_args),
        tags: parse_json_list(&row.tags),
        keywords: parse_json_list(&row.keywords),
        group_id: row.group_id.and_then(crate::domain::text::non_empty_owned),
        running: false,
        debug_port: 0,
        debug_ready: false,
        pid: 0,
        automation_run_id: None,
        runtime_proxy_summary: String::new(),
        runtime_warning: String::new(),
        last_error: String::new(),
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_start_at: None,
        last_stop_at: None,
        launch_code: None,
    }
}

pub(crate) fn map_bookmark_row(row: BrowserBookmarkRow) -> BrowserBookmark {
    BrowserBookmark {
        name: row.name,
        url: row.url,
        open_on_start: row.open_on_start != 0,
    }
}

pub(crate) fn empty_str_to_option_ref(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn parse_json_list(value: &str) -> Vec<String> {
    match serde_json::from_str::<Vec<String>>(value) {
        Ok(items) => items,
        Err(err) => {
            crate::infra::logging::log_target(
                "warn",
                "database",
                "failed to parse stored string list JSON",
                err.to_string(),
            );
            Vec::new()
        }
    }
}

pub(crate) fn parse_fingerprint(value: &str) -> Option<BrowserFingerprint> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

pub(crate) fn generate_profile_dir_name(profile_name: &str) -> String {
    use pinyin::ToPinyin;

    let name = profile_name.trim();
    let mut pinyin_parts: Vec<String> = Vec::new();

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            pinyin_parts.push(ch.to_lowercase().to_string());
        } else if let Some(pinyin) = ch.to_pinyin() {
            pinyin_parts.push(pinyin.plain().to_string());
        } else if ch == '_' || ch == '-' {
            pinyin_parts.push(ch.to_string());
        }
    }

    let slug = pinyin_parts.join("-");
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    format!("yubai_{slug}_{timestamp}")
}
