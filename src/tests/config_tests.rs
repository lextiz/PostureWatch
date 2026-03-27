use super::Config;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("posturewatch-{name}-{nanos}"))
}

#[test]
fn default_thresholds_are_valid() {
    let config = Config::default();

    assert_eq!(config.posture_threshold, 5);
    assert_eq!(config.alert_threshold, 2);
    assert!(config.desk_raise_enabled);
}

#[test]
fn default_config_round_trips_through_toml() {
    let default_config = Config::default();

    let toml_text = toml::to_string(&default_config).expect("serialize default config");
    let parsed: Config = toml::from_str(&toml_text).expect("parse serialized config");

    assert_eq!(parsed.provider_endpoint, default_config.provider_endpoint);
    assert_eq!(parsed.model, default_config.model);
    assert_eq!(parsed.api_key, default_config.api_key);
    assert_eq!(parsed.llm_prompt, default_config.llm_prompt);
    assert_eq!(parsed.cycle_time_secs, default_config.cycle_time_secs);
    assert_eq!(parsed.posture_threshold, default_config.posture_threshold);
    assert_eq!(parsed.alert_threshold, default_config.alert_threshold);
    assert_eq!(parsed.desk_raise_enabled, default_config.desk_raise_enabled);
    assert_eq!(
        parsed.desk_raise_interval_mins,
        default_config.desk_raise_interval_mins
    );
}

#[test]
fn config_path_prefers_existing_appdata_location() {
    let appdata_root = unique_temp_dir("appdata-pref");
    let user_config = appdata_root
        .join("com.posturewatch")
        .join("PostureWatch")
        .join("config.toml");
    fs::create_dir_all(
        user_config
            .parent()
            .expect("user config file should have parent dir"),
    )
    .expect("create appdata directory");
    fs::write(&user_config, "model = 'test'").expect("write user config");

    std::env::set_var("APPDATA", &appdata_root);
    let resolved = Config::config_path().expect("config path should resolve");
    assert_eq!(resolved, user_config);
}

#[test]
fn load_returns_default_when_toml_invalid() {
    let appdata_root = unique_temp_dir("appdata-invalid");
    let user_config = appdata_root
        .join("com.posturewatch")
        .join("PostureWatch")
        .join("config.toml");
    fs::create_dir_all(
        user_config
            .parent()
            .expect("user config file should have parent dir"),
    )
    .expect("create appdata directory");
    fs::write(&user_config, "not valid toml = [").expect("write invalid config");

    std::env::set_var("APPDATA", &appdata_root);
    let loaded = Config::load();
    assert_eq!(loaded.model, Config::default().model);

    let rewritten = fs::read_to_string(&user_config).expect("read rewritten config");
    let reparsed: Config = toml::from_str(&rewritten).expect("rewritten config should parse");
    assert_eq!(reparsed.model, Config::default().model);
}

#[test]
fn load_reads_existing_valid_config() {
    let appdata_root = unique_temp_dir("appdata-valid");
    let user_config = appdata_root
        .join("com.posturewatch")
        .join("PostureWatch")
        .join("config.toml");
    fs::create_dir_all(
        user_config
            .parent()
            .expect("user config file should have parent dir"),
    )
    .expect("create appdata directory");
    fs::write(
        &user_config,
        r#"
provider_endpoint = "http://localhost:1234/v1/chat/completions"
model = "test-model"
api_key = "abc"
llm_prompt = "custom prompt"
cycle_time_secs = 22
posture_threshold = 6
alert_threshold = 3
desk_raise_enabled = false
desk_raise_interval_mins = 90
"#,
    )
    .expect("write valid config");

    std::env::set_var("APPDATA", &appdata_root);
    let loaded = Config::load();
    assert_eq!(loaded.model, "test-model");
    assert_eq!(
        loaded.provider_endpoint,
        "http://localhost:1234/v1/chat/completions"
    );
    assert_eq!(loaded.api_key, "abc");
    assert_eq!(loaded.llm_prompt, "custom prompt");
    assert_eq!(loaded.cycle_time_secs, 22);
    assert_eq!(loaded.posture_threshold, 6);
    assert_eq!(loaded.alert_threshold, 3);
    assert!(!loaded.desk_raise_enabled);
    assert_eq!(loaded.desk_raise_interval_mins, 90);
}

#[test]
fn deserialize_uses_default_prompt_when_field_missing() {
    let parsed: Config = toml::from_str(
        r#"
provider_endpoint = "https://api.openai.com/v1/chat/completions"
model = "gpt-5.4-mini"
api_key = ""
cycle_time_secs = 10
posture_threshold = 5
alert_threshold = 2
desk_raise_enabled = true
desk_raise_interval_mins = 60
"#,
    )
    .expect("parse config missing llm_prompt");

    assert_eq!(parsed.llm_prompt, super::default_llm_prompt());
}
