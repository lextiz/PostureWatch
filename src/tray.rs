// System tray module for PostureWatch
// Simplified - tray functionality disabled due to cross-platform issues

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

    // Tray is disabled - relies on process continuing to run
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // Tray functionality temporarily disabled due to cross-platform issues
        // The app will continue running as long as the monitoring loop runs
    }
}