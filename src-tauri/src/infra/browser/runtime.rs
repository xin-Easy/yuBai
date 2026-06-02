use crate::{
    app::state::{AppState, BrowserRuntimeEntry},
    domain::{
        browser::BrowserProfile,
        paths::resolve_app_path,
        runtime::{BrowserTabInfo, CookieInfo},
    },
    error::AppError,
    infra::{automation::util::quiet_command, logging, proxy},
};
use chrono::Utc;
use command_group::CommandGroup;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

pub fn status_profile(state: &AppState, profile_id: &str) -> Result<BrowserProfile, AppError> {
    let mut profile = state
        .repositories
        .database
        .get_profile(profile_id)?
        .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;
    let mut runtime = state.runtime_lock()?;
    let mut remove_stale = false;
    let mut stale_bridge_key = None;
    if let Some(entry) = runtime.get_mut(profile_id) {
        let child_exited = entry
            .child
            .lock()
            .map_err(|_| AppError::LockPoisoned("browser child lock poisoned".to_string()))?
            .try_wait()
            .map(|status| status.is_some())
            .unwrap_or(true);

        if !child_exited || super::cdp::probe_debug_ready(entry.debug_port) {
            let proxy_summary = profile.runtime_proxy_summary.clone();
            apply_runtime_state(
                &mut profile,
                entry.pid,
                entry.debug_port,
                proxy_summary,
            );
        } else {
            stale_bridge_key = entry.proxy_bridge_key.clone();
            remove_stale = true;
        }
    }
    if remove_stale {
        runtime.remove(profile_id);
    }
    drop(runtime);
    if stale_bridge_key.is_some() {
        state
            .proxy_bridge_lock()?
            .release_bridge(stale_bridge_key.as_deref());
    }
    if remove_stale {
        apply_stopped_state(&mut profile);
    }
    apply_automation_runtime_state(state, &mut profile);
    Ok(profile)
}

pub fn list_profiles(state: &AppState) -> Result<Vec<BrowserProfile>, AppError> {
    let profiles = state.repositories.database.list_profiles()?;
    profiles
        .into_iter()
        .map(|profile| status_profile(state, &profile.profile_id))
        .collect()
}

