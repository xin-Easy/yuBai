use crate::{
    infra::db::{
        database::Database,
        support::{generate_profile_dir_name, map_profile_row, to_json, BrowserProfileRow, NewBrowserProfileRow},
    },
    domain::{
        config::Config,
        browser::{BrowserProfile, BrowserProfileInput},
        text::non_empty_owned,
    },
    error::AppError,
};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

impl Database {
    pub fn list_profiles(&self) -> Result<Vec<BrowserProfile>, AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        let rows = profiles::browser_profiles
            .order(profiles::created_at.asc())
            .load::<BrowserProfileRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_profile_row).collect())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<BrowserProfile>, AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        let row = profiles::browser_profiles
            .filter(profiles::profile_id.eq(profile_id))
            .first::<BrowserProfileRow>(&mut *conn)
            .optional()?;

        Ok(row.map(map_profile_row))
    }

    pub fn create_profile(
        &self,
        input: BrowserProfileInput,
        config: &Config,
    ) -> Result<BrowserProfile, AppError> {
        let now = Utc::now().to_rfc3339();
        let profile_id = format!("profile-{}", Uuid::new_v4());
        let user_data_dir = if input.user_data_dir.trim().is_empty() {
            let dir_name = generate_profile_dir_name(&input.profile_name);
            format!(
                "{}/{}",
                config.browser.user_data_root.trim_end_matches('/'),
                dir_name
            )
        } else {
            input.user_data_dir.trim().to_string()
        };

        let profile = BrowserProfile {
            profile_id,
            profile_name: input.profile_name.trim().to_string(),
            user_data_dir,
            core_id: input.core_id.trim().to_string(),
            fingerprint_args: input.fingerprint_args,
            fingerprint: input.fingerprint,
            proxy_id: input.proxy_id.trim().to_string(),
            proxy_config: input.proxy_config.trim().to_string(),
            proxy_bind_source_id: None,
            proxy_bind_source_url: None,
            proxy_bind_name: None,
            proxy_bind_updated_at: None,
            launch_args: input.launch_args,
            tags: input.tags,
            keywords: input.keywords,
            group_id: input.group_id.and_then(non_empty_owned),
            running: false,
            debug_port: 0,
            debug_ready: false,
            pid: 0,
            automation_run_id: None,
            runtime_proxy_summary: "未运行".to_string(),
            runtime_warning: String::new(),
            last_error: String::new(),
            created_at: now.clone(),
            updated_at: now,
            last_start_at: None,
            last_stop_at: None,
            launch_code: None,
        };

        self.upsert_profile(&profile)?;
        Ok(profile)
    }

    pub fn update_profile(
        &self,
        profile_id: String,
        input: BrowserProfileInput,
    ) -> Result<BrowserProfile, AppError> {
        let mut profile = self
            .get_profile(&profile_id)?
            .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;

        profile.profile_name = input.profile_name.trim().to_string();
        profile.user_data_dir = input.user_data_dir.trim().to_string();
        profile.core_id = input.core_id.trim().to_string();
        profile.fingerprint_args = input.fingerprint_args;
        profile.fingerprint = input.fingerprint;
        profile.proxy_id = input.proxy_id.trim().to_string();
        profile.proxy_config = input.proxy_config.trim().to_string();
        profile.launch_args = input.launch_args;
        profile.tags = input.tags;
        profile.keywords = input.keywords;
        profile.group_id = input.group_id.and_then(non_empty_owned);
        profile.updated_at = Utc::now().to_rfc3339();

        self.upsert_profile(&profile)?;
        Ok(profile)
    }

    pub fn delete_profile(&self, profile_id: String) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        diesel::delete(profiles::browser_profiles.filter(profiles::profile_id.eq(profile_id)))
            .execute(&mut *conn)?;
        Ok(())
    }

    fn upsert_profile(&self, profile: &BrowserProfile) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        let fingerprint_args = to_json(&profile.fingerprint_args)?;
        let fingerprint_json = to_json(&profile.fingerprint)?;
        let launch_args = to_json(&profile.launch_args)?;
        let tags = to_json(&profile.tags)?;
        let keywords = to_json(&profile.keywords)?;
        diesel::insert_into(profiles::browser_profiles)
            .values(&NewBrowserProfileRow {
                profile_id: &profile.profile_id,
                profile_name: &profile.profile_name,
                user_data_dir: &profile.user_data_dir,
                core_id: &profile.core_id,
                fingerprint_args: &fingerprint_args,
                fingerprint_json: &fingerprint_json,
                proxy_id: &profile.proxy_id,
                proxy_config: &profile.proxy_config,
                launch_args: &launch_args,
                tags: &tags,
                keywords: &keywords,
                group_id: profile.group_id.as_deref(),
                proxy_bind_source_id: profile.proxy_bind_source_id.as_deref(),
                proxy_bind_source_url: profile.proxy_bind_source_url.as_deref(),
                proxy_bind_name: profile.proxy_bind_name.as_deref(),
                proxy_bind_updated_at: profile.proxy_bind_updated_at.as_deref(),
                created_at: &profile.created_at,
                updated_at: &profile.updated_at,
            })
            .on_conflict(profiles::profile_id)
            .do_update()
            .set((
                profiles::profile_name.eq(&profile.profile_name),
                profiles::user_data_dir.eq(&profile.user_data_dir),
                profiles::core_id.eq(&profile.core_id),
                profiles::fingerprint_args.eq(&fingerprint_args),
                profiles::fingerprint_json.eq(&fingerprint_json),
                profiles::proxy_id.eq(&profile.proxy_id),
                profiles::proxy_config.eq(&profile.proxy_config),
                profiles::proxy_bind_source_id.eq(profile.proxy_bind_source_id.as_deref()),
                profiles::proxy_bind_source_url.eq(profile.proxy_bind_source_url.as_deref()),
                profiles::proxy_bind_name.eq(profile.proxy_bind_name.as_deref()),
                profiles::proxy_bind_updated_at.eq(profile.proxy_bind_updated_at.as_deref()),
                profiles::launch_args.eq(&launch_args),
                profiles::tags.eq(&tags),
                profiles::keywords.eq(&keywords),
                profiles::group_id.eq(profile.group_id.as_deref()),
                profiles::updated_at.eq(&profile.updated_at),
            ))
            .execute(&mut *conn)?;
        Ok(())
    }
}
