use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider_endpoint: String,
    pub model: String,
    pub api_key: String,
    #[serde(default = "default_llm_prompt")]
    pub llm_prompt: String,
    pub cycle_time_secs: u64,
    #[serde(default)]
    pub camera_index: Option<u32>,
    #[serde(default = "default_keep_camera_on")]
    pub keep_camera_on: bool,
    pub posture_threshold: u32,
    pub alert_threshold: u32,
    pub desk_raise_enabled: bool,
    pub desk_raise_interval_mins: u64,
    #[serde(default = "default_break_reminder_enabled")]
    pub break_reminder_enabled: bool,
    #[serde(default = "default_break_reminder_after_mins")]
    #[serde(alias = "break_reminder_after_mins")]
    pub max_session_screen_time_mins: u64,
    #[serde(default = "default_max_daily_screen_time_mins")]
    pub max_daily_screen_time_mins: u64,
    #[serde(default = "default_break_reminder_repeat_secs")]
    pub break_reminder_repeat_secs: u64,
    #[serde(default = "default_break_reset_after_mins")]
    pub break_reset_after_mins: u64,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_frame_notification_enabled")]
    pub frame_notification_enabled: bool,
    #[serde(default = "default_frame_notification_duration_ms")]
    pub frame_notification_duration_ms: u64,
    #[serde(default = "default_frame_notification_area_percent")]
    pub frame_notification_area_percent: f32,
    #[serde(default = "default_frame_bad_posture_color")]
    pub frame_notification_bad_posture_color: String,
    #[serde(default = "default_frame_desk_raise_color")]
    pub frame_notification_desk_raise_color: String,
    #[serde(default = "default_frame_session_limit_color")]
    pub frame_notification_session_limit_color: String,
    #[serde(default = "default_frame_daily_limit_color")]
    pub frame_notification_daily_limit_color: String,
    #[serde(default = "default_frame_api_setup_color")]
    pub frame_notification_api_setup_color: String,
}

fn default_llm_prompt() -> String {
    "Rate the primary person's working posture from 1 to 10.\n\nUse the best possible estimate from visible posture cues, even if the full upper body is not visible.\n\n1 = terrible posture (severe slouching, head far forward)\n10 = excellent posture (upright back, shoulders aligned, head balanced)\n\nReply 'N' only if no person is visible, or posture truly cannot be estimated from the image.\n\nDo not return 'N' just because the person is standing, looking aside, partially visible, or briefly using a phone, unless those make posture impossible to judge.\n\nReply with ONLY a single number (1-10) or 'N'.".to_string()
}

fn default_break_reminder_enabled() -> bool {
    true
}

fn default_break_reminder_after_mins() -> u64 {
    60
}

fn default_max_daily_screen_time_mins() -> u64 {
    480
}

fn default_break_reminder_repeat_secs() -> u64 {
    30
}

fn default_break_reset_after_mins() -> u64 {
    5
}

fn default_keep_camera_on() -> bool {
    true
}

fn default_language() -> String {
    "en".to_string()
}

fn default_frame_notification_enabled() -> bool {
    false
}

fn default_frame_notification_duration_ms() -> u64 {
    1_000
}

fn default_frame_notification_area_percent() -> f32 {
    2.0
}

fn default_frame_bad_posture_color() -> String {
    "#FF0000".to_string()
}

fn default_frame_desk_raise_color() -> String {
    "#FFA500".to_string()
}

fn default_frame_session_limit_color() -> String {
    "#FFCC00".to_string()
}

fn default_frame_daily_limit_color() -> String {
    "#A020F0".to_string()
}

