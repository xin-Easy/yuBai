use crate::{
    domain::{config::Config, browser::BrowserProxy, paths::resolve_app_path},
    error::AppError,
    infra::{
        automation::util::quiet_command,
        logging,
        proxy::{extension, mihomo, parser},
    },
};
use command_group::{CommandGroup, GroupChild};
use portpicker::pick_unused_port;
use std::{
    collections::HashMap,
    fs::{self, File},
    net::TcpStream,
    path::PathBuf,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

struct ProxyBridgeEntry {
    child: GroupChild,
    port: u16,
    ref_count: usize,
}

pub struct ProxyBridgeManager {
    app_root: PathBuf,
    bridges: HashMap<String, ProxyBridgeEntry>,
}

pub struct AcquiredBridge {
    pub bridge_key: String,
    pub port: u16,
}

impl ProxyBridgeManager {
    pub fn new(app_root: PathBuf) -> Self {
        Self {
            app_root,
            bridges: HashMap::new(),
        }
    }

    pub fn acquire_bridge(
        &mut self,
        config: &Config,
        source: &str,
        proxy_yaml: &str,
    ) -> Result<AcquiredBridge, AppError> {
        let bridge_key = format!("mihomo-{}", compute_node_key(source));
        if let Some(entry) = self.bridges.get_mut(&bridge_key) {
            if wait_port_ready(entry.port, Duration::from_millis(500)).is_ok() {
                entry.ref_count += 1;
                logging::log_target(
                    "info",
                    "proxy.bridge",
                    "reusing existing mihomo bridge",
                    format!(
                        "bridgeKey={} port={} refCount={}",
                        bridge_key,
                        entry.port,
                        entry.ref_count,
                    ),
                );
                return Ok(AcquiredBridge {
                    bridge_key,
                    port: entry.port,
                });
            }
        }

        if let Some(mut stale) = self.bridges.remove(&bridge_key) {
            logging::log_target(
                "warn",
                "proxy.bridge",
                "removing stale mihomo bridge",
                format!("bridgeKey={} port={}", bridge_key, stale.port),
            );
            let _ = stale.child.kill();
            let _ = stale.child.wait();
        }

        let port = pick_unused_port()
            .ok_or_else(|| AppError::other("failed to allocate proxy bridge port"))?;
        let workdir = self.bridge_workdir(config, &bridge_key);
        fs::create_dir_all(&workdir)?;
        logging::log_target(
            "info",
            "proxy.bridge",
            "starting mihomo bridge",
            format!(
                "bridgeKey={} port={} workdir={} source={}",
                bridge_key,
                port,
                workdir.to_string_lossy(),
                redact_source_for_log(source),
            ),
        );

        let binary = mihomo::ensure_binary(&self.app_root, config)?;
        let config_path = mihomo::write_runtime_config(&workdir, port, proxy_yaml)?;

        let mut command = quiet_command(binary);
        command.arg("-f").arg(&config_path);
        command.current_dir(&workdir);
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::from(File::create(
            workdir.join("mihomo-stderr.log"),
        )?));

        let child = command
            .group_spawn()
            .map_err(|err| AppError::other(format!("failed to start mihomo bridge: {err}")))?;

        if let Err(err) = wait_port_ready(port, Duration::from_secs(15)) {
            let mut child = child;
            let _ = child.kill();
            let _ = child.wait();
            logging::log_target(
                "error",
                "proxy.bridge",
                "mihomo bridge failed to become ready",
                format!("bridgeKey={} port={} error={err}", bridge_key, port),
            );
            return Err(AppError::other(format!(
                "mihomo bridge did not become ready: {err}"
            )));
        }

        self.bridges.insert(
            bridge_key.clone(),
            ProxyBridgeEntry {
                child,
                port,
                ref_count: 1,
            },
        );
        logging::log_target(
            "info",
            "proxy.bridge",
            "mihomo bridge ready",
            format!("bridgeKey={} port={} refCount=1", bridge_key, port),
        );

        Ok(AcquiredBridge { bridge_key, port })
    }

    pub fn release_bridge(&mut self, bridge_key: Option<&str>) {
        let Some(key) = bridge_key.map(str::trim).filter(|key| !key.is_empty()) else {
            return;
        };

        let mut should_stop = false;
        if let Some(entry) = self.bridges.get_mut(key) {
            if entry.ref_count > 0 {
                entry.ref_count -= 1;
            }
            logging::log_target(
                "info",
                "proxy.bridge",
                "released mihomo bridge reference",
                format!("bridgeKey={} port={} refCount={}", key, entry.port, entry.ref_count),
            );
            should_stop = entry.ref_count == 0;
        }

        if should_stop {
            if let Some(mut entry) = self.bridges.remove(key) {
                logging::log_target(
                    "info",
                    "proxy.bridge",
                    "stopping mihomo bridge",
                    format!("bridgeKey={} port={}", key, entry.port),
                );
                let _ = entry.child.kill();
                let _ = entry.child.wait();
            }
        }
    }

    fn bridge_workdir(&self, config: &Config, bridge_key: &str) -> PathBuf {
        let root = if config.browser.user_data_root.trim().is_empty() {
            self.app_root.join("data")
        } else {
            resolve_app_path(&self.app_root, &config.browser.user_data_root)
        };
        root.join("_proxy_bridge").join(bridge_key)
    }
}

