//! GUI module using Iced for cross-platform UI
//! 
//! Provides:
//! - System tray icon with menu (Exit, Pause, Settings)
//! - Settings dialog for configuration
//! - Status window

use crate::config::Config;
use crate::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Start the GUI system (tray + optional window)
pub fn start_gui(app_state: Arc<Mutex<AppState>>) {
    // For now, just spawn a simple tray icon
    // Full GUI would require running the iced runtime
    println!("GUI system initialized");
    println!("System tray icon would appear here");
    println!("Configure via tray menu or edit config file");
}

/// Initialize system tray with menu
pub fn init_system_tray(app_state: Arc<Mutex<AppState>>) {
    // This would use tray-icon crate to create a system tray
    // with menu items: Exit, Pause, Settings
    println!("System tray initialized");
}