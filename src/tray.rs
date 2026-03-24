// System tray module for PostureWatch

#![allow(dead_code)]

use crate::config::Config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// Global flag to signal app shutdown
pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);

pub struct TrayManager;

impl TrayManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        use std::thread;

        thread::spawn(|| {
            let _ = Self::run_tray();
        });
    }

    #[cfg(windows)]
    fn run_tray() -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::{
            menu::{Menu, MenuItem},
            tray_icon::Icon,
            TrayIconBuilder,
        };

        // Create menu items - use MenuItem::new with (id, text, enabled)
        let open_item = MenuItem::new("open", "Open Settings", true)?;
        let strictness_low = MenuItem::new("strictness_low", "Low", true)?;
        let strictness_medium = MenuItem::new("strictness_medium", "Medium", true)?;
        let strictness_high = MenuItem::new("strictness_high", "High", true)?;
        let quit_item = MenuItem::new("quit", "Quit", true)?;

        // Build menu using raw API
        let menu = Menu::new(&[
            &open_item,
            &strictness_low,
            &strictness_medium,
            &strictness_high,
            &quit_item,
        ]);

        // Create icon from RGBA data - 32x32 blue square
        let size = 32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for i in (0..rgba.len()).step_by(4) {
            rgba[i] = 0; // R
            rgba[i + 1] = 123; // G
            rgba[i + 2] = 255; // B
            rgba[i + 3] = 255; // A
        }

        let icon = Icon::from_rgba(rgba, size, size)?;

        let _tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(&menu)
            .with_tooltip("PostureWatch")
            .build()?;

        // Keep running
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // No tray on non-Windows for now
    }
}
