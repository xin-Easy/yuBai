use crate::{
    app::state::AppState,
    domain::{browser::BrowserProxy, automation::{BrowserProxyBatchTestResult, BrowserProxyTestResult}},
    error::AppError,
    infra::db::database::SYSTEM_DIRECT_PROXY_ID,
};
use chrono::Utc;
use reqwest::blocking::ClientBuilder;
use serde_json::{json, Map, Value};
use std::time::{Duration, Instant};

const DEFAULT_IP_HEALTH_URL: &str = "http://ip-api.com/json/?lang=zh-CN";
const DEFAULT_CLOUDFLARE_TRACE_URL: &str = "https://cloudflare.com/cdn-cgi/trace";

pub fn test_proxy(state: &AppState, proxy_id: String) -> Result<BrowserProxyTestResult, AppError> {
    if proxy_id == SYSTEM_DIRECT_PROXY_ID {
        return Err(AppError::validation("系统内置直连代理不需要检测"));
    }
    let proxy = state
        .repositories
        .database
        .list_proxies()?
        .into_iter()
        .find(|item| item.proxy_id == proxy_id)
        .ok_or_else(|| AppError::not_found(format!("proxy not found: {proxy_id}")))?;
    run_proxy_health_check(state, &proxy)
}

pub fn batch_test_proxies(
    state: &AppState,
    proxy_ids: Vec<String>,
) -> Result<BrowserProxyBatchTestResult, AppError> {
    let ids = if proxy_ids.is_empty() {
        state
            .repositories
            .database
            .list_proxies()?
            .into_iter()
            .map(|item| item.proxy_id)
            .collect()
    } else {
        proxy_ids
    };
    let mut items = Vec::new();
    for proxy_id in ids {
        items.push(test_proxy(state, proxy_id)?);
    }
    let ok = items.iter().filter(|item| item.ok).count();
    let total = items.len();
    Ok(BrowserProxyBatchTestResult {
        total,
        ok,
        failed: total.saturating_sub(ok),
        items,
    })
}

pub fn ip_health(state: &AppState, proxy_id: String) -> Result<BrowserProxyTestResult, AppError> {
    test_proxy(state, proxy_id)
}

fn run_proxy_health_check(
    state: &AppState,
    proxy: &BrowserProxy,
) -> Result<BrowserProxyTestResult, AppError> {
    let tested_at = Utc::now().to_rfc3339();
    let started = Instant::now();
    let test = proxy_egress_request(state, proxy, Duration::from_secs(25));
    let latency = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

    let (ok, ip_health_json, error) = match test {
        Ok(value) => (true, Some(value.to_string()), None),
        Err(err) => (false, None, Some(err.to_string())),
    };
    let latency_ms = if ok { latency } else { -1 };
    state.repositories.database.update_proxy_health(
        &proxy.proxy_id,
        latency_ms,
        ok,
        &tested_at,
        ip_health_json.as_deref(),
    )?;

    Ok(BrowserProxyTestResult {
        proxy_id: proxy.proxy_id.clone(),
        ok,
        latency_ms,
        tested_at,
        ip_health_json,
        error,
    })
}

fn proxy_egress_request(
    state: &AppState,
    proxy: &BrowserProxy,
    timeout: Duration,
) -> Result<Value, AppError> {
    let bridge = acquire_proxy_bridge(state, proxy)?;
    match request_with_proxy(
        bridge.proxy_server.as_deref(),
        DEFAULT_IP_HEALTH_URL,
        timeout,
    ) {
        Ok(mut value) => {
            if let Value::Object(ref mut map) = value {
                map.insert("provider".to_string(), Value::String("ip-api.com".to_string()));
            }
            Ok(value)
        }
        Err(ip_api_error) => match request_with_proxy(
            bridge.proxy_server.as_deref(),
            DEFAULT_CLOUDFLARE_TRACE_URL,
            timeout,
        ) {
            Ok(mut value) => {
                if let Value::Object(ref mut map) = value {
                    map.insert("provider".to_string(), Value::String("cloudflare".to_string()));
                }
                Ok(value)
            }
            Err(cloudflare_error) => Err(AppError::other(format!(
                "ip-api.com failed: {ip_api_error}; cloudflare trace failed: {cloudflare_error}"
            ))),
        },
    }
}

fn acquire_proxy_bridge<'a>(
    state: &'a AppState,
    proxy: &BrowserProxy,
) -> Result<ProxyBridgeLease<'a>, AppError> {
    let config = state.config_snapshot()?;
    let proxies = state.repositories.database.list_proxies()?;
    let resolution = state.proxy_bridge_lock()?.resolve_for_browser(
        &config,
        &proxy.proxy_config,
        &proxy.proxy_id,
        &proxies,
    )?;
    Ok(ProxyBridgeLease {
        state,
        proxy_server: resolution.proxy_server,
        bridge_key: resolution.bridge_key,
    })
}

struct ProxyBridgeLease<'a> {
    state: &'a AppState,
    proxy_server: Option<String>,
    bridge_key: Option<String>,
}

impl Drop for ProxyBridgeLease<'_> {
    fn drop(&mut self) {
        if self.bridge_key.is_none() {
            return;
        }
        if let Ok(mut manager) = self.state.proxy_bridge_lock() {
            manager.release_bridge(self.bridge_key.as_deref());
        }
    }
}

fn request_with_proxy(
    proxy_server: Option<&str>,
    target_url: &str,
    timeout: Duration,
) -> Result<Value, AppError> {
    let mut builder = ClientBuilder::new().timeout(timeout);
    if let Some(proxy) = proxy_server.filter(|value| !value.trim().is_empty()) {
        builder = builder.proxy(reqwest::Proxy::all(proxy)?);
    } else {
        builder = builder.no_proxy();
    }
    let response = builder
        .build()?
        .get(target_url)
        .header("Accept", "application/json,text/plain,*/*")
        .header("User-Agent", "Yubai/0.1")
        .send()?;
    let status = response.status();
    let body = response.text()?;
    if !status.is_success() {
        return Err(AppError::other(format!(
            "health request failed: HTTP {status}"
        )));
    }
    parse_health_body(&body)
}

fn parse_health_body(body: &str) -> Result<Value, AppError> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Ok(value);
    }
    let mut object = Map::new();
    for line in body.lines() {
        if let Some((key, value)) = line.split_once('=') {
            object.insert(
                key.trim().to_string(),
                Value::String(value.trim().to_string()),
            );
        }
    }
    if !object.is_empty() {
        return Ok(Value::Object(object));
    }
    Ok(json!({ "body": body.trim() }))
}
