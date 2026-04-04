//! Logging system for slio-git
//!
//! Provides structured logging with file rotation support

#![allow(dead_code)]

use log::{error, info, warn, LevelFilter};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Log file manager with rotation
#[derive(Clone)]
pub struct LogManager {
    log_path: PathBuf,
    max_file_size: u64,
    max_files: usize,
}

static LOG_MANAGER: Mutex<Option<LogManager>> = Mutex::new(None);

impl LogManager {
    /// Initialize logging system
    pub fn init(log_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        let log_dir = log_dir.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("slio-git")
                .join("logs")
        });

        fs::create_dir_all(&log_dir)?;

        let log_path = log_dir.join("slio-git.log");

        let manager = LogManager {
            log_path,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        };

        // Initialize env_logger
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();

        // Store manager instance
        {
            let mut guard = LOG_MANAGER.lock().unwrap();
            *guard = Some(manager);
        }

        info!("slio-git logging initialized");
        info!("Log directory: {:?}", log_dir);

        Ok(())
    }

    /// Log a repository operation
    pub fn log_repo_operation(operation: &str, repo_path: &str, success: bool) {
        if success {
            info!("[REPO] {}: {} - OK", operation, repo_path);
        } else {
            error!("[REPO] {}: {} - FAILED", operation, repo_path);
        }
    }

    /// Log a git operation
    pub fn log_git_operation(operation: &str, details: &str) {
        info!("[GIT] {} - {}", operation, details);
    }

    /// Log UI event
    pub fn log_ui_event(event: &str) {
        info!("[UI] {}", event);
    }

    /// Log shell navigation between major sections.
    pub fn log_navigation(section: &str, detail: &str) {
        info!("[NAV] {} - {}", section, detail);
    }

    /// Log a navigation transition between major shell sections.
    pub fn log_navigation_transition(from: &str, to: &str, detail: &str) {
        info!("[NAV] {} -> {} ({})", from, to, detail);
    }

    /// Log an async or background operation failure that also surfaces in UI feedback.
    pub fn log_async_failure(operation: &str, source: &str, detail: &str) {
        error!("[ASYNC] {} - {} - {}", operation, source, detail);
    }

    /// Log a blocked action where the UI deliberately keeps the user in place.
    pub fn log_action_blocked(operation: &str, source: &str, reason: &str) {
        warn!("[BLOCKED] {} - {} - {}", operation, source, reason);
    }

    /// Log a feedback banner that becomes visible to the user.
    pub fn log_feedback(level: &str, title: &str, detail: Option<&str>) {
        match detail {
            Some(detail) => info!("[FEEDBACK] {} - {} ({})", level, title, detail),
            None => info!("[FEEDBACK] {} - {}", level, title),
        }
    }

    /// Log compact feedback surfaced in the status bar or minimal chrome.
    pub fn log_compact_feedback(level: &str, title: &str) {
        info!("[FEEDBACK_COMPACT] {} - {}", level, title);
    }

    /// Log context switcher lifecycle and quick actions.
    pub fn log_context_switcher(event: &str, detail: &str) {
        info!("[CTX] {} - {}", event, detail);
    }

    /// Log a candidate defect discovered during the redesign pass.
    pub fn log_defect(area: &str, summary: &str) {
        warn!("[DEFECT] {} - {}", area, summary);
    }

    /// Rotate log file if needed
    pub fn maybe_rotate(&self) -> std::io::Result<()> {
        if let Ok(metadata) = fs::metadata(&self.log_path) {
            if metadata.len() > self.max_file_size {
                self.rotate()?;
            }
        }
        Ok(())
    }

    /// Rotate log files
    fn rotate(&self) -> std::io::Result<()> {
        // Remove oldest log file if we have too many
        let oldest = format!("{}.{}", self.log_path.display(), self.max_files);
        let _ = fs::remove_file(&oldest);

        // Shift log files
        for i in (1..self.max_files).rev() {
            let src = format!("{}.{}", self.log_path.display(), i);
            let dst = format!("{}.{}", self.log_path.display(), i + 1);
            let _ = fs::rename(&src, &dst);
        }

        // Rename current log
        let archive = format!("{}.1", self.log_path.display());
        let _ = fs::rename(&self.log_path, &archive);

        info!("Log file rotated");
        Ok(())
    }
}

/// Get log manager instance
pub fn get_log_manager() -> Option<LogManager> {
    let guard = LOG_MANAGER.lock().unwrap();
    guard.clone()
}
