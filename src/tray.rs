// System tray module for PostureWatch
// Simplified approach for Windows tray icon

#![allow(dead_code)]

use crate::config::Config;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex as TokioMutex;

pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);

pub struct TrayManager;

impl TrayManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn setup_tray(config: Arc<TokioMutex<Config>>) {
        // Spawn tray in a separate thread with its own message loop
        let config_clone = config.clone();
        std::thread::spawn(move || {
            let _ = Self::run_tray_loop(config_clone);
        });
    }

    #[cfg(windows)]
    fn run_tray_loop(_config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::icon::Icon;
        use tray_icon::TrayIconBuilder;
        
        // Create tray icon
        let icon = Self::create_icon()?;

        let mut _tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("PostureWatch")
            .build()?;

        // Keep alive - tray icon stays until app exits
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if !APP_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    fn create_icon() -> Result<Icon, Box<dyn std::error::Error>> {
        // Create a simple 32x32 blue icon
        let size: u32 = 32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for i in (0..rgba.len()).step_by(4) {
            rgba[i] = 0;       // R
            rgba[i + 1] = 123; // G  
            rgba[i + 2] = 255; // B
            rgba[i + 3] = 255; // A
        }
        Ok(Icon::from_rgba(rgba, size, size)?)
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // No tray on non-Windows for now
    }
}