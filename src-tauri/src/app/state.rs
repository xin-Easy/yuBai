use crate::{
    domain::config::{load_config, Config},
    error::AppError,
    infra::{
        automation::AutomationQueuedTask, launch_api::LaunchApiService,
        db::database::Database,
        proxy::ProxyBridgeManager,
    },
    services::Services,
};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    process::Child,
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

use command_group::GroupChild;

pub struct AppState {
    pub app_root: PathBuf,
    pub app_version: String,
    pub config: ConfigStore,
    pub repositories: Repositories,
    pub services: Services,
    pub runtimes: RuntimeManagers,
    pub launch_api: LaunchApiState,
}

pub struct BrowserRuntimeEntry {
    pub child: Arc<Mutex<GroupChild>>,
    pub pid: u32,
    pub debug_port: i32,
    pub proxy_bridge_key: Option<String>,
    pub monitor_started: bool,
    #[allow(dead_code)]
    pub started_at: Instant,
}

#[derive(Default)]
pub struct AutomationQueueState {
    pub worker_running: bool,
    pub pending: VecDeque<AutomationQueuedTask>,
    pub running: HashMap<String, Arc<Mutex<Child>>>,
    pub profile_runs: HashMap<String, String>,
    pub cancelled: std::collections::HashSet<String>,
}

pub struct ConfigStore {
    inner: RwLock<Config>,
}

pub struct Repositories {
    pub database: Database,
}

pub struct RuntimeManagers {
    pub browser_runtime: Mutex<HashMap<String, BrowserRuntimeEntry>>,
    pub automation_queue: Arc<Mutex<AutomationQueueState>>,
    pub proxy_bridge: Mutex<ProxyBridgeManager>,
}

pub struct LaunchApiState {
    pub service: LaunchApiService,
}

impl AppState {
    pub fn new(app_root: PathBuf) -> Self {
        let config = load_config(&app_root);
        let database = Database::open(&app_root, &config).expect("failed to initialize database");

        Self {
            app_root: app_root.clone(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            config: ConfigStore::new(config),
            repositories: Repositories { database },
            services: Services::new(),
            runtimes: RuntimeManagers {
                browser_runtime: Mutex::new(HashMap::new()),
                automation_queue: Arc::new(Mutex::new(AutomationQueueState::default())),
                proxy_bridge: Mutex::new(ProxyBridgeManager::new(app_root.clone())),
            },
            launch_api: LaunchApiState {
                service: LaunchApiService::new(app_root.clone()),
            },
        }
    }

    pub fn config_snapshot(&self) -> Result<Config, String> {
        self.config.snapshot()
    }

    pub fn replace_config(&self, config: Config) -> Result<(), String> {
        self.config.replace(&self.app_root, config)
    }

    pub fn runtime_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, BrowserRuntimeEntry>>, String> {
        self.runtimes
            .browser_runtime
            .lock()
            .map_err(|_| "浏览器运行态锁已损坏".to_string())
    }

    pub fn proxy_bridge_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, ProxyBridgeManager>, AppError> {
        self.runtimes
            .proxy_bridge
            .lock()
            .map_err(|_| AppError::LockPoisoned("proxy bridge lock poisoned".to_string()))
    }
}

impl ConfigStore {
    pub fn new(config: Config) -> Self {
        Self {
            inner: RwLock::new(config),
        }
    }

    pub fn snapshot(&self) -> Result<Config, String> {
        self.inner
            .read()
            .map_err(|_| "config lock poisoned".to_string())
            .map(|guard| guard.clone())
    }

    pub fn replace(&self, app_root: &PathBuf, config: Config) -> Result<(), String> {
        let config_path = app_root.join("config.yaml");
        let existing_raw = std::fs::read_to_string(&config_path).unwrap_or_default();
        let mut existing = serde_yaml::from_str::<serde_yaml::Value>(&existing_raw)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(Default::default()));
        let next = serde_yaml::to_value(&config)
            .map_err(|err| format!("failed to serialize config: {err}"))?;

        merge_config_sections(&mut existing, next, &["database", "app", "browser", "automation"])?;

        let raw = serde_yaml::to_string(&existing)
            .map_err(|err| format!("failed to render config: {err}"))?;
        std::fs::write(config_path, raw)
            .map_err(|err| format!("failed to write config.yaml: {err}"))?;
        let mut guard = self
            .inner
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        *guard = config;
        Ok(())
    }
}

fn merge_config_sections(
    target: &mut serde_yaml::Value,
    source: serde_yaml::Value,
    keys: &[&str],
) -> Result<(), String> {
    let target_map = target
        .as_mapping_mut()
        .ok_or_else(|| "config root is not a mapping".to_string())?;
    let source_map = source
        .as_mapping()
        .ok_or_else(|| "new config root is not a mapping".to_string())?;

    for key in keys {
        let yaml_key = serde_yaml::Value::String((*key).to_string());
        if let Some(value) = source_map.get(&yaml_key) {
            target_map.insert(yaml_key, value.clone());
        }
    }

    Ok(())
}
