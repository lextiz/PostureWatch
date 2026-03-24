// System tray and GUI module for PostureWatch using system tray menu

use crate::config::Config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// Global flag to signal app shutdown
pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
const TRAY_ICON_SIZE: u32 = 32;

#[allow(dead_code)]
pub struct TrayManager {
    // Tray state
}

impl TrayManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(windows)]
    pub fn setup_tray(config: Arc<TokioMutex<Config>>) {
        use std::thread;

        // Spawn tray in a separate thread since it requires blocking API
        let config_clone = config.clone();
        thread::spawn(move || {
            if let Err(e) = Self::run_tray(config_clone) {
                eprintln!("Failed to setup tray: {}", e);
            }
        });
    }

    #[cfg(windows)]
    fn run_tray(config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::{MenuBuilder, TrayIconBuilder};

        // Load configuration
        let _current_config = Config::load();

        // Create system tray menu
        let menu = MenuBuilder::new(None)
            .text("open", "Open Settings")
            .text("strictness", "Strictness Level")
            .text("strictness_low", "  Low")
            .text("strictness_medium", "  Medium")
            .text("strictness_high", "  High")
            .separator()
            .text("quit", "Quit PostureWatch")
            .build()?;

        // Create tray icon
        let icon = Self::create_tray_icon()?;

        let _tray = TrayIconBuilder::new()
            .icon(icon)
            .menu(&menu)
            .tooltip("PostureWatch - Posture Monitoring")
            .on_menu_event(move |_tray, event| {
                match event.id.as_ref() {
                    "open" => {
                        // Open config file in default editor
                        if let Some(config_path) = crate::config::Config::config_path() {
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("notepad")
                                    .arg(&config_path)
                                    .spawn();
                            }
                        }
                    }
                    "strictness_low" => {
                        Self::update_strictness("Low".to_string());
                    }
                    "strictness_medium" => {
                        Self::update_strictness("Medium".to_string());
                    }
                    "strictness_high" => {
                        Self::update_strictness("High".to_string());
                    }
                    "quit" => {
                        APP_RUNNING.store(false, Ordering::SeqCst);
                        std::process::exit(0);
                    }
                    _ => {}
                }
            })
            .build(None)?;

        // Run a simple event loop for the tray
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    #[cfg(windows)]
    fn update_strictness(strictness: String) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut cfg = Config::load();
            cfg.strictness = strictness;
            let _ = cfg.save();
        });
    }

    #[cfg(windows)]
    fn create_tray_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
        let size = TRAY_ICON_SIZE;
        let mut img = image::RgbaImage::new(size, size);
        let color = image::Rgba([0, 123, 255, 255]); // #007bff

        for pixel in img.pixels_mut() {
            *pixel = color;
        }

        let icon = tray_icon::Icon::from_rgba(img.into_raw(), size, size)?;

        Ok(icon)
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // On non-Windows, just run silently without tray for now
    }

    #[cfg(not(windows))]
    #[allow(dead_code)]
    fn run_tray(_config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        // Keep thread alive
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
}