pub fn start_profile(state: &AppState, profile_id: String) -> Result<BrowserProfile, AppError> {
    let config = state.config_snapshot()?;
    let mut profile = state
        .repositories
        .database
        .get_profile(&profile_id)?
        .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;

    {
        let mut runtime = state.runtime_lock()?;
        let mut remove_stale = false;
        let mut stale_bridge_key = None;
        if let Some(entry) = runtime.get_mut(&profile_id) {
            let child_exited = entry
                .child
                .lock()
                .map_err(|_| AppError::LockPoisoned("browser child lock poisoned".to_string()))?
                .try_wait()
                .map_err(AppError::from)?
                .is_some();

            if !child_exited || super::cdp::probe_debug_ready(entry.debug_port) {
                let proxy_summary = profile.runtime_proxy_summary.clone();
                apply_runtime_state(
                    &mut profile,
                    entry.pid,
                    entry.debug_port,
                    proxy_summary,
                );
                return Ok(profile);
            }

            stale_bridge_key = entry.proxy_bridge_key.clone();
            remove_stale = true;
        }
        if remove_stale {
            runtime.remove(&profile_id);
        }
        drop(runtime);
        if stale_bridge_key.is_some() {
            state
                .proxy_bridge_lock()?
                .release_bridge(stale_bridge_key.as_deref());
        }
    }

    let core = state
        .repositories
        .database
        .resolve_core_for_profile(&profile.core_id)?
        .ok_or_else(|| AppError::not_found("browser core is not configured"))?;
    let browser_binary = resolve_browser_binary(&state.app_root, &core.core_path)?;
    let user_data_dir = resolve_app_path(&state.app_root, &profile.user_data_dir);
    fs::create_dir_all(&user_data_dir).map_err(AppError::from)?;

    let debug_port = allocate_debug_port()?;
    let mut command = quiet_command(browser_binary);
    command.arg(format!(
        "--user-data-dir={}",
        user_data_dir.to_string_lossy()
    ));
    command.arg(format!("--remote-debugging-port={debug_port}"));
    let proxy_resolution = resolve_instance_proxy(state, &config, &profile)?;
    let fingerprint_launch_args = super::fingerprint::launch_args(&profile.fingerprint);
    if let Some(fingerprint) = profile.fingerprint.as_ref() {
        logging::log_target(
            "info",
            "browser.fingerprint",
            "prepared browser fingerprint launch args",
            format!(
                "profileId={} brand={} platform={} language={} timezone={} resolution={}x{} generatedArgs={} manualArgs={}",
                profile.profile_id,
                fingerprint.browser_brand,
                fingerprint.os,
                fingerprint.language,
                fingerprint.timezone,
                fingerprint.resolution_width,
                fingerprint.resolution_height,
                fingerprint_launch_args.len(),
                profile.fingerprint_args.len(),
            ),
        );
    }
    append_browser_args(
        &mut command,
        &state.app_root,
        &user_data_dir,
        &proxy_resolution.extension_config,
        config
            .browser
            .default_launch_args
            .iter()
            .chain(fingerprint_launch_args.iter())
            .chain(profile.fingerprint_args.iter())
            .chain(profile.launch_args.iter()),
    )?;
    if let Some(proxy_server) = proxy_resolution.proxy_server.as_deref() {
        command.arg(format!("--proxy-server={proxy_server}"));
    }
    logging::log_target(
        "info",
        "browser.launch",
        "prepared browser launch proxy settings",
        format!(
            "profileId={} profileName={} fingerprintArgs={} proxyServer={} bridgeKey={} extensionEnabled={} extensionScheme={} extensionHost={} extensionPort={}",
            profile.profile_id,
            profile.profile_name,
            fingerprint_launch_args.len() + profile.fingerprint_args.len(),
            proxy_resolution.proxy_server.as_deref().unwrap_or("-"),
            proxy_resolution.bridge_key.as_deref().unwrap_or("-"),
            proxy_resolution.extension_config.enabled,
            if proxy_resolution.extension_config.scheme.is_empty() {
                "-"
            } else {
                proxy_resolution.extension_config.scheme.as_str()
            },
            if proxy_resolution.extension_config.host.is_empty() {
                "-"
            } else {
                proxy_resolution.extension_config.host.as_str()
            },
            proxy_resolution.extension_config.port,
        ),
    );

    let child = match command.group_spawn() {
        Ok(child) => child,
        Err(err) => {
            state
                .proxy_bridge_lock()?
                .release_bridge(proxy_resolution.bridge_key.as_deref());
            return Err(AppError::other(format!("failed to start browser: {err}")));
        }
    };
    let pid = child.id();
    let child = Arc::new(Mutex::new(child));
    let runtime_proxy_summary = proxy_runtime_summary(&proxy_resolution);
    let proxy_bridge_key = proxy_resolution.bridge_key.clone();

    state.runtime_lock()?.insert(
        profile_id,
        BrowserRuntimeEntry {
            child,
            pid,
            debug_port,
            proxy_bridge_key,
            monitor_started: false,
            started_at: Instant::now(),
        },
    );

    apply_runtime_state(
        &mut profile,
        pid,
        debug_port,
        runtime_proxy_summary,
    );
    apply_automation_runtime_state(state, &mut profile);
    Ok(profile)
}

pub fn wait_debug_ready(
    state: &AppState,
    profile_id: &str,
    timeout_ms: i32,
) -> Result<i32, AppError> {
    let timeout = Duration::from_millis(timeout_ms.max(1000) as u64);
    let started = Instant::now();
    loop {
        let (debug_port, child_exited) = {
            let mut runtime = state.runtime_lock()?;
            let entry = runtime
                .get_mut(profile_id)
                .ok_or_else(|| AppError::validation("browser instance is not running"))?;
            let child_exited = entry
                .child
                .lock()
                .map_err(|_| AppError::LockPoisoned("browser child lock poisoned".to_string()))?
                .try_wait()
                .map_err(AppError::from)?
                .is_some();
            (entry.debug_port, child_exited)
        };

        if debug_port > 0 && super::cdp::probe_debug_ready(debug_port) {
            return Ok(debug_port);
        }
        if child_exited {
            return Err(AppError::validation("browser exited before debug port became ready"));
        }
        if started.elapsed() >= timeout {
            return Err(AppError::validation(format!(
                "browser debug port is not ready after {}ms",
                timeout.as_millis()
            )));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn append_browser_args<'a>(
    command: &mut Command,
    app_root: &Path,
    user_data_dir: &Path,
    extension_proxy_config: &crate::infra::proxy::ExtensionProxyConfig,
    args: impl Iterator<Item = &'a String>,
) -> Result<(), AppError> {
    let extension_dir =
        proxy::extension_installer::ensure_installed_with_config(
            app_root,
            user_data_dir,
            extension_proxy_config,
        )?;
    let mut extension_paths = vec![extension_dir.to_string_lossy().to_string()];
    let mut disabled_features = vec!["ExtensionsToolbarMenu".to_string()];

    for arg in args {
        let trimmed = arg.trim();
        if trimmed.is_empty() || disables_extensions(trimmed) {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("--disable-features=") {
            disabled_features.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|feature| !feature.is_empty())
                    .map(ToOwned::to_owned),
            );
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("--load-extension=") {
            extension_paths.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|path| !path.is_empty())
                    .map(ToOwned::to_owned),
            );
            continue;
        }

        command.arg(arg);
    }

    dedup_preserve_order(&mut disabled_features);
    command.arg(format!("--load-extension={}", extension_paths.join(",")));
    command.arg(format!(
        "--disable-features={}",
        disabled_features.join(",")
    ));
    Ok(())
}

