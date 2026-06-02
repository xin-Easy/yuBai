use crate::{
    domain::{config::Config, app::DashboardStats},
    error::AppError,
    infra::db::database::Database,
};
use diesel::{dsl::count_star, prelude::*};

impl Database {
    pub fn dashboard_stats(
        &self,
        _config: &Config,
        app_version: &str,
        app_root: &std::path::Path,
    ) -> Result<DashboardStats, AppError> {
        let mut conn = self.lock()?;
        Ok(DashboardStats {
            total_instances: crate::infra::db::schema::browser_profiles::table
                .select(count_star())
                .first::<i64>(&mut *conn)?
                .max(0) as usize,
            running_instances: 0,
            proxy_count: crate::infra::db::schema::browser_proxies::table
                .select(count_star())
                .first::<i64>(&mut *conn)?
                .max(0) as usize,
            core_count: crate::infra::db::schema::browser_cores::table
                .select(count_star())
                .first::<i64>(&mut *conn)?
                .max(0) as usize,
            app_version: app_version.to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            app_root: app_root.to_string_lossy().to_string(),
        })
    }
}
