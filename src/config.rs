use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider_endpoint: String,
    pub model: String,
    pub api_key: String,
    pub cycle_time_secs: u64,
    pub posture_threshold: u32,
    pub alert_threshold: u32,
    pub desk_raise_enabled: bool,
    pub desk_raise_interval_mins: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-5.4-mini".to_string(),
            api_key: String::new(),
            cycle_time_secs: 10,
            posture_threshold: 5,
            alert_threshold: 2,
            desk_raise_enabled: true,
            desk_raise_interval_mins: 60,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            // Check if config file exists
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
            // Create config directory if needed
            if let Some(dir) = path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            // Write default config
            let default = Config::default();
            let _ = fs::write(path, toml::to_string(&default).unwrap_or_default());
            return default;
        }
        Config::default()
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
