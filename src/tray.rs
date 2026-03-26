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

        // Get the config file path
        let config_path = match Config::config_path() {
            Some(p) => p,
            None => {
                let _ = DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Configuration Error")
                    .set_text("Could not determine config file location.")
                    .alert()
                    .show();
                return;
            }
        };

        // Create config file if it doesn't exist
        let current_config = Config::load();
        if let Err(e) = current_config.save() {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Configuration Error")
                .set_text(&format!("Failed to create config file: {}", e))
                .alert()
                .show();
            return;
        }

        // Open the config file in the default text editor
        if let Err(e) = std::process::Command::new("notepad.exe")
            .arg(&config_path)
            .spawn()
        {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Configuration Error")
                .set_text(&format!("Failed to open config editor: {}", e))
                .alert()
                .show();
            return;
        }

        let _ = DialogBuilder::message()
            .set_level(MessageLevel::Info)
            .set_title("Configuration")
            .set_text(&format!(
                "Config file opened in Notepad.\n\n\
                Edit the following settings:\n\
                - api_key: Your OpenAI API key\n\
                - strictness: Low, Medium, or High\n\
                - cycle_time_secs: Check interval (5-300)\n\n\
                Save the file and restart the app to apply changes.\n\n\
                Config location:\n{}",
                config_path.display()
            ))
            .alert()
            .show();
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
