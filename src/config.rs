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
    pub posture_threshold: u32,
    pub alert_threshold: u32,
    pub desk_raise_enabled: bool,
    pub desk_raise_interval_mins: u64,
}

fn default_llm_prompt() -> String {
    "Rate the person’s working posture from the image. \nReply with ONLY: \n- a single number 1-10, or \n- 'N' if posture cannot be judged reliably. \nReturn 'N' unless ALL are true: \n- exactly one person is clearly visible \n- the person is at a desk/workstation, either seated or standing \n- the upper body is visible enough to judge posture: head, neck, shoulders, and torso \n- the pose is neutral and representative of normal working posture \nReturn 'N' for any ambiguity, including: partial upper body, occlusion, blur, multiple people, walking, stretching, leaning far away from the desk, talking on the phone, turning strongly sideways, looking far aside, or any temporary/non-working pose. \nIf valid, score only posture alignment: \n1 = severe slouch / head far forward / poor upper-body alignment \n10 = upright neutral posture / shoulders aligned / head balanced".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-5.4-mini".to_string(),
            api_key: String::new(),
            llm_prompt: default_llm_prompt(),
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
#[path = "tests/config_tests.rs"]
mod tests;
