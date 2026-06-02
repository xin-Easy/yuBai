use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionProxyConfig {
    pub enabled: bool,
    pub mode: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub bypass_list: Vec<String>,
    pub source: String,
    pub error: String,
}

impl Default for ExtensionProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "direct".to_string(),
            scheme: String::new(),
            host: String::new(),
            port: 0,
            username: String::new(),
            password: String::new(),
            bypass_list: vec!["<local>".to_string()],
            source: String::new(),
            error: String::new(),
        }
    }
}

pub fn is_direct_browser_proxy(src: &str) -> bool {
    let lower = src.trim().to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("socks4://")
        || lower.starts_with("socks5://")
}

pub fn parse_extension_proxy_config(src: &str) -> Result<ExtensionProxyConfig, AppError> {
    let url = url::Url::parse(src).map_err(|err| {
        AppError::validation(format!("invalid proxy url for extension mode: {err}"))
    })?;
    let scheme = match url.scheme().to_lowercase().as_str() {
        "http" => "http",
        "https" => "https",
        "socks4" => "socks4",
        "socks5" => "socks5",
        other => {
            return Err(AppError::validation(format!(
                "浏览器插件代理不支持 {other} 协议，请切换 Mihomo"
            )))
        }
    };
    let host = url
        .host_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::validation("browser extension proxy missing host"))?;
    let port = url
        .port()
        .ok_or_else(|| AppError::validation("browser extension proxy missing port"))?;

    Ok(ExtensionProxyConfig {
        enabled: true,
        mode: "fixed_servers".to_string(),
        scheme: scheme.to_string(),
        host,
        port,
        username: percent_decode(url.username()),
        password: url.password().map(percent_decode).unwrap_or_default(),
        bypass_list: vec!["<local>".to_string()],
        source: redact_proxy_source(src),
        error: String::new(),
    })
}

fn percent_decode(value: &str) -> String {
    urlencoding::decode(value)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| value.to_string())
}

fn redact_proxy_source(src: &str) -> String {
    let Ok(mut url) = url::Url::parse(src) else {
        return src.to_string();
    };
    if !url.username().is_empty() {
        let _ = url.set_username("***");
    }
    if url.password().is_some() {
        let _ = url.set_password(Some("***"));
    }
    url.to_string()
}