fn disables_extensions(arg: &str) -> bool {
    matches!(arg, "--disable-extensions" | "--disable-extensions-except")
        || arg.starts_with("--disable-extensions-except=")
}

fn dedup_preserve_order(values: &mut Vec<String>) {
    let mut deduped = Vec::with_capacity(values.len());
    for value in values.drain(..) {
        if !deduped.iter().any(|existing| existing == &value) {
            deduped.push(value);
        }
    }
    *values = deduped;
}

pub fn stop_profile(state: &AppState, profile_id: String) -> Result<BrowserProfile, AppError> {
    let mut profile = state
        .repositories
        .database
        .get_profile(&profile_id)?
        .ok_or_else(|| AppError::not_found(format!("profile not found: {profile_id}")))?;

    if let Some(entry) = state.runtime_lock()?.remove(&profile_id) {
        if let Ok(mut child) = entry.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state
            .proxy_bridge_lock()?
            .release_bridge(entry.proxy_bridge_key.as_deref());
    }

    apply_stopped_state(&mut profile);
    Ok(profile)
}

pub fn running_debug_port(state: &AppState, profile_id: &str) -> Result<i32, AppError> {
    let mut runtime = state.runtime_lock()?;
    let entry = runtime
        .get_mut(profile_id)
        .ok_or_else(|| AppError::validation("browser instance is not running"))?;
    let child_exited = entry
        .child
        .lock()
        .map_err(|_| AppError::LockPoisoned("browser child lock poisoned".to_string()))?
        .try_wait()
        .map_err(AppError::from)?
        .is_some();

    if entry.debug_port <= 0 {
        return Err(AppError::validation("debug port is not available"));
    }
    if child_exited && !super::cdp::probe_debug_ready(entry.debug_port) {
        return Err(AppError::validation("browser instance has exited"));
    }
    Ok(entry.debug_port)
}

