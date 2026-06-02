use crate::error::AppError;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

pub fn normalize_proxy_node_scheme(src: &str) -> String {
    let trimmed = src.trim();
    if trimmed.to_lowercase().starts_with("hysteria://") {
        format!("hysteria2://{}", &trimmed["hysteria://".len()..])
    } else {
        trimmed.to_string()
    }
}

pub fn is_supported_proxy_node(src: &str) -> bool {
    parse_proxy_to_mihomo_yaml(src).is_ok()
}

pub fn parse_proxy_to_mihomo_yaml(src: &str) -> Result<String, AppError> {
    let normalized = normalize_proxy_node_scheme(src);
    let lower = normalized.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return build_http_proxy(&normalized);
    }
    if lower.starts_with("socks5://") {
        return build_socks5_proxy(&normalized);
    }
    if lower.starts_with("socks4://") {
        return Err(AppError::validation(
            "mihomo mode does not support socks4 proxy; use auto or extension mode",
        ));
    }
    if lower.starts_with("vmess://") {
        return build_vmess_proxy(&normalized);
    }
    if lower.starts_with("vless://") {
        return build_vless_proxy(&normalized);
    }
    if lower.starts_with("trojan://") {
        return build_trojan_proxy(&normalized);
    }
    if lower.starts_with("ss://") {
        return build_shadowsocks_proxy(&normalized);
    }
    if lower.starts_with("hysteria2://") {
        return build_hysteria2_proxy(&normalized);
    }
    if lower.starts_with("tuic://") {
        return build_tuic_proxy(&normalized);
    }
    Err(AppError::validation(format!(
        "unsupported proxy protocol: {src}"
    )))
}

fn build_http_proxy(src: &str) -> Result<String, AppError> {
    let url = Url::parse(src)
        .map_err(|err| AppError::validation(format!("invalid http proxy url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    if host.is_empty() || port == 0 {
        return Err(AppError::validation(
            "http proxy is missing host or port",
        ));
    }

    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "http"),
        line(1, "server", &host),
        line_num(1, "port", port),
    ];
    append_url_auth(&mut lines, &url);
    if url.scheme().eq_ignore_ascii_case("https") {
        lines.push(line_bool(1, "tls", true));
    }
    Ok(lines.join("\n"))
}

fn build_socks5_proxy(src: &str) -> Result<String, AppError> {
    let url = Url::parse(src)
        .map_err(|err| AppError::validation(format!("invalid socks5 proxy url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    if host.is_empty() || port == 0 {
        return Err(AppError::validation(
            "socks5 proxy is missing host or port",
        ));
    }

    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "socks5"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line_bool(1, "udp", true),
    ];
    append_url_auth(&mut lines, &url);
    Ok(lines.join("\n"))
}

fn build_vmess_proxy(src: &str) -> Result<String, AppError> {
    let raw = src.trim().trim_start_matches("vmess://");
    let decoded = decode_base64(raw)?;
    let payload: Value = serde_json::from_slice(&decoded)?;
    let address = string_field(&payload, "add");
    let port = string_field(&payload, "port").parse::<u16>().unwrap_or(0);
    let uuid = string_field(&payload, "id");
    if address.is_empty() || port == 0 || uuid.is_empty() {
        return Err(AppError::validation(
            "vmess node is missing address, port or id",
        ));
    }

    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "vmess"),
        line(1, "server", &address),
        line_num(1, "port", port),
        line(1, "uuid", &uuid),
        line_num(
            1,
            "alterId",
            string_field(&payload, "aid").parse::<u32>().unwrap_or(0),
        ),
        line(
            1,
            "cipher",
            &first_non_empty(&[
                string_field(&payload, "scy"),
                string_field(&payload, "security"),
                "auto".to_string(),
            ]),
        ),
        line_bool(1, "udp", true),
    ];
    apply_v2ray_transport(
        &mut lines,
        &payload,
        Some(&first_non_empty(&[
            string_field(&payload, "sni"),
            string_field(&payload, "host"),
        ])),
    );
    Ok(lines.join("\n"))
}

