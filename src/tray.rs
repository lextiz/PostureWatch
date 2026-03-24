// System tray module for PostureWatch
// Uses tray-icon crate for cross-platform tray functionality

#![allow(dead_code)]

use crate::config::Config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);

pub struct TrayManager;

impl TrayManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn setup_tray(config: Arc<TokioMutex<Config>>) {
        let config_clone = config.clone();
        std::thread::spawn(move || {
            let _ = Self::run_tray_loop(config_clone);
        });
    }

    #[cfg(windows)]
    fn run_tray_loop(_config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::Icon;
        use tray_icon::TrayIconBuilder;

        // Create tray icon from RGBA data
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
    fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
        // Create a simple 32x32 cyan icon (PostureWatch brand color)
        let size: u32 = 32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for i in (0..rgba.len()).step_by(4) {
            rgba[i] = 0; // R
            rgba[i + 1] = 180; // G
            rgba[i + 2] = 200; // B
            rgba[i + 3] = 255; // A
        }
        Ok(tray_icon::Icon::from_rgba(rgba, size, size)?)
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // No tray on non-Windows for now
    }
}
