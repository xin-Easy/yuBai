use crate::{
    app::state::AppState,
    domain::{
        automation::{BrowserProxyImportInput, BrowserProxyImportResult},
        browser::{BrowserProxyInput},
        text::non_empty_owned,
    },
    error::AppError,
};
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::collections::HashSet;

use crate::infra::proxy::parser::{is_supported_proxy_node, normalize_proxy_node_scheme};

#[derive(Debug, Clone)]
pub struct ProxyCandidate {
    pub name: String,
    pub config: String,
    pub group_name: Option<String>,
}

pub fn import_text(
    state: &AppState,
    input: BrowserProxyImportInput,
) -> Result<BrowserProxyImportResult, AppError> {
    let source_id = input.source_id.clone().and_then(non_empty_owned);
    let source_url = input.source_url.clone().and_then(non_empty_owned);
    let source_name_prefix = input.source_name_prefix.clone().and_then(non_empty_owned);

    let candidates = parse_proxy_candidates(
        &input.content,
        input.group_name.as_deref(),
        input.name_prefix.as_deref(),
    )?;
    let mut result = BrowserProxyImportResult {
        imported: 0,
        skipped: 0,
        failed: 0,
        items: Vec::new(),
        errors: Vec::new(),
    };
    let existing = state.repositories.database.list_proxies()?;
    let mut seen: HashSet<String> = existing
        .iter()
        .map(|item| item.proxy_config.trim().to_string())
        .collect();

    for candidate in candidates {
        if !seen.insert(candidate.config.clone()) {
            result.skipped += 1;
            continue;
        }
        match state.repositories.database.save_proxy(BrowserProxyInput {
            proxy_id: None,
            proxy_name: candidate.name,
            proxy_config: candidate.config,
            dns_servers: input.dns_servers.clone().and_then(non_empty_owned),
            group_name: candidate
                .group_name
                .or_else(|| input.group_name.clone())
                .and_then(non_empty_owned),
            source_id: source_id.clone(),
            source_url: source_url.clone(),
            source_name_prefix: source_name_prefix.clone(),
        }) {
            Ok(proxy) => {
                result.imported += 1;
                result.items.push(proxy);
            }
            Err(err) => {
                result.failed += 1;
                result.errors.push(err.to_string());
            }
        }
    }

    Ok(result)
}

pub fn parse_proxy_candidates(
    content: &str,
    group_name: Option<&str>,
    name_prefix: Option<&str>,
) -> Result<Vec<ProxyCandidate>, AppError> {
    let mut out = Vec::new();
    let normalized = normalize_text(content);
    parse_uri_lines(&normalized, group_name, name_prefix, &mut out);
    if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&normalized) {
        parse_yaml_candidates(&yaml, group_name, name_prefix, &mut out)?;
    }

    let mut seen = HashSet::new();
    out.retain(|item| seen.insert(item.config.clone()));
    if out.is_empty() {
        return Err(AppError::validation("no supported proxy nodes were found"));
    }
    Ok(out)
}

fn parse_uri_lines(
    content: &str,
    group_name: Option<&str>,
    name_prefix: Option<&str>,
    out: &mut Vec<ProxyCandidate>,
) {
    for (index, line) in content.lines().enumerate() {
        for piece in line.split_whitespace() {
            let node =
                normalize_proxy_node_scheme(piece.trim().trim_matches('"').trim_matches('\''));
            if is_supported_proxy_node(&node) {
                out.push(ProxyCandidate {
                    name: candidate_name(&node, index + 1, name_prefix, None),
                    config: node,
                    group_name: group_name.map(str::to_string),
                });
            }
        }
    }
}

fn parse_yaml_candidates(
    yaml: &serde_yaml::Value,
    group_name: Option<&str>,
    name_prefix: Option<&str>,
    out: &mut Vec<ProxyCandidate>,
) -> Result<(), AppError> {
    let Some(nodes) = clash_proxy_nodes(yaml) else {
        return Ok(());
    };
    for (index, node) in nodes.iter().enumerate() {
        if let Some(text) = node.as_str() {
            let normalized = normalize_proxy_node_scheme(text);
            if is_supported_proxy_node(&normalized) {
                out.push(ProxyCandidate {
                    name: candidate_name(&normalized, index + 1, name_prefix, None),
                    config: normalized,
                    group_name: group_name.map(str::to_string),
                });
            }
            continue;
        }
        let Some(map) = node.as_mapping() else {
            continue;
        };
        if let Some(candidate) = clash_map_to_candidate(map, index + 1, group_name, name_prefix)? {
            out.push(candidate);
        }
    }
    Ok(())
}

