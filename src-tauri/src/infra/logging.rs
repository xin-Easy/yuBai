use crate::{domain::app::AppLogEntry, error::AppError};
use chrono::Utc;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

static LOGGER: OnceLock<Arc<AppLogger>> = OnceLock::new();

pub struct AppLogger {
    logs: Mutex<Vec<AppLogEntry>>,
    file: Mutex<Option<File>>,
}

impl AppLogger {
    pub fn new(log_path: PathBuf) -> Result<Arc<Self>, AppError> {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();
        Ok(Arc::new(Self {
            logs: Mutex::new(Vec::new()),
            file: Mutex::new(file),
        }))
    }

    #[allow(dead_code)]
    pub fn push(&self, level: impl Into<String>, message: impl Into<String>) {
        self.push_with_target(level, "app", message, String::new());
    }

    pub fn push_with_target(
        &self,
        level: impl Into<String>,
        target: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) {
        let entry = AppLogEntry {
            level: level.into(),
            message: message.into(),
            target: target.into(),
            details: details.into(),
            created_at: Utc::now().to_rfc3339(),
        };
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(entry.clone());
            if logs.len() > 1000 {
                let excess = logs.len() - 1000;
                logs.drain(0..excess);
            }
        }
        if let Ok(mut file_guard) = self.file.lock() {
            if let Some(file) = file_guard.as_mut() {
                let _ = writeln!(
                    file,
                    "{} [{}] [{}] {}{}",
                    entry.created_at,
                    entry.level,
                    entry.target,
                    entry.message,
                    if entry.details.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" | {}", entry.details)
                    }
                );
            }
        }
    }

    pub fn list(&self) -> Vec<AppLogEntry> {
        self.logs
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }
}

pub fn init_global(log_path: PathBuf) -> Result<Arc<AppLogger>, AppError> {
    let logger = AppLogger::new(log_path)?;
    let _ = LOGGER.set(logger.clone());
    Ok(logger)
}

pub fn global() -> Option<Arc<AppLogger>> {
    LOGGER.get().cloned()
}

#[allow(dead_code)]
pub fn log(level: impl Into<String>, message: impl Into<String>) {
    if let Some(logger) = global() {
        logger.push(level, message);
    }
}

pub fn log_target(
    level: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
    details: impl Into<String>,
) {
    if let Some(logger) = global() {
        logger.push_with_target(level, target, message, details);
    }
}
