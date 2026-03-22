use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider_endpoint: String,
    pub model: String,
    pub api_key: String,
    pub cycle_time_secs: u64,
    pub alert_color: String,
    pub alert_duration_secs: u64,
    pub desk_raise_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: "".to_string(), // No hardcoded secrets!
            cycle_time_secs: 10,    // 10 seconds
            alert_color: "red".to_string(),
            alert_duration_secs: 5,
            desk_raise_interval_secs: 3600, // 1 hour
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
                        // Auto-migrate deprecated models
                        if config.model == "gpt-4-vision-preview" {
                            config.model = "gpt-4o-mini".to_string();
                            let _ = config.save();
                        }
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

    fn config_path() -> Option<PathBuf> {
        let possible_paths = vec![
            // Standard: %APPDATA%/com/posturewatch/PostureWatch/config.toml
            ProjectDirs::from("com", "posturewatch", "PostureWatch"),
            // With dot: %APPDATA%/com.posturewatch/PostureWatch/config.toml
            ProjectDirs::from("com.posturewatch", "PostureWatch", "PostureWatch"),
            // User created: %APPDATA%/com.posturewatch/PostureWatch/
            ProjectDirs::from("com", "posturewatch", "PostureWatch"),
        ];
        
        for dirs in possible_paths.into_iter().flatten() {
            let path = dirs.config_dir().join("config.toml");
            // Return the user's path if it exists, otherwise use it as the default
            return Some(path);
        }
        
        None
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

    pub fn prompt_for_api_key(&mut self) {
        if self.api_key.is_empty() {
            println!("\n=================================================");
            println!("  Welcome to PostureWatch!");
            println!("=================================================");
            println!("To analyze your posture, PostureWatch needs an API key.");
            println!("You can get a free API key from OpenAI or other providers.");
            print!("Please enter your API key: ");
            
            io::stdout().flush().unwrap();
            let mut api_key = String::new();
            if io::stdin().read_line(&mut api_key).is_ok() {
                self.api_key = api_key.trim().to_string();
                if !self.api_key.is_empty() {
                    println!("\nAPI key saved successfully!");
                    if let Err(e) = self.save() {
                        eprintln!("Warning: Could not save config: {}", e);
                    }
                }
            }
        }
    }
}
