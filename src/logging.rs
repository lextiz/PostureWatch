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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_and_log_write_line_to_log_file() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let appdata_root = std::env::temp_dir().join(format!("posturewatch-log-{nanos}"));
        let config_path = appdata_root
            .join("com.posturewatch")
            .join("PostureWatch")
            .join("config.toml");
        let config_dir = config_path
            .parent()
            .expect("config path should have parent");
        fs::create_dir_all(config_dir).expect("create test config directory");
        fs::write(&config_path, "model = 'gpt-5.4-mini'").expect("create existing config file");

        std::env::set_var("APPDATA", &appdata_root);
        init();
        log("INFO", "hello test");

        let log_path = config_dir.join("posturewatch.log");
        let contents = fs::read_to_string(log_path).expect("log file should be readable");
        assert!(contents.contains("INFO: hello test"));
    }
}