fn build_vless_proxy(src: &str) -> Result<String, AppError> {
    let url =
        Url::parse(src).map_err(|err| AppError::validation(format!("invalid vless url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    let id = url.username().to_string();
    if host.is_empty() || port == 0 || id.is_empty() {
        return Err(AppError::validation(
            "vless node is missing host, port or uuid",
        ));
    }

    let query = query_map(&url);
    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "vless"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line(1, "uuid", &id),
        line_bool(1, "udp", true),
        line(1, "packet-encoding", "xudp"),
        line(1, "encryption", ""),
    ];
    if let Some(flow) = query.get("flow").filter(|value| !value.is_empty()) {
        lines.push(line(1, "flow", flow));
    }

    let security = query
        .get("security")
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    if security == "tls" || security == "reality" {
        lines.push(line_bool(1, "tls", true));
        if let Some(sni) = query.get("sni").filter(|value| !value.is_empty()) {
            lines.push(line(1, "servername", sni));
        }
        if security == "reality" {
            lines.push(indent(1, "reality-opts:"));
            if let Some(public_key) = query.get("pbk").filter(|value| !value.is_empty()) {
                lines.push(line(2, "public-key", public_key));
            }
            if let Some(short_id) = query.get("sid").filter(|value| !value.is_empty()) {
                lines.push(line(2, "short-id", short_id));
            }
        }
    }

    apply_url_transport(&mut lines, &host, &query);
    Ok(lines.join("\n"))
}

fn build_trojan_proxy(src: &str) -> Result<String, AppError> {
    let url = Url::parse(src)
        .map_err(|err| AppError::validation(format!("invalid trojan url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    let password = url.username().to_string();
    if host.is_empty() || port == 0 || password.is_empty() {
        return Err(AppError::validation(
            "trojan node is missing host, port or password",
        ));
    }

    let query = query_map(&url);
    let sni = query
        .get("sni")
        .or_else(|| query.get("peer"))
        .cloned()
        .unwrap_or_else(|| host.clone());

    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "trojan"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line(1, "password", &password),
        line_bool(1, "udp", true),
        line(1, "sni", &sni),
        line_bool(1, "skip-cert-verify", truthy(query.get("allowInsecure"))),
    ];
    apply_url_transport(&mut lines, &host, &query);
    Ok(lines.join("\n"))
}

fn build_shadowsocks_proxy(src: &str) -> Result<String, AppError> {
    let raw = src
        .trim()
        .trim_start_matches("ss://")
        .split('#')
        .next()
        .unwrap_or_default();
    let (cipher, password, host, port) = if raw.contains('@') {
        let url = Url::parse(&format!("ss://{raw}"))
            .map_err(|err| AppError::validation(format!("invalid ss url: {err}")))?;
        let user = percent_decode(url.username());
        let decoded_user = if let Some(password) = url.password() {
            format!("{user}:{}", percent_decode(password))
        } else {
            decode_base64_to_string(&user).unwrap_or(user)
        };
        let (cipher, password) = decoded_user
            .split_once(':')
            .ok_or_else(|| AppError::validation("ss user info must be method:password"))?;
        (
            cipher.to_string(),
            password.to_string(),
            url.host_str().unwrap_or_default().to_string(),
            url.port().unwrap_or(0),
        )
    } else {
        let decoded = decode_base64_to_string(raw)?;
        let (user, host_port) = decoded
            .rsplit_once('@')
            .ok_or_else(|| AppError::validation("invalid ss base64 payload"))?;
        let (cipher, password) = user
            .split_once(':')
            .ok_or_else(|| AppError::validation("ss user info must be method:password"))?;
        let (host, port) = host_port
            .rsplit_once(':')
            .ok_or_else(|| AppError::validation("ss host info must be host:port"))?;
        (
            cipher.to_string(),
            password.to_string(),
            host.to_string(),
            port.parse::<u16>().unwrap_or(0),
        )
    };
    if cipher.is_empty() || host.is_empty() || port == 0 {
        return Err(AppError::validation(
            "ss node is missing method, host or port",
        ));
    }

    Ok([
        line(0, "- name", "proxy-out"),
        line(1, "type", "ss"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line(1, "cipher", &cipher),
        line(1, "password", &password),
        line_bool(1, "udp", true),
    ]
    .join("\n"))
}

fn build_hysteria2_proxy(src: &str) -> Result<String, AppError> {
    let url = Url::parse(src)
        .map_err(|err| AppError::validation(format!("invalid hysteria2 url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    if host.is_empty() || port == 0 {
        return Err(AppError::validation(
            "hysteria2 node is missing host or port",
        ));
    }

    let query = query_map(&url);
    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "hysteria2"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line_bool(1, "udp", true),
        line(1, "password", url.username()),
        line_bool(1, "skip-cert-verify", truthy(query.get("insecure"))),
    ];
    if let Some(sni) = query
        .get("sni")
        .or_else(|| query.get("peer"))
        .filter(|value| !value.is_empty())
    {
        lines.push(line(1, "sni", sni));
    }
    if let Some(obfs_password) = query.get("obfs-password").filter(|value| !value.is_empty()) {
        lines.push(line(1, "obfs", "salamander"));
        lines.push(line(1, "obfs-password", obfs_password));
    }
    Ok(lines.join("\n"))
}

fn build_tuic_proxy(src: &str) -> Result<String, AppError> {
    let url =
        Url::parse(src).map_err(|err| AppError::validation(format!("invalid tuic url: {err}")))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let port = url.port().unwrap_or(0);
    let uuid = url.username().to_string();
    let password = url.password().unwrap_or_default().to_string();
    if host.is_empty() || port == 0 || uuid.is_empty() {
        return Err(AppError::validation(
            "tuic node is missing host, port or uuid",
        ));
    }

    let query = query_map(&url);
    let mut lines = vec![
        line(0, "- name", "proxy-out"),
        line(1, "type", "tuic"),
        line(1, "server", &host),
        line_num(1, "port", port),
        line(1, "uuid", &uuid),
        line(1, "password", &password),
        line_bool(1, "udp", true),
        line(
            1,
            "congestion-controller",
            query
                .get("congestion_control")
                .map(String::as_str)
                .unwrap_or("bbr"),
        ),
        line_bool(1, "skip-cert-verify", truthy(query.get("insecure"))),
    ];
    if let Some(sni) = query.get("sni").filter(|value| !value.is_empty()) {
        lines.push(line(1, "sni", sni));
    }
    Ok(lines.join("\n"))
}

fn apply_v2ray_transport(lines: &mut Vec<String>, payload: &Value, sni: Option<&String>) {
    if string_field(payload, "tls") == "tls" {
        lines.push(line_bool(1, "tls", true));
        if let Some(sni) = sni.filter(|value| !value.is_empty()) {
            lines.push(line(1, "servername", sni));
        }
    }

    let network = string_field(payload, "net");
    if network == "ws" {
        lines.push(line(1, "network", "ws"));
        lines.push(indent(1, "ws-opts:"));
        let path = string_field(payload, "path");
        if !path.is_empty() {
            lines.push(line(2, "path", &path));
        }
        let host = string_field(payload, "host");
        if !host.is_empty() {
            lines.push(indent(2, "headers:"));
            lines.push(line(3, "Host", &host));
        }
    }
}

fn apply_url_transport(lines: &mut Vec<String>, host: &str, query: &HashMap<String, String>) {
    let network = query
        .get("type")
        .or_else(|| query.get("network"))
        .map(String::as_str)
        .unwrap_or("");

    if network == "ws" {
        lines.push(line(1, "network", "ws"));
        lines.push(indent(1, "ws-opts:"));
        if let Some(path) = query.get("path").filter(|value| !value.is_empty()) {
            lines.push(line(2, "path", path));
        }
        let ws_host = query.get("host").map(String::as_str).unwrap_or(host);
        lines.push(indent(2, "headers:"));
        lines.push(line(3, "Host", ws_host));
    }
}

fn query_map(url: &Url) -> HashMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn append_url_auth(lines: &mut Vec<String>, url: &Url) {
    let username = percent_decode(url.username());
    if !username.is_empty() {
        lines.push(line(1, "username", &username));
    }
    if let Some(password) = url.password() {
        lines.push(line(1, "password", &percent_decode(password)));
    }
}

fn decode_base64(raw: &str) -> Result<Vec<u8>, AppError> {
    let cleaned = raw.trim();
    for engine in [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ] {
        if let Ok(data) = engine.decode(cleaned) {
            return Ok(data);
        }
    }
    Err(AppError::validation("invalid base64 content"))
}

fn decode_base64_to_string(raw: &str) -> Result<String, AppError> {
    let data = decode_base64(raw)?;
    String::from_utf8(data)
        .map_err(|err| AppError::validation(format!("invalid utf8 content: {err}")))
}

fn percent_decode(raw: &str) -> String {
    url::form_urlencoded::parse(raw.as_bytes())
        .map(|(key, value)| {
            if value.is_empty() {
                key.into_owned()
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn string_field(value: &Value, field: &str) -> String {
    match value.get(field) {
        Some(Value::String(value)) => value.trim().to_string(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn first_non_empty(values: &[String]) -> String {
    values
        .iter()
        .find(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

fn truthy(value: Option<&String>) -> bool {
    value
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn indent(level: usize, text: &str) -> String {
    format!("{}{}", "  ".repeat(level), text)
}

fn line(level: usize, key: &str, value: &str) -> String {
    format!("{}{}: {}", "  ".repeat(level), key, yaml_scalar(value))
}

fn line_num<T: std::fmt::Display>(level: usize, key: &str, value: T) -> String {
    format!("{}{}: {}", "  ".repeat(level), key, value)
}

fn line_bool(level: usize, key: &str, value: bool) -> String {
    format!(
        "{}{}: {}",
        "  ".repeat(level),
        key,
        if value { "true" } else { "false" }
    )
}

fn yaml_scalar(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('\"', "\\\""))
}
