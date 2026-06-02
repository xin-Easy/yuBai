use crate::{
    domain::automation::BrowserProxySubscriptionFetchResult,
    error::AppError,
};
use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::Client;
use std::time::Duration;

use super::ingest::parse_proxy_candidates;

const MAX_SUBSCRIPTION_BYTES: u64 = 8 * 1024 * 1024;

pub fn fetch_clash_subscription(
    url: String,
) -> Result<BrowserProxySubscriptionFetchResult, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::validation("subscription url is required"));
    }
    let parsed = url::Url::parse(url)
        .map_err(|err| AppError::validation(format!("invalid subscription url: {err}")))?;
    let scheme = parsed.scheme().to_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(AppError::validation(
            "subscription url only supports http/https",
        ));
    }

    let client = Client::builder().timeout(Duration::from_secs(25)).build()?;
    let response = client
        .get(parsed.as_str())
        .header("User-Agent", "clash-verge/2.0 yubai/0.1")
        .header("Accept", "application/yaml,text/yaml,text/plain,*/*")
        .send()?;
    if !response.status().is_success() {
        return Err(AppError::other(format!(
            "subscription request failed: HTTP {}",
            response.status()
        )));
    }

    let body = response.bytes()?;
    if body.len() as u64 > MAX_SUBSCRIPTION_BYTES {
        return Err(AppError::validation(
            "subscription content is larger than 8MB",
        ));
    }
    let content = normalize_subscription_text(&body)?;
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&content).ok();
    let candidates = parse_proxy_candidates(&content, None, None)?;
    let suggested_group = yaml.as_ref().and_then(suggest_group_name).or_else(|| {
        parsed
            .host_str()
            .map(|host| host.trim_start_matches("www.").to_string())
    });
    let dns_servers = yaml.as_ref().and_then(extract_dns_yaml);

    Ok(BrowserProxySubscriptionFetchResult {
        url: parsed.to_string(),
        content,
        proxy_count: candidates.len(),
        dns_servers,
        suggested_group,
    })
}

fn normalize_subscription_text(body: &[u8]) -> Result<String, AppError> {
    let text = normalize_text(&String::from_utf8_lossy(body));
    let mut candidates = vec![text.clone()];
    let decoded = url::form_urlencoded::parse(text.as_bytes())
        .map(|(key, value)| {
            if value.is_empty() {
                key.into_owned()
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    if decoded != text {
        candidates.push(normalize_text(&decoded));
    }
    if let Some(decoded) = decode_base64_text(&text) {
        candidates.push(decoded);
    }
    candidates
        .into_iter()
        .find(|candidate| !candidate.trim().is_empty())
        .ok_or_else(|| AppError::validation("subscription content is empty"))
}

fn decode_base64_text(raw: &str) -> Option<String> {
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return None;
    }
    let padded = match cleaned.len() % 4 {
        0 => cleaned.to_string(),
        n => format!("{cleaned}{}", "=".repeat(4 - n)),
    };
    for candidate in [cleaned, padded.as_str()] {
        for engine in [
            &general_purpose::STANDARD,
            &general_purpose::STANDARD_NO_PAD,
            &general_purpose::URL_SAFE,
            &general_purpose::URL_SAFE_NO_PAD,
        ] {
            if let Ok(data) = engine.decode(candidate) {
                let text = normalize_text(&String::from_utf8_lossy(&data));
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn extract_dns_yaml(yaml: &serde_yaml::Value) -> Option<String> {
    let map = yaml.as_mapping()?;
    let dns = map.get(serde_yaml::Value::String("dns".to_string()))?;
    serde_yaml::to_string(&serde_yaml::Value::Mapping(
        [(serde_yaml::Value::String("dns".to_string()), dns.clone())]
            .into_iter()
            .collect(),
    ))
    .ok()
    .map(|value| value.trim().to_string())
}

fn suggest_group_name(yaml: &serde_yaml::Value) -> Option<String> {
    let map = yaml.as_mapping()?;
    let groups = map
        .get(serde_yaml::Value::String("proxy-groups".to_string()))?
        .as_sequence()?;
    groups.iter().find_map(|item| {
        item.as_mapping()
            .and_then(|map| yaml_str_opt(map, "name"))
            .and_then(crate::domain::text::non_empty_owned)
    })
}

fn yaml_str_opt(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(serde_yaml::Value::String(key.to_string()))
        .and_then(|value| match value {
            serde_yaml::Value::String(text) => Some(text.trim().to_string()),
            serde_yaml::Value::Number(number) => Some(number.to_string()),
            serde_yaml::Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
}

fn normalize_text(raw: &str) -> String {
    raw.replace("\r\n", "\n").trim().to_string()
}
