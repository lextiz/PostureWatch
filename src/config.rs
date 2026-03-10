use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider_endpoint: String,
    pub model: String,
    pub api_key: String,
    pub privacy_mode: bool,
    pub cycle_time_secs: u64,
    pub alert_color: String,
    pub alert_duration_secs: u64,
    pub desk_raise_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4-vision-preview".to_string(),
            api_key: "".to_string(), // No hardcoded secrets!
            privacy_mode: true, // true means send only minimal data or use local fallback
            cycle_time_secs: 300, // 5 minutes
            alert_color: "red".to_string(),
            alert_duration_secs: 5,
            desk_raise_interval_secs: 3600, // 1 hour
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
            // Create default if not exists
            let default = Config::default();
            if let Some(dir) = path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = fs::write(path, toml::to_string(&default).unwrap_or_default());
            return default;
        }
        Config::default()
    }

    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "posturewatch", "PostureWatch").map(|dirs| {
            dirs.config_dir().join("config.toml")
        })
    }
}
