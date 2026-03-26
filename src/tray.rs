// System tray module for PostureWatch
// Uses tray-icon crate for cross-platform tray functionality

use crate::config::Config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[cfg(windows)]
use std::sync::atomic::Ordering;

pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);
pub static MONITORING_ENABLED: AtomicBool = AtomicBool::new(true);

pub struct TrayManager;

impl TrayManager {
    #[cfg(windows)]
    pub fn setup_tray(config: Arc<TokioMutex<Config>>) {
        std::thread::spawn(move || {
            let _ = Self::run_tray_loop(config);
        });
    }

    #[cfg(windows)]
    fn run_tray_loop(config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
        use tray_icon::TrayIconBuilder;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };

        let icon = Self::create_icon()?;

        // Create menu items with unique IDs
        let configure_item = MenuItem::with_id("configure", "Configure...", true, None);
        let pause_item = MenuItem::with_id("pause", "Pause", true, None);
        let about_item = MenuItem::with_id("about", "About", true, None);
        let exit_item = MenuItem::with_id("exit", "Exit", true, None);

        let menu = Menu::with_items(&[
            &configure_item,
            &pause_item,
            &PredefinedMenuItem::separator(),
            &about_item,
            &PredefinedMenuItem::separator(),
            &exit_item,
        ])?;

        let _tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("PostureWatch - Monitoring your posture")
            .build()?;

        let menu_channel = MenuEvent::receiver();

