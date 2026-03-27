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
