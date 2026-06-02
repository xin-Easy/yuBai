use crate::{
    infra::db::{database::Database, support::empty_str_to_option_ref},
    domain::{
        browser::{BrowserGroup, BrowserGroupInput, BrowserGroupWithCount},
        text::non_empty_owned,
    },
    error::AppError,
};
use chrono::Utc;
use diesel::{dsl::count_star, prelude::*};
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::infra::db::schema::browser_groups)]
struct NewBrowserGroupRow<'a> {
    group_id: &'a str,
    group_name: &'a str,
    parent_id: Option<&'a str>,
    sort_order: i32,
    created_at: &'a str,
    updated_at: &'a str,
}

impl Database {
    pub fn list_groups(&self) -> Result<Vec<BrowserGroupWithCount>, AppError> {
        use crate::infra::db::schema::browser_groups::dsl as groups;
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        let rows = groups::browser_groups
            .left_join(
                profiles::browser_profiles.on(profiles::group_id.eq(groups::group_id.nullable())),
            )
            .group_by((
                groups::group_id,
                groups::group_name,
                groups::parent_id,
                groups::sort_order,
                groups::created_at,
                groups::updated_at,
            ))
            .select((
                groups::group_id,
                groups::group_name,
                groups::parent_id,
                groups::sort_order,
                groups::created_at,
                groups::updated_at,
                count_star(),
            ))
            .order((groups::sort_order.asc(), groups::created_at.asc()))
            .load::<(String, String, Option<String>, i32, String, String, i64)>(&mut *conn)?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    group_id,
                    group_name,
                    parent_id,
                    sort_order,
                    created_at,
                    updated_at,
                    instance_count,
                )| BrowserGroupWithCount {
                    group_id,
                    group_name,
                    parent_id: parent_id.unwrap_or_default(),
                    sort_order,
                    created_at,
                    updated_at,
                    instance_count: instance_count as i32,
                },
            )
            .collect())
    }

    pub fn save_group(&self, input: BrowserGroupInput) -> Result<BrowserGroup, AppError> {
        use crate::infra::db::schema::browser_groups::dsl as groups;

        let now = Utc::now().to_rfc3339();
        let group_id = input
            .group_id
            .and_then(non_empty_owned)
            .unwrap_or_else(|| format!("group-{}", Uuid::new_v4()));
        let group = BrowserGroup {
            group_id,
            group_name: input.group_name.trim().to_string(),
            parent_id: input.parent_id.and_then(non_empty_owned).unwrap_or_default(),
            sort_order: input.sort_order,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut conn = self.lock()?;
        diesel::insert_into(groups::browser_groups)
            .values(&NewBrowserGroupRow {
                group_id: &group.group_id,
                group_name: &group.group_name,
                parent_id: empty_str_to_option_ref(&group.parent_id),
                sort_order: group.sort_order,
                created_at: &group.created_at,
                updated_at: &group.updated_at,
            })
            .on_conflict(groups::group_id)
            .do_update()
            .set((
                groups::group_name.eq(&group.group_name),
                groups::parent_id.eq(empty_str_to_option_ref(&group.parent_id)),
                groups::sort_order.eq(group.sort_order),
                groups::updated_at.eq(&group.updated_at),
            ))
            .execute(&mut *conn)?;

        Ok(group)
    }

    pub fn delete_group(&self, group_id: String) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_groups::dsl as groups;
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::update(
                profiles::browser_profiles.filter(profiles::group_id.eq(Some(group_id.clone()))),
            )
            .set(profiles::group_id.eq::<Option<String>>(None))
            .execute(conn)?;
            diesel::delete(groups::browser_groups.filter(groups::group_id.eq(group_id)))
                .execute(conn)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn move_profiles_to_group(
        &self,
        profile_ids: Vec<String>,
        group_id: String,
    ) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        diesel::update(profiles::browser_profiles.filter(profiles::profile_id.eq_any(profile_ids)))
            .set(profiles::group_id.eq(Some(group_id)))
            .execute(&mut *conn)?;
        Ok(())
    }
}
