use crate::{
    infra::db::{
        database::Database,
        support::{map_core_row, BrowserCoreRow, NewBrowserCoreRow},
    },
    domain::{
        browser::{BrowserCore, BrowserCoreInput},
        text::non_empty_owned,
    },
    error::AppError,
};
use diesel::prelude::*;
use uuid::Uuid;

impl Database {
    pub fn list_cores(&self) -> Result<Vec<BrowserCore>, AppError> {
        use crate::infra::db::schema::browser_cores::dsl as cores;

        let mut conn = self.lock()?;
        let rows = cores::browser_cores
            .select(BrowserCoreRow::as_select())
            .order((
                cores::is_default.desc(),
                cores::sort_order.asc(),
                cores::created_at.asc(),
            ))
            .load::<BrowserCoreRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_core_row).collect())
    }

    pub fn save_core(&self, input: BrowserCoreInput) -> Result<BrowserCore, AppError> {
        use crate::infra::db::schema::browser_cores::dsl as cores;

        let core_id = input
            .core_id
            .and_then(non_empty_owned)
            .unwrap_or_else(|| format!("core-{}", Uuid::new_v4()));
        let core = BrowserCore {
            core_id,
            core_name: input.core_name.trim().to_string(),
            core_path: input.core_path.trim().to_string(),
            is_default: input.is_default,
        };

        let mut conn = self.lock()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            if core.is_default {
                diesel::update(cores::browser_cores)
                    .set(cores::is_default.eq(0))
                    .execute(conn)?;
            }

            diesel::insert_into(cores::browser_cores)
                .values(&NewBrowserCoreRow {
                    core_id: &core.core_id,
                    core_name: &core.core_name,
                    core_path: &core.core_path,
                    is_default: if core.is_default { 1 } else { 0 },
                })
                .on_conflict(cores::core_id)
                .do_update()
                .set((
                    cores::core_name.eq(&core.core_name),
                    cores::core_path.eq(&core.core_path),
                    cores::is_default.eq(if core.is_default { 1 } else { 0 }),
                ))
                .execute(conn)?;

            Ok(())
        })?;

        Ok(core)
    }

    pub fn delete_core(&self, core_id: String) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_cores::dsl as cores;

        let mut conn = self.lock()?;
        diesel::delete(cores::browser_cores.filter(cores::core_id.eq(core_id)))
            .execute(&mut *conn)?;
        Ok(())
    }

    pub fn resolve_core_for_profile(&self, core_id: &str) -> Result<Option<BrowserCore>, AppError> {
        let cores = self.list_cores()?;
        if !core_id.trim().is_empty() {
            if let Some(core) = cores.iter().find(|core| core.core_id == core_id).cloned() {
                return Ok(Some(core));
            }
        }
        Ok(cores
            .iter()
            .find(|core| core.is_default)
            .cloned()
            .or_else(|| cores.into_iter().next()))
    }
}