        // Run Windows message loop
        unsafe {
            let mut msg: MSG = std::mem::zeroed();

            loop {
                // Check for menu events first
                if let Ok(event) = menu_channel.try_recv() {
                    match event.id.0.as_str() {
                        "configure" => {
                            Self::show_configure_dialog(&config);
                        }
                        "pause" => {
                            let currently_enabled = MONITORING_ENABLED.load(Ordering::SeqCst);
                            MONITORING_ENABLED.store(!currently_enabled, Ordering::SeqCst);
                            if currently_enabled {
                                pause_item.set_text("Resume");
                            } else {
                                pause_item.set_text("Pause");
                            }
                        }
                        "about" => {
                            Self::show_about_dialog();
                        }
                        "exit" => {
                            APP_RUNNING.store(false, Ordering::SeqCst);
                            break;
                        }
                        _ => {}
                    }
                }

                // Check app running status
                if !APP_RUNNING.load(Ordering::SeqCst) {
                    break;
                }

                // Process Windows messages (non-blocking with timeout)
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
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

    #[cfg(windows)]
    fn show_configure_dialog(_config: &Arc<TokioMutex<Config>>) {
        use native_dialog::{DialogBuilder, MessageLevel};

        let mut current_config = Config::load();

        // Step 1: API Key
        let api_key = Self::show_api_key_dialog(&current_config.api_key);
        if api_key.is_none() {
            return; // User cancelled
        }

        // Step 2: Strictness (using Yes/No dialogs)
        let strictness = Self::show_strictness_dialog(&current_config.strictness);
        if strictness.is_none() {
            return; // User cancelled
        }

        // Step 3: Monitoring interval
        let interval = Self::show_interval_dialog(current_config.cycle_time_secs);
        if interval.is_none() {
            return; // User cancelled
        }

        // Save configuration
        current_config.api_key = api_key.unwrap();
        current_config.strictness = strictness.unwrap();
        current_config.cycle_time_secs = interval.unwrap();

        if let Err(e) = current_config.save() {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Save Error")
                .set_text(&format!("Failed to save configuration: {}", e))
                .alert()
                .show();
        } else {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Info)
                .set_title("Configuration Saved")
                .set_text("Settings have been saved successfully.\n\nChanges will take effect on the next posture check.")
                .alert()
                .show();
        }
    }

    #[cfg(windows)]
    fn show_api_key_dialog(current: &str) -> Option<String> {
        use native_dialog::{DialogBuilder, MessageLevel};

        // For API key, we'll use a file-based approach that's safe from antivirus
        // Create a temporary file with the current key, open in notepad, read back
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("posturewatch_apikey.txt");

        // Write current key to temp file
        if let Err(_) = std::fs::write(&temp_file, current) {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_text("Could not create temporary file for API key input.")
                .alert()
                .show();
            return None;
        }

        // Show instructions
        let proceed = DialogBuilder::message()
            .set_level(MessageLevel::Info)
            .set_title("OpenAI API Key")
            .set_text(
                "A Notepad window will open with your current API key.\n\n\
                      Edit the key and save the file (Ctrl+S), then close Notepad.\n\n\
                      Click OK to continue or Cancel to skip.",
            )
            .confirm()
            .show()
            .unwrap_or(false);

        if !proceed {
            let _ = std::fs::remove_file(&temp_file);
            return Some(current.to_string()); // Keep current value
        }

        // Open notepad and wait for it to close
        let result = std::process::Command::new("notepad.exe")
            .arg(&temp_file)
            .status();

        if result.is_err() {
            let _ = std::fs::remove_file(&temp_file);
            return Some(current.to_string());
        }

        // Read back the key
        let new_key = std::fs::read_to_string(&temp_file)
            .unwrap_or_else(|_| current.to_string())
            .trim()
            .to_string();

        // Clean up
        let _ = std::fs::remove_file(&temp_file);

        Some(new_key)
    }

    #[cfg(windows)]
    fn show_strictness_dialog(current: &str) -> Option<String> {
        use native_dialog::{DialogBuilder, MessageLevel};

        // Show current setting and options
        let msg = format!(
            "Current strictness: {}\n\n\
             Choose new strictness level:\n\n\
             • Low - Alerts only for very poor posture\n\
             • Medium - Balanced monitoring (recommended)\n\
             • High - Strict posture requirements\n\n\
             Click Yes for Low, No for Medium, or Cancel for High",
            current
        );

        // Use Yes/No/Cancel to select: Yes=Low, No=Medium, Cancel handled separately
        let result = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Strictness Level")
            .set_text(&msg)
            .confirm()
            .show();

        match result {
            Ok(true) => {
                // User clicked Yes - but we need 3 options
                // Let's do a two-step approach
            }
            Ok(false) => {}
            Err(_) => return None,
        }

        // Better approach: Sequential dialogs
        let use_low = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Strictness: Low?")
            .set_text(&format!(
                "Current: {}\n\n\
                 Set strictness to LOW?\n\
                 (Alerts only for very poor posture)\n\n\
                 Click Yes for Low, or No to see other options.",
                current
            ))
            .confirm()
            .show()
            .unwrap_or(false);

        if use_low {
            return Some("Low".to_string());
        }

        let use_medium = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Strictness: Medium?")
            .set_text(
                "Set strictness to MEDIUM?\n\
                      (Balanced monitoring - recommended)\n\n\
                      Click Yes for Medium, or No for High.",
            )
            .confirm()
            .show()
            .unwrap_or(false);

        if use_medium {
            Some("Medium".to_string())
        } else {
            Some("High".to_string())
        }
    }

    #[cfg(windows)]
    fn show_interval_dialog(current: u64) -> Option<u64> {
        use native_dialog::{DialogBuilder, MessageLevel};

        // Offer preset intervals
        let use_10 = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Monitoring Interval")
            .set_text(&format!(
                "Current interval: {} seconds\n\n\
                 Set interval to 10 seconds?\n\
                 (Check posture every 10 seconds)\n\n\
                 Click Yes for 10s, or No to see other options.",
                current
            ))
            .confirm()
            .show()
            .unwrap_or(false);

        if use_10 {
            return Some(10);
        }

        let use_30 = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Monitoring Interval")
            .set_text(
                "Set interval to 30 seconds?\n\n\
                      Click Yes for 30s, or No to see other options.",
            )
            .confirm()
            .show()
            .unwrap_or(false);

        if use_30 {
            return Some(30);
        }

        let use_60 = DialogBuilder::message()
            .set_level(MessageLevel::Question)
            .set_title("Monitoring Interval")
            .set_text(
                "Set interval to 60 seconds (1 minute)?\n\n\
                      Click Yes for 60s, or No for 120s.",
            )
            .confirm()
            .show()
            .unwrap_or(false);

        if use_60 {
            Some(60)
        } else {
            Some(120)
        }
    }

    #[cfg(windows)]
    fn show_about_dialog() {
        use native_dialog::{DialogBuilder, MessageLevel};

        let _ = DialogBuilder::message()
            .set_level(MessageLevel::Info)
            .set_title("About PostureWatch")
            .set_text(
                "PostureWatch v1.0.3\n\n\
                A posture monitoring application that uses your webcam\n\
                and AI to help you maintain good posture.\n\n\
                © 2024 PostureWatch",
            )
            .alert()
            .show();
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        // No tray on non-Windows for now
    }
}