pub fn watch_profile_exit(app: AppHandle, profile_id: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let (child, bridge_key, debug_port) = {
        let mut runtime = state.runtime_lock()?;
        let entry = runtime
            .get_mut(&profile_id)
            .ok_or_else(|| AppError::validation("browser instance is not running"))?;
        if entry.monitor_started {
            return Ok(());
        }
        entry.monitor_started = true;
        (
            Arc::clone(&entry.child),
            entry.proxy_bridge_key.clone(),
            entry.debug_port,
        )
    };

    thread::spawn(move || {
        let mut missed_debug_checks = 0;
        loop {
            let child_exited = child
                .lock()
                .ok()
                .and_then(|mut child| child.try_wait().ok())
                .flatten()
                .is_some();

            if child_exited {
                if super::cdp::probe_debug_ready(debug_port) {
                    missed_debug_checks = 0;
                } else {
                    missed_debug_checks += 1;
                    if missed_debug_checks >= 6 {
                        break;
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
        }

        let state = app.state::<AppState>();
        let should_emit = state
            .runtime_lock()
            .map(|mut runtime| runtime.remove(&profile_id).is_some())
            .unwrap_or(false);

        if should_emit {
            if let Ok(mut proxy_bridge) = state.proxy_bridge_lock() {
                proxy_bridge.release_bridge(bridge_key.as_deref());
            }

            if let Ok(Some(mut profile)) = state.repositories.database.get_profile(&profile_id) {
                apply_stopped_state(&mut profile);
                let _ = app.emit("browser:instance:exited", profile);
            }
        }
    });

    Ok(())
}

pub fn tabs_list(state: &AppState, profile_id: &str) -> Result<Vec<BrowserTabInfo>, AppError> {
    let debug_port = running_debug_port(state, profile_id)?;
    super::cdp::list_tabs(debug_port)
}

pub fn get_cookies(state: &AppState, profile_id: &str) -> Result<Vec<CookieInfo>, AppError> {
    let debug_port = running_debug_port(state, profile_id)?;
    super::cdp::get_cookies(debug_port)
}

pub fn clear_cookies(state: &AppState, profile_id: &str) -> Result<(), AppError> {
    let debug_port = running_debug_port(state, profile_id)?;
    super::cdp::clear_cookies(debug_port)
}

pub fn export_cookies(state: &AppState, profile_id: &str) -> Result<String, AppError> {
    let debug_port = running_debug_port(state, profile_id)?;
    super::cdp::export_cookies(debug_port)
}

fn apply_runtime_state(profile: &mut BrowserProfile, pid: u32, debug_port: i32, proxy_summary: String) {
    profile.running = true;
    profile.pid = pid as i32;
    profile.debug_port = debug_port;
    profile.debug_ready = super::cdp::probe_debug_ready(debug_port);
    profile.runtime_proxy_summary = proxy_summary;
}

fn apply_automation_runtime_state(state: &AppState, profile: &mut BrowserProfile) {
    profile.automation_run_id = state
        .runtimes
        .automation_queue
        .lock()
        .ok()
        .and_then(|queue| queue.profile_runs.get(&profile.profile_id).cloned());
}

fn apply_stopped_state(profile: &mut BrowserProfile) {
    let now = Utc::now().to_rfc3339();
    profile.running = false;
    profile.pid = 0;
    profile.debug_port = 0;
    profile.debug_ready = false;
    profile.automation_run_id = None;
    profile.runtime_proxy_summary = "未运行".to_string();
    profile.updated_at = now.clone();
    profile.last_stop_at = Some(now);
}

fn allocate_debug_port() -> Result<i32, AppError> {
    let port = portpicker::pick_unused_port()
        .ok_or_else(|| AppError::other("failed to allocate debug port"))?;
    Ok(i32::from(port))
}

fn resolve_instance_proxy(
    state: &AppState,
    config: &crate::domain::config::Config,
    profile: &BrowserProfile,
) -> Result<proxy::BrowserProxyResolution, AppError> {
    let proxies = state.repositories.database.list_proxies()?;
    state.proxy_bridge_lock()?.resolve_for_browser(
        config,
        &profile.proxy_config,
        &profile.proxy_id,
        &proxies,
    )
}

fn proxy_runtime_summary(resolution: &proxy::BrowserProxyResolution) -> String {
    if let Some(bridge_key) = resolution.bridge_key.as_deref().filter(|value| !value.is_empty()) {
        return format!(
            "Mihomo bridge / {} / {}",
            bridge_key,
            resolution.proxy_server.as_deref().unwrap_or("-"),
        );
    }

    if resolution.extension_config.enabled {
        return format!(
            "插件 {}://{}:{}",
            resolution.extension_config.scheme,
            resolution.extension_config.host,
            resolution.extension_config.port,
        );
    }

    if let Some(proxy_server) = resolution.proxy_server.as_deref().filter(|value| !value.is_empty()) {
        return format!("直连参数 / {proxy_server}");
    }

    "直连".to_string()
}

fn resolve_browser_binary(app_root: &Path, core_path: &str) -> Result<PathBuf, AppError> {
    let path = resolve_app_path(app_root, core_path);
    if path.is_file() {
        return Ok(path);
    }

    if path.is_dir() {
        #[cfg(target_os = "windows")]
        let candidates = ["chrome.exe", "chromium.exe"];
        #[cfg(target_os = "macos")]
        let candidates = [
            "Google Chrome.app/Contents/MacOS/Google Chrome",
            "Chromium.app/Contents/MacOS/Chromium",
            "chrome",
            "chromium",
        ];
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let candidates = ["chrome", "chromium", "chromium-browser", "google-chrome"];

        for candidate in candidates {
            let binary = path.join(candidate);
            if binary.is_file() {
                return Ok(binary);
            }
        }
    }

    Err(AppError::validation(format!(
        "invalid browser core path: {}",
        path.to_string_lossy()
    )))
}
