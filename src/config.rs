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
        assert_eq!(parsed.cycle_time_secs, default_config.cycle_time_secs);
        assert_eq!(parsed.posture_threshold, default_config.posture_threshold);
        assert_eq!(parsed.alert_threshold, default_config.alert_threshold);
        assert_eq!(parsed.desk_raise_enabled, default_config.desk_raise_enabled);
        assert_eq!(
            parsed.desk_raise_interval_mins,
            default_config.desk_raise_interval_mins
        );
    }
}
