use crate::{
    app::state::AppState,
    domain::{browser::BrowserProfile, runtime::LaunchApiLogEntry},
    error::AppError,
    infra::{browser::runtime, logging},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager};
use tower_http::trace::TraceLayer;

const DEFAULT_LAUNCH_PORT: u16 = 19876;

#[derive(Clone)]
pub struct LaunchApiService {
    state: Arc<Mutex<LaunchApiState>>,
}

#[derive(Default)]
struct LaunchApiState {
    enabled: bool,
    port: u16,
    join: Option<tauri::async_runtime::JoinHandle<()>>,
    active_profile_id: Option<String>,
    active_debug_port: i32,
    logs: Vec<LaunchApiLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchApiConfig {
    pub enabled: bool,
    pub port: u16,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub profile_id: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchApiHealth {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchApiRuntimePayload {
    pub ok: bool,
    pub profile: Option<BrowserProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchApiLaunchResponse {
    pub ok: bool,
    pub profile: Option<BrowserProfile>,
    pub launch_code: Option<String>,
}

impl LaunchApiService {
    pub fn new(app_root: PathBuf) -> Self {
        let _ = app_root;
        Self {
            state: Arc::new(Mutex::new(LaunchApiState::default())),
        }
    }

    pub fn start(&self, app: AppHandle, port: u16) -> Result<(), AppError> {
        let port = if port == 0 { DEFAULT_LAUNCH_PORT } else { port };
        let mut guard = self
            .state
            .lock()
            .map_err(|_| AppError::LockPoisoned("launch api lock poisoned".to_string()))?;
        if guard.enabled && guard.port == port {
            return Ok(());
        }
        if let Some(handle) = guard.join.take() {
            handle.abort();
        }

        let router = Self::router(app.clone(), self.state.clone());
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let join = tauri::async_runtime::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => listener,
                Err(err) => {
                    logging::log_target(
                        "error",
                        "launch-api",
                        "launch api bind failed",
                        err.to_string(),
                    );
                    return;
                }
            };
            if let Err(err) = axum::serve(listener, router).await {
                logging::log_target(
                    "error",
                    "launch-api",
                    "launch api stopped",
                    err.to_string(),
                );
            }
        });

        guard.enabled = true;
        guard.port = port;
        guard.join = Some(join);
        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.state.lock() {
            if let Some(handle) = guard.join.take() {
                handle.abort();
            }
            guard.enabled = false;
        }
    }

    pub fn port(&self) -> u16 {
        self.state.lock().map(|guard| guard.port).unwrap_or(0)
    }

    pub fn enabled(&self) -> bool {
        self.state
            .lock()
            .map(|guard| guard.enabled)
            .unwrap_or(false)
    }

    fn router(app: AppHandle, api_state: Arc<Mutex<LaunchApiState>>) -> Router {
        Router::new()
            .route("/api/health", get(Self::health))
            .route("/api/profiles", get(Self::profiles))
            .route("/api/profiles/{id}", get(Self::profile))
            .route("/api/launch", post(Self::launch))
            .route("/api/launch/{code}", get(Self::launch_by_code))
            .route("/api/runtime/active", get(Self::runtime_active))
            .route("/api/runtime/status", post(Self::runtime_status))
            .route("/api/runtime/stop", post(Self::runtime_stop))
            .route("/api/launch/logs", get(Self::logs))
            .with_state((app, api_state))
            .layer(TraceLayer::new_for_http())
    }

    async fn health() -> Json<LaunchApiHealth> {
        Json(LaunchApiHealth { ok: true })
    }

    async fn profiles(
        State((app, _)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
    ) -> Result<Json<Vec<BrowserProfile>>, (StatusCode, Json<Value>)> {
        app.state::<AppState>()
            .repositories
            .database
            .list_profiles()
            .map(Json)
            .map_err(map_err)
    }

    async fn profile(
        State((app, _)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Path(id): Path<String>,
    ) -> Result<Json<BrowserProfile>, (StatusCode, Json<Value>)> {
        app.state::<AppState>()
            .repositories
            .database
            .get_profile(&id)
            .map_err(map_err)?
            .map(Json)
            .ok_or_else(|| map_not_found("profile not found"))
    }

    async fn launch(
        State((app, api_state)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Json(req): Json<LaunchRequest>,
    ) -> Result<Json<LaunchApiLaunchResponse>, (StatusCode, Json<Value>)> {
        let state = app.state::<AppState>();
        let request_log = serde_json::to_value(&req).unwrap_or(Value::Null);
        let selector_log = selector_log_value(&req);
        let profile = resolve_profile(&state, req).map_err(map_err)?;
        let profile = runtime::start_profile(&state, profile.profile_id.clone())
            .map_err(|err| map_text(StatusCode::BAD_REQUEST, err))?;
        if let Ok(mut guard) = api_state.lock() {
            guard.active_profile_id = Some(profile.profile_id.clone());
            guard.active_debug_port = profile.debug_port;
        }
        append_log(
            &api_state,
            LaunchApiLogEntry {
                method: "POST".to_string(),
                path: "/api/launch".to_string(),
                client_ip: String::new(),
                launch_code: profile.launch_code.clone().unwrap_or_default(),
                selector: selector_log,
                request: request_log,
                success: true,
                status_code: 200,
                error: String::new(),
                profile_id: profile.profile_id.clone(),
                profile_name: profile.profile_name.clone(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
        Ok(Json(LaunchApiLaunchResponse {
            ok: true,
            launch_code: profile.launch_code.clone(),
            profile: Some(profile),
        }))
    }

    async fn launch_by_code(
        State((app, api_state)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Path(code): Path<String>,
    ) -> Result<Json<LaunchApiLaunchResponse>, (StatusCode, Json<Value>)> {
        let state = app.state::<AppState>();
        let profile_id = state
            .repositories
            .database
            .list_profiles()
            .map_err(map_err)?
            .into_iter()
            .find(|profile| profile.launch_code.as_deref() == Some(code.as_str()))
            .map(|profile| profile.profile_id)
            .ok_or_else(|| map_not_found("launch code not found"))?;
        let profile = runtime::start_profile(&state, profile_id)
            .map_err(|err| map_text(StatusCode::BAD_REQUEST, err))?;
        if let Ok(mut guard) = api_state.lock() {
            guard.active_profile_id = Some(profile.profile_id.clone());
            guard.active_debug_port = profile.debug_port;
        }
        append_log(
            &api_state,
            LaunchApiLogEntry {
                method: "GET".to_string(),
                path: format!("/api/launch/{code}"),
                client_ip: String::new(),
                launch_code: code,
                selector: json!({ "code": profile.launch_code }),
                request: Value::Null,
                success: true,
                status_code: 200,
                error: String::new(),
                profile_id: profile.profile_id.clone(),
                profile_name: profile.profile_name.clone(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
        Ok(Json(LaunchApiLaunchResponse {
            ok: true,
            launch_code: profile.launch_code.clone(),
            profile: Some(profile),
        }))
    }

    async fn runtime_active(
        State((app, api_state)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let state = app.state::<AppState>();
        let active_id = api_state
            .lock()
            .ok()
            .and_then(|guard| guard.active_profile_id.clone());
        let Some(profile_id) = active_id else {
            return Ok(Json(
                json!({ "ok": true, "active": false, "profile": null }),
            ));
        };
        let profile = runtime::status_profile(&state, &profile_id).map_err(map_err)?;
        Ok(Json(
            json!({ "ok": true, "active": true, "profile": profile }),
        ))
    }

    async fn runtime_status(
        State((app, _)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Json(req): Json<LaunchRequest>,
    ) -> Result<Json<LaunchApiRuntimePayload>, (StatusCode, Json<Value>)> {
        let state = app.state::<AppState>();
        let profile = resolve_profile(&state, req).map_err(map_err)?;
        Ok(Json(LaunchApiRuntimePayload {
            ok: true,
            profile: Some(profile),
        }))
    }

    async fn runtime_stop(
        State((app, api_state)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Json(req): Json<LaunchRequest>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let state = app.state::<AppState>();
        let profile = resolve_profile(&state, req).map_err(map_err)?;
        let profile = runtime::stop_profile(&state, profile.profile_id.clone())
            .map_err(|err| map_text(StatusCode::BAD_REQUEST, err))?;
        if let Ok(mut guard) = api_state.lock() {
            guard.active_profile_id = None;
            guard.active_debug_port = 0;
        }
        append_log(
            &api_state,
            LaunchApiLogEntry {
                method: "POST".to_string(),
                path: "/api/runtime/stop".to_string(),
                client_ip: String::new(),
                launch_code: profile.launch_code.clone().unwrap_or_default(),
                selector: json!({ "profileId": profile.profile_id }),
                request: Value::Null,
                success: true,
                status_code: 200,
                error: String::new(),
                profile_id: profile.profile_id.clone(),
                profile_name: profile.profile_name.clone(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
        Ok(Json(
            json!({ "ok": true, "profile": profile, "stopped": true }),
        ))
    }

    async fn logs(
        State((_app, api_state)): State<(AppHandle, Arc<Mutex<LaunchApiState>>)>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let limit = params
            .get("limit")
            .and_then(|raw| raw.parse::<usize>().ok())
            .unwrap_or(50)
            .clamp(1, 200);
        let logs = api_state
            .lock()
            .map_err(|_| {
                map_text(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "launch api lock poisoned",
                )
            })?
            .logs
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        Ok(Json(json!({ "ok": true, "items": logs })))
    }
}

fn selector_log_value(req: &LaunchRequest) -> Value {
    json!({
        "profileId": req.profile_id.as_deref(),
        "code": req.code.as_deref(),
    })
}

fn append_log(api_state: &Arc<Mutex<LaunchApiState>>, entry: LaunchApiLogEntry) {
    if let Ok(mut guard) = api_state.lock() {
        guard.logs.push(entry);
        if guard.logs.len() > 1000 {
            let excess = guard.logs.len() - 1000;
            guard.logs.drain(0..excess);
        }
    }
}

fn resolve_profile(state: &AppState, req: LaunchRequest) -> Result<BrowserProfile, AppError> {
    let id = req
        .profile_id
        .or(req.code)
        .ok_or_else(|| AppError::validation("selector is required"))?;
    state
        .repositories
        .database
        .get_profile(&id)?
        .ok_or_else(|| AppError::not_found("profile not found"))
}

fn map_text(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({ "ok": false, "error": message.into() })),
    )
}

fn map_not_found(message: impl Into<String>) -> (StatusCode, Json<Value>) {
    map_text(StatusCode::NOT_FOUND, message)
}

fn map_err(err: AppError) -> (StatusCode, Json<Value>) {
    match err {
        AppError::NotFound(msg) => map_not_found(msg),
        AppError::Validation(msg) => map_text(StatusCode::BAD_REQUEST, msg),
        _ => map_text(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
