use crate::config::Config;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn init() {
    if let Some(config_path) = Config::config_path() {
        if let Some(dir) = config_path.parent() {
            let log_path = dir.join("posturewatch.log");
            if let Ok(file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                if let Ok(mut guard) = LOG_FILE.lock() {
                    *guard = Some(file);
                }
            }
        }
    }
}

pub fn log(level: &str, message: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let line = format!("[{}] {}: {}\n", timestamp, level, message);

    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::log("ERROR", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::log("INFO", &format!($($arg)*))
    };
}