fn clash_map_to_candidate(
    map: &serde_yaml::Mapping,
    index: usize,
    group_name: Option<&str>,
    name_prefix: Option<&str>,
) -> Result<Option<ProxyCandidate>, AppError> {
    let proxy_type = yaml_str(map, "type").to_lowercase();
    if proxy_type.is_empty() {
        return Ok(None);
    }
    let name = yaml_str(map, "name");
    let server = yaml_str(map, "server");
    let port = yaml_u16(map, "port");
    let node = match proxy_type.as_str() {
        "http" => {
            let auth = proxy_auth(map);
            format!("http://{auth}{server}:{port}")
        }
        "socks5" | "socks" => {
            let auth = proxy_auth(map);
            format!("socks5://{auth}{server}:{port}")
        }
        "ss" | "shadowsocks" => {
            let method = yaml_str(map, "cipher");
            let password = yaml_str(map, "password");
            let user = general_purpose::URL_SAFE_NO_PAD.encode(format!("{method}:{password}"));
            format!("ss://{user}@{server}:{port}#{}", urlencoding(&name))
        }
        "trojan" => {
            let password = yaml_str(map, "password");
            let mut node = format!("trojan://{}@{}:{}", urlencoding(&password), server, port);
            append_common_query(
                map,
                &mut node,
                &["sni", "peer", "allowInsecure", "type", "path", "host"],
            );
            node.push('#');
            node.push_str(&urlencoding(&name));
            node
        }
        "vmess" => build_vmess_uri(map, &name, &server, port),
        "vless" => build_vless_uri(map, &name, &server, port),
        "hysteria2" | "hysteria" => build_hysteria2_uri(map, &name, &server, port),
        "tuic" => build_tuic_uri(map, &name, &server, port),
        _ => return Ok(None),
    };
    if server.is_empty() || port == 0 || !is_supported_proxy_node(&node) {
        return Ok(None);
    }
    Ok(Some(ProxyCandidate {
        name: candidate_name(&node, index, name_prefix, Some(&name)),
        config: normalize_proxy_node_scheme(&node),
        group_name: group_name.map(str::to_string),
    }))
}

fn build_vmess_uri(map: &serde_yaml::Mapping, name: &str, server: &str, port: u16) -> String {
    let network = first_yaml_str(map, &["network", "net"]);
    let ws_path = yaml_nested_str(map, "ws-opts", "path")
        .or_else(|| yaml_str_opt(map, "path"))
        .unwrap_or_default();
    let ws_host = yaml_nested_str(map, "ws-opts", "headers.Host")
        .or_else(|| yaml_nested_str(map, "ws-opts", "headers.host"))
        .or_else(|| yaml_str_opt(map, "host"))
        .unwrap_or_default();
    let payload = json!({
        "v": "2",
        "ps": name,
        "add": server,
        "port": port.to_string(),
        "id": first_yaml_str(map, &["uuid", "id"]),
        "aid": first_yaml_str(map, &["alterId", "alter-id", "aid"]),
        "scy": first_yaml_str(map, &["cipher", "security"]),
        "net": network,
        "type": first_yaml_str(map, &["network-type", "headerType", "type"]),
        "host": ws_host,
        "path": ws_path,
        "tls": if yaml_bool(map, "tls") { "tls" } else { "" },
        "sni": first_yaml_str(map, &["servername", "sni"])
    });
    let raw = serde_json::to_string(&payload).unwrap_or_default();
    format!("vmess://{}", general_purpose::STANDARD_NO_PAD.encode(raw))
}

fn build_vless_uri(map: &serde_yaml::Mapping, name: &str, server: &str, port: u16) -> String {
    let uuid = first_yaml_str(map, &["uuid", "id"]);
    let mut params = Vec::new();
    let security = if yaml_bool(map, "tls") {
        "tls".to_string()
    } else {
        first_yaml_str(map, &["security"])
    };
    push_query(&mut params, "security", &security);
    push_query(&mut params, "flow", &yaml_str(map, "flow"));
    push_query(&mut params, "sni", &first_yaml_str(map, &["servername", "sni"]));
    let network = first_yaml_str(map, &["network", "type"]);
    push_query(&mut params, "type", &network);
    if network == "ws" {
        push_query(
            &mut params,
            "path",
            &yaml_nested_str(map, "ws-opts", "path").unwrap_or_default(),
        );
        push_query(
            &mut params,
            "host",
            &yaml_nested_str(map, "ws-opts", "headers.Host")
                .or_else(|| yaml_nested_str(map, "ws-opts", "headers.host"))
                .unwrap_or_default(),
        );
    }
    format!(
        "vless://{}@{}:{}{}#{}",
        urlencoding(&uuid),
        server,
        port,
        format_query(&params),
        urlencoding(name)
    )
}

fn build_hysteria2_uri(map: &serde_yaml::Mapping, name: &str, server: &str, port: u16) -> String {
    let password = yaml_str(map, "password");
    let mut params = Vec::new();
    push_query(&mut params, "sni", &first_yaml_str(map, &["sni", "servername"]));
    if yaml_bool(map, "skip-cert-verify") || yaml_bool(map, "insecure") {
        push_query(&mut params, "insecure", "1");
    }
    push_query(
        &mut params,
        "obfs-password",
        &first_yaml_str(map, &["obfs-password", "obfs"]),
    );
    format!(
        "hysteria2://{}@{}:{}{}#{}",
        urlencoding(&password),
        server,
        port,
        format_query(&params),
        urlencoding(name)
    )
}

