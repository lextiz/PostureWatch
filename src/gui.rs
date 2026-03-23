//! GUI module for system tray and window management
//!
//! Provides:
//! - System tray icon with menu (Exit, Pause, Settings)
//! - Settings dialog for configuration
//! - Status display

use crate::AppState;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// Initialize system tray with menu (placeholder for now)
pub fn init_system_tray() {
    println!("System tray initialized with menu:");
    println!("  - Exit: Stop the application");
    println!("  - Pause/Resume: Toggle monitoring");
    println!("  - Settings: Open configuration");
    println!("");
    println!("Configure settings via config file or edit below:");
    println!("  Windows: %APPDATA%\\posturewatch\\config.toml");
    println!("  Linux: ~/.config/posturewatch/config.toml");
}

/// Start GUI event loop (placeholder)
pub fn start_gui(_app_state: Arc<TokioMutex<AppState>>) {
    // Would run GUI event loop here
    // For now, just print a message
    println!("GUI event loop would run here");
}
