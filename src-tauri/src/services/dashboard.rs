use crate::{
    app::state::AppState,
    domain::app::{
        AppConfigPayload, DashboardStats, DashboardWorkbench, DashboardWorkbenchAutomation,
        DashboardWorkbenchDefaultCore, DashboardWorkbenchRunningProfile, DashboardWorkbenchStats,
        DashboardWorkbenchSystem,
    },
    error::AppError,
    infra::{automation, browser, proxy::mihomo},
};

pub struct DashboardService;

impl DashboardService {
    pub fn stats(&self, state: &AppState) -> Result<DashboardStats, AppError> {
        let config = state.config_snapshot().map_err(AppError::other)?;
        state
            .repositories
            .database
            .dashboard_stats(&config, &state.app_version, &state.app_root)
    }

    pub fn workbench(&self, state: &AppState) -> Result<DashboardWorkbench, AppError> {
        let config = state.config_snapshot().map_err(AppError::other)?;
        let profiles = browser::runtime::list_profiles(state)?;
        let proxies = state.repositories.database.list_proxies()?;
        let cores = state.repositories.database.list_cores()?;
        let core_info = browser::browser_core::extended_info(state).map_err(AppError::other)?;
        let automation_state = automation::state_get(state)?;
        let script_count = automation::list_scripts(state)?.len();
        let run_count = automation::list_runs(state, 20)?.len();
        let mihomo_binary_path = mihomo::installed_binary_path(&state.app_root);
        let running_instances = profiles.iter().filter(|profile| profile.running).count();

        let running_profiles = profiles
            .iter()
            .filter(|profile| profile.running)
            .take(5)
            .map(|profile| DashboardWorkbenchRunningProfile {
                profile_id: profile.profile_id.clone(),
                profile_name: profile.profile_name.clone(),
                debug_port: profile.debug_port,
                runtime_proxy_summary: profile.runtime_proxy_summary.clone(),
                updated_at: profile.updated_at.clone(),
            })
            .collect::<Vec<_>>();

        let default_core = cores.iter().find(|core| core.is_default).map(|core| {
            let info = core_info.iter().find(|item| item.core_id == core.core_id);
            DashboardWorkbenchDefaultCore {
                core_id: core.core_id.clone(),
                core_name: core.core_name.clone(),
                path_valid: info.map(|item| item.path_valid).unwrap_or(true),
                path_message: info
                    .map(|item| item.path_message.clone())
                    .unwrap_or_default(),
            }
        });

        Ok(DashboardWorkbench {
            stats: DashboardWorkbenchStats {
                total_instances: profiles.len(),
                running_instances,
                unbound_instances: profiles
                    .iter()
                    .filter(|profile| profile.proxy_id.trim().is_empty())
                    .count(),
                proxy_count: proxies.len(),
                healthy_proxy_count: proxies.iter().filter(|proxy| proxy.last_test_ok).count(),
                untested_proxy_count: proxies
                    .iter()
                    .filter(|proxy| proxy.last_tested_at.is_none())
                    .count(),
                core_count: cores.len(),
                invalid_core_count: core_info.iter().filter(|core| !core.path_valid).count(),
                script_count,
                run_count,
            },
            system: DashboardWorkbenchSystem {
                app_version: state.app_version.clone(),
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                app_root: state.app_root.to_string_lossy().to_string(),
                proxy_mode: config.browser.proxy_mode,
                mihomo_downloaded: mihomo_binary_path.is_file(),
            },
            default_core,
            automation: DashboardWorkbenchAutomation {
                ready: automation_state.ready,
                installed: automation_state.installed,
                last_error: automation_state.last_error,
                node_version: automation_state.node_version,
                playwright_version: automation_state.playwright_version,
            },
            running_profiles,
        })
    }

    pub fn app_config(&self, state: &AppState) -> AppConfigPayload {
        AppConfigPayload {
            name: state
                .config_snapshot()
                .map(|config| config.app.name)
                .unwrap_or_else(|_| "yubai".to_string()),
            app_root: state.app_root.to_string_lossy().to_string(),
            version: state.app_version.clone(),
        }
    }
}