impl Drop for ProxyBridgeManager {
    fn drop(&mut self) {
        for (_, mut entry) in self.bridges.drain() {
            let _ = entry.child.kill();
            let _ = entry.child.wait();
        }
    }
}

fn compute_node_key(src: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(src.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn wait_port_ready(port: u16, timeout: Duration) -> Result<(), AppError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(120));
    }
    Err(AppError::other(format!("127.0.0.1:{port} is not ready")))
}

fn redact_source_for_log(src: &str) -> String {
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

#[derive(Debug, Clone)]
pub struct BrowserProxyResolution {
    pub proxy_server: Option<String>,
    pub bridge_key: Option<String>,
    pub extension_config: extension::ExtensionProxyConfig,
}

impl ProxyBridgeManager {
    pub fn resolve_for_browser(
        &mut self,
        config: &Config,
        proxy_config: &str,
        proxy_id: &str,
        proxies: &[BrowserProxy],
    ) -> Result<BrowserProxyResolution, AppError> {
        let source = resolve_proxy_config(proxy_config, proxy_id, proxies);
        let source = parser::normalize_proxy_node_scheme(&source);
        let proxy_mode = config.browser.proxy_mode.trim().to_lowercase();
        let source_kind = if !proxy_id.trim().is_empty() {
            "proxy-pool"
        } else {
            "profile-inline"
        };

        logging::log_target(
            "info",
            "proxy.resolve",
            "resolving browser proxy",
            format!(
                "mode={} sourceKind={} proxyId={} source={}",
                proxy_mode,
                source_kind,
                if proxy_id.trim().is_empty() { "-" } else { proxy_id.trim() },
                redact_source_for_log(&source),
            ),
        );

        if source.is_empty() || source.eq_ignore_ascii_case("direct://") || source == "__direct__" {
            logging::log_target(
                "info",
                "proxy.resolve",
                "proxy resolved to direct connection",
                format!("mode={} sourceKind={}", proxy_mode, source_kind),
            );
            return Ok(BrowserProxyResolution {
                proxy_server: None,
                bridge_key: None,
                extension_config: extension::ExtensionProxyConfig::default(),
            });
        }

        if proxy_mode == "extension" {
            let extension_config = extension::parse_extension_proxy_config(&source)?;
            logging::log_target(
                "info",
                "proxy.resolve",
                "proxy resolved via browser extension",
                format!(
                    "source={} scheme={} host={} port={}",
                    extension_config.source,
                    extension_config.scheme,
                    extension_config.host,
                    extension_config.port,
                ),
            );
            return Ok(BrowserProxyResolution {
                proxy_server: None,
                bridge_key: None,
                extension_config,
            });
        }

        if proxy_mode == "auto" && extension::is_direct_browser_proxy(&source) {
            let extension_config = extension::parse_extension_proxy_config(&source)?;
            logging::log_target(
                "info",
                "proxy.resolve",
                "proxy resolved via direct browser proxy",
                format!(
                    "proxyServer={} scheme={} host={} port={}",
                    redact_source_for_log(&source),
                    extension_config.scheme,
                    extension_config.host,
                    extension_config.port,
                ),
            );
            return Ok(BrowserProxyResolution {
                proxy_server: Some(source.clone()),
                bridge_key: None,
                extension_config,
            });
        }

        let proxy_yaml = parser::parse_proxy_to_mihomo_yaml(&source)?;
        let bridge = self.acquire_bridge(config, &source, &proxy_yaml)?;
        let bridge_proxy = format!("socks5://127.0.0.1:{}", bridge.port);
        let extension_config = extension::parse_extension_proxy_config(&bridge_proxy)?;
        logging::log_target(
            "info",
            "proxy.resolve",
            "proxy resolved via mihomo bridge",
            format!(
                "bridgeKey={} bridgePort={} source={}",
                bridge.bridge_key,
                bridge.port,
                redact_source_for_log(&source),
            ),
        );
        Ok(BrowserProxyResolution {
            proxy_server: Some(bridge_proxy.clone()),
            bridge_key: Some(bridge.bridge_key),
            extension_config,
        })
    }
}

fn resolve_proxy_config(proxy_config: &str, proxy_id: &str, proxies: &[BrowserProxy]) -> String {
    if !proxy_id.trim().is_empty() {
        if let Some(proxy) = proxies
            .iter()
            .find(|proxy| proxy.proxy_id.eq_ignore_ascii_case(proxy_id))
        {
            return proxy.proxy_config.trim().to_string();
        }
    }
    proxy_config.trim().to_string()
}
