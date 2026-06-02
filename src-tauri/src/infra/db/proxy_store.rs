use crate::{
    infra::db::{
        database::{Database, SYSTEM_DIRECT_PROXY_CONFIG, SYSTEM_DIRECT_PROXY_ID, SYSTEM_DIRECT_PROXY_NAME},
        support::{map_proxy_row, BrowserProxyRow, NewBrowserProxyRow},
    },
    domain::{
        browser::{BrowserProxy, BrowserProxyInput},
        text::non_empty_owned,
    },
    error::AppError,
};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

impl Database {
    pub fn list_proxies(&self) -> Result<Vec<BrowserProxy>, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        self.ensure_system_direct_proxy()?;
        let mut conn = self.lock()?;
        let rows = proxies::browser_proxies
            .select(BrowserProxyRow::as_select())
            .order((proxies::sort_order.asc(), proxies::created_at.asc()))
            .load::<BrowserProxyRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_proxy_row).collect())
    }

    pub fn save_proxy(&self, input: BrowserProxyInput) -> Result<BrowserProxy, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let proxy_id = input
            .proxy_id
            .and_then(non_empty_owned)
            .unwrap_or_else(|| format!("proxy-{}", Uuid::new_v4()));
        if proxy_id == SYSTEM_DIRECT_PROXY_ID {
            return Err(AppError::validation("系统内置直连代理不可修改"));
        }
        let now = Utc::now().to_rfc3339();
        let source_id = input.source_id.and_then(non_empty_owned);
        let source_url = input.source_url.and_then(non_empty_owned);
        let source_name_prefix = input.source_name_prefix.and_then(non_empty_owned);
        let source_auto_refresh = source_id.is_some();
        let proxy = BrowserProxy {
            proxy_id: proxy_id.clone(),
            proxy_name: input.proxy_name.trim().to_string(),
            proxy_config: input.proxy_config.trim().to_string(),
            dns_servers: input.dns_servers.and_then(non_empty_owned),
            group_name: input.group_name.and_then(non_empty_owned),
            source_id: source_id.clone(),
            source_url: source_url.clone(),
            source_name_prefix: source_name_prefix.clone(),
            source_auto_refresh,
            source_refresh_interval_m: 0,
            source_last_refresh_at: if source_id.is_some() { Some(now) } else { None },
            last_latency_ms: -1,
            last_test_ok: false,
            last_tested_at: None,
            last_ip_health_json: None,
            bound_profile_names: Vec::new(),
        };

        let mut conn = self.lock()?;
        diesel::insert_into(proxies::browser_proxies)
            .values(&NewBrowserProxyRow {
                proxy_id: &proxy.proxy_id,
                proxy_name: &proxy.proxy_name,
                proxy_config: &proxy.proxy_config,
                dns_servers: proxy.dns_servers.as_deref(),
                group_name: proxy.group_name.as_deref(),
                last_latency_ms: proxy.last_latency_ms,
                last_test_ok: if proxy.last_test_ok { 1 } else { 0 },
                last_tested_at: proxy.last_tested_at.as_deref(),
                last_ip_health_json: proxy.last_ip_health_json.as_deref().unwrap_or(""),
                source_id: proxy.source_id.as_deref(),
                source_url: proxy.source_url.as_deref(),
                source_name_prefix: proxy.source_name_prefix.as_deref(),
                source_auto_refresh: if proxy.source_auto_refresh { 1 } else { 0 },
                source_refresh_interval_m: proxy.source_refresh_interval_m,
                source_last_refresh_at: proxy.source_last_refresh_at.as_deref(),
            })
            .on_conflict(proxies::proxy_id)
            .do_update()
            .set((
                proxies::proxy_name.eq(&proxy.proxy_name),
                proxies::proxy_config.eq(&proxy.proxy_config),
                proxies::dns_servers.eq(proxy.dns_servers.as_deref()),
                proxies::group_name.eq(proxy.group_name.as_deref()),
                proxies::source_id.eq(proxy.source_id.as_deref()),
                proxies::source_url.eq(proxy.source_url.as_deref()),
                proxies::source_name_prefix.eq(proxy.source_name_prefix.as_deref()),
                proxies::source_auto_refresh.eq(if proxy.source_auto_refresh { 1 } else { 0 }),
                proxies::source_refresh_interval_m.eq(proxy.source_refresh_interval_m),
                proxies::source_last_refresh_at.eq(proxy.source_last_refresh_at.as_deref()),
            ))
            .execute(&mut *conn)?;

        Ok(proxy)
    }

    pub fn delete_proxy(&self, proxy_id: String) -> Result<(), AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        if proxy_id == SYSTEM_DIRECT_PROXY_ID {
            return Err(AppError::validation("系统内置直连代理不可删除"));
        }
        let mut conn = self.lock()?;
        diesel::delete(proxies::browser_proxies.filter(proxies::proxy_id.eq(proxy_id)))
            .execute(&mut *conn)?;
        Ok(())
    }

    pub fn ensure_system_direct_proxy(&self) -> Result<BrowserProxy, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let mut conn = self.lock()?;
        diesel::insert_into(proxies::browser_proxies)
            .values(&NewBrowserProxyRow {
                proxy_id: SYSTEM_DIRECT_PROXY_ID,
                proxy_name: SYSTEM_DIRECT_PROXY_NAME,
                proxy_config: SYSTEM_DIRECT_PROXY_CONFIG,
                dns_servers: None,
                group_name: None,
                last_latency_ms: -1,
                last_test_ok: 0,
                last_tested_at: None,
                last_ip_health_json: "",
                source_id: None,
                source_url: None,
                source_name_prefix: None,
                source_auto_refresh: 0,
                source_refresh_interval_m: 0,
                source_last_refresh_at: None,
            })
            .on_conflict(proxies::proxy_id)
            .do_update()
            .set((
                proxies::proxy_name.eq(SYSTEM_DIRECT_PROXY_NAME),
                proxies::proxy_config.eq(SYSTEM_DIRECT_PROXY_CONFIG),
                proxies::dns_servers.eq::<Option<&str>>(None),
                proxies::group_name.eq::<Option<&str>>(None),
                proxies::source_id.eq::<Option<&str>>(None),
                proxies::source_url.eq::<Option<&str>>(None),
                proxies::source_name_prefix.eq::<Option<&str>>(None),
                proxies::source_auto_refresh.eq(0),
                proxies::source_refresh_interval_m.eq(0),
                proxies::source_last_refresh_at.eq::<Option<&str>>(None),
            ))
            .execute(&mut *conn)?;

        Ok(BrowserProxy {
            proxy_id: SYSTEM_DIRECT_PROXY_ID.to_string(),
            proxy_name: SYSTEM_DIRECT_PROXY_NAME.to_string(),
            proxy_config: SYSTEM_DIRECT_PROXY_CONFIG.to_string(),
            dns_servers: None,
            group_name: None,
            source_id: None,
            source_url: None,
            source_name_prefix: None,
            source_auto_refresh: false,
            source_refresh_interval_m: 0,
            source_last_refresh_at: None,
            last_latency_ms: -1,
            last_test_ok: false,
            last_tested_at: None,
            last_ip_health_json: None,
            bound_profile_names: Vec::new(),
        })
    }

    pub fn delete_proxies_by_source(&self, source_id: &str) -> Result<usize, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let mut conn = self.lock()?;
        let count =
            diesel::delete(proxies::browser_proxies.filter(proxies::source_id.eq(source_id)))
                .execute(&mut *conn)?;
        Ok(count)
    }

    pub fn list_proxies_by_source(&self, source_id: &str) -> Result<Vec<BrowserProxy>, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let mut conn = self.lock()?;
        let rows = proxies::browser_proxies
            .filter(proxies::source_id.eq(source_id))
            .select(BrowserProxyRow::as_select())
            .order((proxies::sort_order.asc(), proxies::created_at.asc()))
            .load::<BrowserProxyRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_proxy_row).collect())
    }

    #[allow(dead_code)]
    pub fn list_subscription_sources(&self) -> Result<Vec<BrowserProxy>, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let mut conn = self.lock()?;
        let rows = proxies::browser_proxies
            .filter(proxies::source_url.ne::<&str>(""))
            .select(BrowserProxyRow::as_select())
            .load::<BrowserProxyRow>(&mut *conn)?;

        Ok(rows.into_iter().map(map_proxy_row).collect())
    }

    pub fn find_profile_names_by_proxy_id(&self, proxy_id: &str) -> Result<Vec<String>, AppError> {
        use crate::infra::db::schema::browser_profiles::dsl as profiles;

        let mut conn = self.lock()?;
        let names: Vec<String> = profiles::browser_profiles
            .filter(profiles::proxy_id.eq(proxy_id))
            .select(profiles::profile_name)
            .load(&mut *conn)?;

        Ok(names)
    }

    pub fn update_proxy_health(
        &self,
        proxy_id: &str,
        latency_ms: i64,
        ok: bool,
        tested_at: &str,
        ip_health_json: Option<&str>,
    ) -> Result<Option<BrowserProxy>, AppError> {
        use crate::infra::db::schema::browser_proxies::dsl as proxies;

        let mut conn = self.lock()?;
        let ip_health_json = ip_health_json.unwrap_or("");
        diesel::update(proxies::browser_proxies.filter(proxies::proxy_id.eq(proxy_id)))
            .set((
                proxies::last_latency_ms.eq(latency_ms),
                proxies::last_test_ok.eq(if ok { 1 } else { 0 }),
                proxies::last_tested_at.eq(tested_at),
                proxies::last_ip_health_json.eq(ip_health_json),
            ))
            .execute(&mut *conn)?;

        let row = proxies::browser_proxies
            .filter(proxies::proxy_id.eq(proxy_id))
            .select(BrowserProxyRow::as_select())
            .first::<BrowserProxyRow>(&mut *conn)
            .optional()?;
        Ok(row.map(map_proxy_row))
    }
}
