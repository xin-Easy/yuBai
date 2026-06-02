use crate::domain::config::Config;
use crate::domain::paths::resolve_app_path;
use crate::error::AppError;
use diesel::Connection;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use std::{fs, path::Path, sync::Mutex};

pub const SYSTEM_DIRECT_PROXY_ID: &str = "proxy-direct";
pub const SYSTEM_DIRECT_PROXY_NAME: &str = "本机直连（不走代理）";
pub const SYSTEM_DIRECT_PROXY_CONFIG: &str = "direct://";

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS browser_profiles (
    profile_id TEXT PRIMARY KEY NOT NULL,
    profile_name TEXT NOT NULL,
    user_data_dir TEXT NOT NULL,
    core_id TEXT NOT NULL,
    fingerprint_args TEXT NOT NULL,
    fingerprint_json TEXT NOT NULL,
    proxy_id TEXT NOT NULL,
    proxy_config TEXT NOT NULL,
    launch_args TEXT NOT NULL,
    tags TEXT NOT NULL,
    keywords TEXT NOT NULL,
    group_id TEXT,
    proxy_bind_source_id TEXT,
    proxy_bind_source_url TEXT,
    proxy_bind_name TEXT,
    proxy_bind_updated_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS browser_proxies (
    proxy_id TEXT PRIMARY KEY NOT NULL,
    proxy_name TEXT NOT NULL,
    proxy_config TEXT NOT NULL,
    dns_servers TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    group_name TEXT,
    last_latency_ms BIGINT NOT NULL,
    last_test_ok INTEGER NOT NULL,
    last_tested_at TEXT,
    last_ip_health_json TEXT NOT NULL,
    source_id TEXT,
    source_url TEXT,
    source_name_prefix TEXT,
    source_auto_refresh INTEGER NOT NULL,
    source_refresh_interval_m INTEGER NOT NULL,
    source_last_refresh_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS browser_cores (
    core_id TEXT PRIMARY KEY NOT NULL,
    core_name TEXT NOT NULL,
    core_path TEXT NOT NULL,
    is_default INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS browser_bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    open_on_start INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS browser_groups (
    group_id TEXT PRIMARY KEY NOT NULL,
    group_name TEXT NOT NULL,
    parent_id TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

pub struct Database {
    conn: Mutex<SqliteConnection>,
}

impl Database {
    pub fn open(app_root: &Path, config: &Config) -> Result<Self, AppError> {
        let db_path = resolve_app_path(app_root, &config.database.sqlite.path);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut conn = SqliteConnection::establish(&db_path.to_string_lossy())
            .map_err(|err| AppError::other(format!("failed to open db: {err}")))?;
        conn.batch_execute("PRAGMA journal_mode = WAL;")
            .map_err(|err| AppError::other(format!("failed to enable WAL: {err}")))?;
        conn.batch_execute("PRAGMA foreign_keys = ON;")
            .map_err(|err| AppError::other(format!("failed to enable foreign keys: {err}")))?;
        conn.batch_execute(SCHEMA_SQL)
            .map_err(|err| AppError::other(format!("failed to bootstrap schema: {err}")))?;

        let database = Self {
            conn: Mutex::new(conn),
        };
        Ok(database)
    }

    pub(crate) fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, SqliteConnection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::LockPoisoned("database mutex poisoned".to_string()))
    }
}
