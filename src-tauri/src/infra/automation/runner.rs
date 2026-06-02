use crate::{
    app::state::AppState,
    domain::browser::BrowserProfile,
    error::AppError,
    infra::browser::runtime as browser_runtime,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

use super::runtime;
use super::script;
use super::settings;
use super::types::*;
use super::util::*;

const RESULT_PREFIX: &str = "__YUBAI_AUTOMATION_RESULT__";

pub fn list_runs(state: &AppState, limit: i32) -> Result<Vec<AutomationRun>, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let take = limit.clamp(1, 200) as usize;
    let mut runs = Vec::new();
    for entry in fs::read_dir(&paths.runs)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
            let run = read_json::<AutomationRun>(&entry.path())?;
            runs.push(run);
        }
    }
    runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    runs.truncate(take);
    Ok(runs)
}

#[allow(dead_code)]
pub fn run_script(
    state: &AppState,
    input: AutomationRunInput,
) -> Result<AutomationRun, AppError> {
    enqueue_script_run_record(state, input)
}

pub fn enqueue_script_run(
    app: AppHandle,
    input: AutomationRunInput,
) -> Result<AutomationRun, AppError> {
    let run = {
        let state = app.state::<AppState>();
        enqueue_script_run_record(state.inner(), input)?
    };
    emit_run_event(&app, "automation:run:queued", &run, "queued");
    ensure_queue_worker(app)?;
    Ok(run)
}

pub fn cancel_run(app: AppHandle, run_id: String) -> Result<AutomationRun, AppError> {
    let state = app.state::<AppState>();
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;

    let mut child_to_kill = None;
    let mut cancelled_pending = false;
    {
        let mut queue = state
            .runtimes
            .automation_queue
            .lock()
            .map_err(|_| AppError::LockPoisoned("automation queue lock poisoned".to_string()))?;
        if let Some(index) = queue.pending.iter().position(|task| task.run_id == run_id) {
            queue.pending.remove(index);
            queue.cancelled.insert(run_id.clone());
            cancelled_pending = true;
        } else if let Some(child) = queue.running.get(&run_id).cloned() {
            queue.cancelled.insert(run_id.clone());
            child_to_kill = Some(child);
        }
    }

    if let Some(child) = child_to_kill {
        if let Ok(mut child) = child.lock() {
            let _ = child.kill();
        }
    }

    let mut run = read_run(&paths, &run_id)?;
    run.status = "cancelled".to_string();
    run.finished_at = Utc::now().to_rfc3339();
    run.error = if cancelled_pending {
        "cancelled before start".to_string()
    } else {
        "cancelled by user".to_string()
    };
    write_run(&paths, &run)?;
    emit_run_event(&app, "automation:run:cancelled", &run, "cancelled");
    Ok(run)
}

fn enqueue_script_run_record(
    state: &AppState,
    input: AutomationRunInput,
) -> Result<AutomationRun, AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let settings = settings::read_settings(&paths)?;
    if !settings.enabled {
        return Err(AppError::validation("automation runtime is disabled"));
    }
    let check = settings::self_check(state)?;
    if !check.ok {
        return Err(AppError::validation(if check.error.is_empty() {
            "automation runtime is not ready".to_string()
        } else {
            check.error
        }));
    }
    let config = state.config_snapshot().map_err(AppError::from)?;
    if runtime::bundled_node_paths(&paths, &config)?.is_none() {
        return Err(AppError::validation("bundled Node.js is not installed"));
    }

    let script = script::get_script_by_id(&paths, &input.script_id)?;
    let profile = state
        .repositories
        .database
        .get_profile(&input.profile_id)?
        .ok_or_else(|| AppError::not_found("browser profile not found"))?;

    let run_id = format!("run-{}", uuid::Uuid::new_v4());
    let started = Utc::now().to_rfc3339();
    let run = pending_run(&run_id, &script, &profile, &started);
    write_run(&paths, &run)?;

    {
        let mut queue = state
            .runtimes
            .automation_queue
            .lock()
            .map_err(|_| AppError::LockPoisoned("automation queue lock poisoned".to_string()))?;
        queue.pending.push_back(AutomationQueuedTask {
            run_id: run_id.clone(),
            input,
        });
    }
    Ok(run)
}

