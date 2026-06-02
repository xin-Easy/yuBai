use crate::domain::config::Config;
use crate::domain::paths::resolve_app_path;
use crate::error::AppError;
use diesel::Connection;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use std::{fs, path::Path, sync::Mutex};

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("../src-tauri/migrations");
pub const SYSTEM_DIRECT_PROXY_ID: &str = "proxy-direct";
pub const SYSTEM_DIRECT_PROXY_NAME: &str = "本机直连（不走代理）";
pub const SYSTEM_DIRECT_PROXY_CONFIG: &str = "direct://";

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

        let database = Self {
            conn: Mutex::new(conn),
        };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&self) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        diesel_migrations::MigrationHarness::run_pending_migrations(&mut *conn, MIGRATIONS)
            .map_err(|err| AppError::other(format!("failed to run pending migrations: {err}")))?;
        Ok(())
    }

    pub(crate) fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, SqliteConnection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::LockPoisoned("database mutex poisoned".to_string()))
    }
}