fn build_tuic_uri(map: &serde_yaml::Mapping, name: &str, server: &str, port: u16) -> String {
    let uuid = first_yaml_str(map, &["uuid", "id"]);
    let password = yaml_str(map, "password");
    let mut params = Vec::new();
    push_query(&mut params, "sni", &first_yaml_str(map, &["sni", "servername"]));
    push_query(
        &mut params,
        "congestion_control",
        &first_yaml_str(map, &["congestion-controller", "congestion_control"]),
    );
    if yaml_bool(map, "skip-cert-verify") || yaml_bool(map, "insecure") {
        push_query(&mut params, "insecure", "1");
    }
    format!(
        "tuic://{}:{}@{}:{}{}#{}",
        urlencoding(&uuid),
        urlencoding(&password),
        server,
        port,
        format_query(&params),
        urlencoding(name)
    )
}

fn clash_proxy_nodes(yaml: &serde_yaml::Value) -> Option<&Vec<serde_yaml::Value>> {
    if let Some(items) = yaml.as_sequence() {
        return Some(items);
    }
    let map = yaml.as_mapping()?;
    for key in ["proxies", "proxy", "Proxy"] {
        if let Some(value) = map.get(serde_yaml::Value::String(key.to_string())) {
            if let Some(items) = value.as_sequence() {
                return Some(items);
            }
        }
    }
    None
}

fn candidate_name(
    node: &str,
    index: usize,
    prefix: Option<&str>,
    explicit_name: Option<&str>,
) -> String {
    let explicit = explicit_name.and_then(|value| non_empty_owned(value.to_string()));
    let fragment = url::Url::parse(node)
        .ok()
        .and_then(|url| url.fragment().map(|value| value.to_string()));
    let base = explicit
        .or(fragment)
        .or_else(|| {
            url::Url::parse(node)
                .ok()
                .and_then(|url| url.host_str().map(|host| host.to_string()))
        })
        .unwrap_or_else(|| format!("Proxy {index}"));
    match prefix.and_then(|value| non_empty_owned(value.to_string())) {
        Some(prefix) => format!("{prefix} {base}"),
        None => base,
    }
}

fn proxy_auth(map: &serde_yaml::Mapping) -> String {
    let username = yaml_str(map, "username");
    let password = yaml_str(map, "password");
    if username.is_empty() {
        String::new()
    } else {
        format!("{}:{}@", urlencoding(&username), urlencoding(&password))
    }
}

fn append_common_query(map: &serde_yaml::Mapping, node: &mut String, fields: &[&str]) {
    let mut params = Vec::new();
    for field in fields {
        if let Some(value) = yaml_str_opt(map, field).and_then(non_empty_owned) {
            params.push(format!("{}={}", field, urlencoding(&value)));
        }
    }
    if !params.is_empty() {
        node.push('?');
        node.push_str(&params.join("&"));
    }
}

fn push_query(params: &mut Vec<String>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        params.push(format!("{}={}", key, urlencoding(value.trim())));
    }
}

fn format_query(params: &[String]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

fn first_yaml_str(map: &serde_yaml::Mapping, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| yaml_str_opt(map, key).and_then(non_empty_owned))
        .unwrap_or_default()
}

fn yaml_bool(map: &serde_yaml::Mapping, key: &str) -> bool {
    map.get(serde_yaml::Value::String(key.to_string()))
        .and_then(|value| match value {
            serde_yaml::Value::Bool(value) => Some(*value),
            serde_yaml::Value::Number(number) => Some(number.as_i64().unwrap_or(0) != 0),
            serde_yaml::Value::String(text) => {
                let lower = text.trim().to_lowercase();
                Some(lower == "true" || lower == "1" || lower == "yes")
            }
            _ => None,
        })
        .unwrap_or(false)
}

fn yaml_nested_str(map: &serde_yaml::Mapping, root: &str, path: &str) -> Option<String> {
    let mut current = map.get(serde_yaml::Value::String(root.to_string()))?;
    for part in path.split('.') {
        current = current
            .as_mapping()?
            .get(serde_yaml::Value::String(part.to_string()))?;
    }
    match current {
        serde_yaml::Value::String(text) => Some(text.trim().to_string()),
        serde_yaml::Value::Number(number) => Some(number.to_string()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn yaml_str(map: &serde_yaml::Mapping, key: &str) -> String {
    yaml_str_opt(map, key).unwrap_or_default()
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

fn yaml_u16(map: &serde_yaml::Mapping, key: &str) -> u16 {
    yaml_str(map, key).parse::<u16>().unwrap_or(0)
}

fn normalize_text(raw: &str) -> String {
    raw.replace("\r\n", "\n").trim().to_string()
}

fn urlencoding(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}