fn ensure_queue_worker(app: AppHandle) -> Result<(), AppError> {
    let should_start = {
        let state = app.state::<AppState>();
        let mut queue = state
            .runtimes
            .automation_queue
            .lock()
            .map_err(|_| AppError::LockPoisoned("automation queue lock poisoned".to_string()))?;
        if queue.worker_running {
            false
        } else {
            queue.worker_running = true;
            true
        }
    };

    if !should_start {
        return Ok(());
    }

    std::thread::spawn(move || run_queue_worker(app));
    Ok(())
}

fn run_queue_worker(app: AppHandle) {
    loop {
        let task = {
            let state = app.state::<AppState>();
            let mut queue = match state.runtimes.automation_queue.lock() {
                Ok(queue) => queue,
                Err(_) => return,
            };
            match queue.pending.pop_front() {
                Some(task) => task,
                None => {
                    queue.worker_running = false;
                    if !queue.pending.is_empty() {
                        queue.worker_running = true;
                        continue;
                    }
                    return;
                }
            }
        };

        let state = app.state::<AppState>();
        if let Err(err) = execute_queued_task(&app, state.inner(), task.clone()) {
            let paths = AutomationPaths::new(&state.app_root);
            if paths.ensure().is_ok() {
                let fallback = failed_run_from_task(&paths, &task, err.to_string());
                let _ = write_run(&paths, &fallback);
                emit_run_event(&app, "automation:run:failed", &fallback, "failed");
            }
        }
    }
}

fn execute_queued_task(
    app: &AppHandle,
    state: &AppState,
    task: AutomationQueuedTask,
) -> Result<(), AppError> {
    let paths = AutomationPaths::new(&state.app_root);
    paths.ensure()?;
    let input = task.input;
    let settings = settings::read_settings(&paths)?;
    let config = state.config_snapshot().map_err(AppError::from)?;
    let node_paths = runtime::bundled_node_paths(&paths, &config)?
        .ok_or_else(|| AppError::validation("bundled Node.js is not installed"))?;

    let script = script::get_script_by_id(&paths, &input.script_id)?;
    let profile = browser_runtime::start_profile(state, input.profile_id.clone())?;
    if profile.debug_port <= 0 {
        return Err(AppError::validation("browser debug port is not available"));
    }
    let debug_port = browser_runtime::wait_debug_ready(
        state,
        &profile.profile_id,
        config.browser.start_ready_timeout_ms,
    )?;

    let started_instant = Instant::now();
    runtime::write_runner(&paths.runner)?;
    let script_path = script::canonical_script_entry(&paths, &script)?;
    let mut run = read_run(&paths, &task.run_id).unwrap_or_else(|_| {
        pending_run(
            &task.run_id,
            &script,
            &profile,
            &Utc::now().to_rfc3339(),
        )
    });
    run.status = "running".to_string();
    run.profile_name = profile.profile_name.clone();
    write_run(&paths, &run)?;
    register_profile_run(state, profile.profile_id.clone(), task.run_id.clone())?;
    emit_run_event(app, "automation:run:started", &run, "started");

    let runner_input = json!({
        "runId": task.run_id,
        "scriptId": script.script_id,
        "profileId": profile.profile_id,
        "debugPort": debug_port,
        "headless": settings.headless_default,
        "timeoutMs": input.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
        "scriptPath": script_path,
        "params": input.params.unwrap_or_else(|| json!({})),
    });

    let output = run_node_runner(
        app,
        state,
        &task.run_id,
        &node_paths.executable_path,
        &paths,
        &runner_input,
        input.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
    );
    let finished = Utc::now().to_rfc3339();
    run.finished_at = finished;
    run.duration_ms = started_instant.elapsed().as_millis().min(i64::MAX as u128) as i64;

    match output {
        Ok(result) => {
            run.exit_code = result.exit_code;
            run.stdout = result.stdout;
            run.stderr = result.stderr;
            run.result_json = result.result_json;
            run.error = result.error;
            run.status = if run.error.is_empty() && run.exit_code == Some(0) {
                "success".to_string()
            } else {
                "failed".to_string()
            };
        }
        Err(err) => {
            if is_run_cancelled(state, &task.run_id) {
                run.status = "cancelled".to_string();
                run.error = "cancelled by user".to_string();
            } else {
                run.status = "failed".to_string();
                run.error = err.to_string();
            }
        }
    }

    write_run(&paths, &run)?;
    clear_running_task(state, &task.run_id);
    clear_profile_run(state, &profile.profile_id, &task.run_id);
    clear_cancelled_task(state, &task.run_id);
    match run.status.as_str() {
        "success" => emit_run_event(app, "automation:run:finished", &run, "finished"),
        "cancelled" => emit_run_event(app, "automation:run:cancelled", &run, "cancelled"),
        _ => emit_run_event(app, "automation:run:failed", &run, "failed"),
    }
    Ok(())
}

