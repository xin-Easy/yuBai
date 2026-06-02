pub mod automation;
pub mod backup;
pub mod bookmark;
pub mod browser;
pub mod dashboard;
pub mod file;
pub mod launch_api;
pub mod proxy;
pub mod settings;
pub mod user;

pub struct Services {
    pub automation: automation::AutomationService,
    pub backup: backup::BackupService,
    pub browser_profile: browser::profile::BrowserProfileService,
    pub browser_core: browser::core::BrowserCoreService,
    pub proxy: proxy::ProxyService,
    pub browser_group: browser::group::BrowserGroupService,
    pub bookmark: bookmark::BookmarkService,
    pub browser_snapshot: browser::snapshot::BrowserSnapshotService,
    pub browser_runtime: browser::runtime::BrowserRuntimeService,
    pub dashboard: dashboard::DashboardService,
    pub file: file::FileService,
    pub settings: settings::SettingsService,
    pub launch_api_app: launch_api::LaunchApiAppService,
    pub user: user::UserService,
}

impl Services {
    pub fn new() -> Self {
        Self {
            automation: automation::AutomationService,
            backup: backup::BackupService,
            browser_profile: browser::profile::BrowserProfileService,
            browser_core: browser::core::BrowserCoreService,
            proxy: proxy::ProxyService,
            browser_group: browser::group::BrowserGroupService,
            bookmark: bookmark::BookmarkService,
            browser_snapshot: browser::snapshot::BrowserSnapshotService,
            browser_runtime: browser::runtime::BrowserRuntimeService,
            dashboard: dashboard::DashboardService,
            file: file::FileService,
            settings: settings::SettingsService,
            launch_api_app: launch_api::LaunchApiAppService,
            user: user::UserService,
        }
    }
}