fn default_frame_api_setup_color() -> String {
    "#FF8000".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-5.4-mini".to_string(),
            api_key: String::new(),
            llm_prompt: default_llm_prompt(),
            cycle_time_secs: 10,
            camera_index: None,
            keep_camera_on: default_keep_camera_on(),
            posture_threshold: 5,
            alert_threshold: 2,
            desk_raise_enabled: true,
            desk_raise_interval_mins: 60,
            break_reminder_enabled: default_break_reminder_enabled(),
            max_session_screen_time_mins: default_break_reminder_after_mins(),
            max_daily_screen_time_mins: default_max_daily_screen_time_mins(),
            break_reminder_repeat_secs: default_break_reminder_repeat_secs(),
            break_reset_after_mins: default_break_reset_after_mins(),
            language: default_language(),
            frame_notification_enabled: default_frame_notification_enabled(),
            frame_notification_duration_ms: default_frame_notification_duration_ms(),
            frame_notification_area_percent: default_frame_notification_area_percent(),
            frame_notification_bad_posture_color: default_frame_bad_posture_color(),
            frame_notification_desk_raise_color: default_frame_desk_raise_color(),
            frame_notification_session_limit_color: default_frame_session_limit_color(),
            frame_notification_daily_limit_color: default_frame_daily_limit_color(),
            frame_notification_api_setup_color: default_frame_api_setup_color(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Config::default();
        };

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }

        let default = Config::default();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, toml::to_string(&default).unwrap_or_default());
        default
    }

    pub fn config_path() -> Option<PathBuf> {
        // Try user's existing path first: C:\Users\...\AppData\Roaming\com.posturewatch\PostureWatch\config.toml
        let user_path = std::env::var("APPDATA").ok().map(|appdata| {
            std::path::PathBuf::from(appdata)
                .join("com.posturewatch")
                .join("PostureWatch")
                .join("config.toml")
        });

        if let Some(ref path) = user_path {
            if path.exists() {
                // Silent: app runs from tray
                return user_path;
            }
        }

        // Standard path: C:\Users\...\AppData\Roaming\com\posturewatch\PostureWatch\config.toml
        let standard = ProjectDirs::from("com", "posturewatch", "PostureWatch").map(|dirs| {
            let path = dirs.config_dir().join("config.toml");
            // Silent: app runs from tray
            path
        });

        standard
    }

    #[cfg(windows)]
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(path, toml::to_string(self)?)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn appdata_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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
        assert!(config.keep_camera_on);
        assert!(config.desk_raise_enabled);
        assert!(config.break_reminder_enabled);
        assert_eq!(config.max_session_screen_time_mins, 60);
        assert_eq!(config.max_daily_screen_time_mins, 480);
        assert_eq!(config.break_reminder_repeat_secs, 30);
        assert_eq!(config.break_reset_after_mins, 5);
        assert!(!config.frame_notification_enabled);
        assert_eq!(config.frame_notification_duration_ms, 1_000);
        assert_eq!(config.frame_notification_area_percent, 2.0);
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
        assert_eq!(parsed.camera_index, default_config.camera_index);
        assert_eq!(parsed.keep_camera_on, default_config.keep_camera_on);
        assert_eq!(parsed.posture_threshold, default_config.posture_threshold);
        assert_eq!(parsed.alert_threshold, default_config.alert_threshold);
        assert_eq!(parsed.desk_raise_enabled, default_config.desk_raise_enabled);
        assert_eq!(
            parsed.desk_raise_interval_mins,
            default_config.desk_raise_interval_mins
        );
        assert_eq!(
            parsed.break_reminder_enabled,
            default_config.break_reminder_enabled
        );
        assert_eq!(
            parsed.max_session_screen_time_mins,
            default_config.max_session_screen_time_mins
        );
        assert_eq!(
            parsed.max_daily_screen_time_mins,
            default_config.max_daily_screen_time_mins
        );
        assert_eq!(
            parsed.break_reminder_repeat_secs,
            default_config.break_reminder_repeat_secs
        );
        assert_eq!(
            parsed.break_reset_after_mins,
            default_config.break_reset_after_mins
        );
        assert_eq!(parsed.language, default_config.language);
        assert_eq!(
            parsed.frame_notification_enabled,
            default_config.frame_notification_enabled
        );
        assert_eq!(
            parsed.frame_notification_duration_ms,
            default_config.frame_notification_duration_ms
        );
        assert_eq!(
            parsed.frame_notification_area_percent,
            default_config.frame_notification_area_percent
        );
        assert_eq!(
            parsed.frame_notification_bad_posture_color,
            default_config.frame_notification_bad_posture_color
        );
        assert_eq!(
            parsed.frame_notification_desk_raise_color,
            default_config.frame_notification_desk_raise_color
        );
        assert_eq!(
            parsed.frame_notification_session_limit_color,
            default_config.frame_notification_session_limit_color
        );
        assert_eq!(
            parsed.frame_notification_daily_limit_color,
            default_config.frame_notification_daily_limit_color
        );
        assert_eq!(
            parsed.frame_notification_api_setup_color,
            default_config.frame_notification_api_setup_color
        );
    }

    #[test]
    fn config_path_prefers_existing_appdata_location() {
        let _guard = appdata_env_lock()
            .lock()
            .expect("appdata env lock should not be poisoned");
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
        let _guard = appdata_env_lock()
            .lock()
            .expect("appdata env lock should not be poisoned");
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
        let _guard = appdata_env_lock()
            .lock()
            .expect("appdata env lock should not be poisoned");
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
camera_index = 1
keep_camera_on = false
posture_threshold = 6
alert_threshold = 3
desk_raise_enabled = false
desk_raise_interval_mins = 90
break_reminder_enabled = false
break_reminder_after_mins = 45
max_daily_screen_time_mins = 420
break_reminder_repeat_secs = 15
break_reset_after_mins = 7
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
        assert_eq!(loaded.camera_index, Some(1));
        assert!(!loaded.keep_camera_on);
        assert_eq!(loaded.posture_threshold, 6);
        assert_eq!(loaded.alert_threshold, 3);
        assert!(!loaded.desk_raise_enabled);
        assert_eq!(loaded.desk_raise_interval_mins, 90);
        assert!(!loaded.break_reminder_enabled);
        assert_eq!(loaded.max_session_screen_time_mins, 45);
        assert_eq!(loaded.max_daily_screen_time_mins, 420);
        assert_eq!(loaded.break_reminder_repeat_secs, 15);
        assert_eq!(loaded.break_reset_after_mins, 7);
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
        assert!(parsed.camera_index.is_none());
        assert!(parsed.keep_camera_on);
        assert!(parsed.break_reminder_enabled);
        assert_eq!(parsed.max_session_screen_time_mins, 60);
        assert_eq!(parsed.max_daily_screen_time_mins, 480);
        assert_eq!(parsed.break_reminder_repeat_secs, 30);
        assert_eq!(parsed.break_reset_after_mins, 5);
    }
}