fn pending_run(
    run_id: &str,
    script: &AutomationScript,
    profile: &BrowserProfile,
    started_at: &str,
) -> AutomationRun {
    AutomationRun {
        run_id: run_id.to_string(),
        script_id: script.script_id.clone(),
        script_name: script.name.clone(),
        profile_id: profile.profile_id.clone(),
        profile_name: profile.profile_name.clone(),
        status: "pending".to_string(),
        started_at: started_at.to_string(),
        finished_at: String::new(),
        duration_ms: 0,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        result_json: String::new(),
        error: String::new(),
    }
}

fn failed_run_from_task(
    paths: &AutomationPaths,
    task: &AutomationQueuedTask,
    error: String,
) -> AutomationRun {
    let now = Utc::now().to_rfc3339();
    match read_run(paths, &task.run_id) {
        Ok(mut run) => {
            run.status = "failed".to_string();
            run.finished_at = now;
            run.duration_ms = 0;
            run.error = error;
            run
        }
        Err(_) => AutomationRun {
            run_id: task.run_id.clone(),
            script_id: task.input.script_id.clone(),
            script_name: String::new(),
            profile_id: task.input.profile_id.clone(),
            profile_name: String::new(),
            status: "failed".to_string(),
            started_at: now.clone(),
            finished_at: now,
            duration_ms: 0,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            result_json: String::new(),
            error,
        },
    }
}

fn run_node_runner(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    node: &Path,
    paths: &AutomationPaths,
    input: &Value,
    timeout_ms: u64,
) -> Result<RunnerResult, AppError> {
    let mut command = Command::new(node);
    command
        .arg(&paths.runner)
        .current_dir(&paths.runtime)
        .env("NODE_PATH", paths.runtime.join("node_modules"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.to_string().as_bytes())?;
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let stderr_lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut stdout_join = Some(spawn_stream_reader(app.clone(), run_id.to_string(), "stdout", stdout, Arc::clone(&stdout_lines)));
    let mut stderr_join = Some(spawn_stream_reader(app.clone(), run_id.to_string(), "stderr", stderr, Arc::clone(&stderr_lines)));
    let child = Arc::new(Mutex::new(child));
    register_running_task(state, run_id.to_string(), Arc::clone(&child))?;

    let timeout = Duration::from_millis(timeout_ms.max(1000));
    let start = Instant::now();
    loop {
        let maybe_status = {
            let mut guard = child
                .lock()
                .map_err(|_| AppError::LockPoisoned("automation child lock poisoned".to_string()))?;
            guard.try_wait()?
        };
        if let Some(status) = maybe_status {
            join_reader(&mut stdout_join);
            join_reader(&mut stderr_join);
            let stdout = join_lines(&stdout_lines);
            let stderr = join_lines(&stderr_lines);
            let parsed = parse_runner_output(&stdout);
            let error = runner_error(status.success(), parsed.as_ref(), &stderr);
            return Ok(RunnerResult {
                exit_code: status.code(),
                stdout,
                stderr,
                result_json: parsed
                    .as_ref()
                    .map(Value::to_string)
                    .unwrap_or_else(String::new),
                error,
            });
        }
        if start.elapsed() > timeout {
            if let Ok(mut guard) = child.lock() {
                let _ = guard.kill();
                let _ = guard.wait();
            }
            join_reader(&mut stdout_join);
            join_reader(&mut stderr_join);
            return Err(AppError::validation(format!(
                "automation script timed out after {timeout_ms}ms"
            )));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn parse_runner_output(stdout: &str) -> Option<Value> {
    stdout
        .lines()
        .rev()
        .find_map(|line| line.trim_start().strip_prefix(RESULT_PREFIX))
        .and_then(|line| serde_json::from_str::<Value>(line).ok())
}

fn spawn_stream_reader(
    app: AppHandle,
    run_id: String,
    stream: &'static str,
    pipe: Option<impl Read + Send + 'static>,
    lines: Arc<Mutex<Vec<String>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let Some(pipe) = pipe else {
            return;
        };
        let reader = BufReader::new(pipe);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(mut guard) = lines.lock() {
                guard.push(line.clone());
            }
            emit_run_log_event(&app, &run_id, stream, &line);
        }
    })
}

fn join_reader(handle: &mut Option<std::thread::JoinHandle<()>>) {
    if let Some(handle) = handle.take() {
        let _ = handle.join();
    }
}

fn join_lines(lines: &Arc<Mutex<Vec<String>>>) -> String {
    lines
        .lock()
        .map(|guard| guard.join("\n"))
        .unwrap_or_default()
}

fn runner_error(success: bool, parsed: Option<&Value>, stderr: &str) -> String {
    if let Some(value) = parsed {
        if value.get("ok").and_then(Value::as_bool) == Some(false) {
            return value
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("automation script failed")
                .to_string();
        }
        if success {
            return String::new();
        }
    }
    if success {
        String::new()
    } else if stderr.trim().is_empty() {
        "automation runner failed".to_string()
    } else {
        stderr.trim().to_string()
    }
}

fn register_running_task(
    state: &AppState,
    run_id: String,
    child: Arc<Mutex<Child>>,
) -> Result<(), AppError> {
    let mut queue = state
        .runtimes
        .automation_queue
        .lock()
        .map_err(|_| AppError::LockPoisoned("automation queue lock poisoned".to_string()))?;
    queue.running.insert(run_id, child);
    Ok(())
}

fn register_profile_run(
    state: &AppState,
    profile_id: String,
    run_id: String,
) -> Result<(), AppError> {
    let mut queue = state
        .runtimes
        .automation_queue
        .lock()
        .map_err(|_| AppError::LockPoisoned("automation queue lock poisoned".to_string()))?;
    queue.profile_runs.insert(profile_id, run_id);
    Ok(())
}

fn clear_running_task(state: &AppState, run_id: &str) {
    if let Ok(mut queue) = state.runtimes.automation_queue.lock() {
        queue.running.remove(run_id);
    }
}

fn clear_profile_run(state: &AppState, profile_id: &str, run_id: &str) {
    if let Ok(mut queue) = state.runtimes.automation_queue.lock() {
        if queue.profile_runs.get(profile_id).is_some_and(|value| value == run_id) {
            queue.profile_runs.remove(profile_id);
        }
    }
}

fn is_run_cancelled(state: &AppState, run_id: &str) -> bool {
    state
        .runtimes
        .automation_queue
        .lock()
        .map(|queue| queue.cancelled.contains(run_id))
        .unwrap_or(false)
}

fn clear_cancelled_task(state: &AppState, run_id: &str) {
    if let Ok(mut queue) = state.runtimes.automation_queue.lock() {
        queue.cancelled.remove(run_id);
    }
}

fn emit_run_event(app: &AppHandle, event: &str, run: &AutomationRun, message: &str) {
    let _ = app.emit(
        event,
        AutomationRunEvent {
            run_id: run.run_id.clone(),
            status: run.status.clone(),
            message: message.to_string(),
            run: Some(run.clone()),
        },
    );
}

fn emit_run_log_event(app: &AppHandle, run_id: &str, stream: &str, line: &str) {
    let _ = app.emit(
        "automation:run:log",
        AutomationRunLogEvent {
            run_id: run_id.to_string(),
            stream: stream.to_string(),
            line: line.to_string(),
            created_at: Utc::now().to_rfc3339(),
        },
    );
}

fn write_run(paths: &AutomationPaths, run: &AutomationRun) -> Result<(), AppError> {
    write_json_pretty(&paths.runs.join(format!("{}.json", run.run_id)), run)
}

fn read_run(paths: &AutomationPaths, run_id: &str) -> Result<AutomationRun, AppError> {
    if run_id.contains('/') || run_id.contains('\\') || run_id.contains("..") {
        return Err(AppError::validation("invalid run id"));
    }
    read_json(&paths.runs.join(format!("{run_id}.json")))
}
