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

        // Load current config synchronously (we reload to get latest values)
        let current_config = Config::load();

        // API Key dialog
        let api_key = Self::show_input_dialog(
            "OpenAI API Key",
            "Enter your OpenAI API key:",
            &current_config.api_key,
        );
        if api_key.is_none() {
            return; // User cancelled
        }

        // Strictness dialog
        let strictness_options = ["Low", "Medium", "High"];
        let current_strictness_idx = match current_config.strictness.as_str() {
            "Low" => 0,
            "Medium" => 1,
            "High" => 2,
            _ => 1,
        };
        let strictness = Self::show_choice_dialog(
            "Strictness Level",
            "Select posture monitoring strictness:",
            &strictness_options,
            current_strictness_idx,
        );
        if strictness.is_none() {
            return;
        }

        // Monitoring interval dialog
        let interval = Self::show_input_dialog(
            "Monitoring Interval",
            "Enter monitoring interval in seconds (5-300):",
            &current_config.cycle_time_secs.to_string(),
        );
        if interval.is_none() {
            return;
        }

        // Validate and save
        let api_key = api_key.unwrap();
        let strictness = strictness.unwrap();
        let interval_str = interval.unwrap();

        let interval_secs: u64 = match interval_str.parse() {
            Ok(v) if (5..=300).contains(&v) => v,
            _ => {
                let _ = DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Invalid Input")
                    .set_text("Interval must be a number between 5 and 300 seconds.")
                    .alert()
                    .show();
                return;
            }
        };

        // Save configuration
        let mut new_config = current_config;
        new_config.api_key = api_key;
        new_config.strictness = strictness;
        new_config.cycle_time_secs = interval_secs;

        if let Err(e) = new_config.save() {
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
                .set_text("Settings have been saved successfully.")
                .alert()
                .show();
        }
    }

    #[cfg(windows)]
    fn show_input_dialog(title: &str, message: &str, default: &str) -> Option<String> {
        use std::process::{Command, Stdio};

        // Use PowerShell to show an input dialog
        let script = format!(
            r#"
Add-Type -AssemblyName Microsoft.VisualBasic
$result = [Microsoft.VisualBasic.Interaction]::InputBox('{}', '{}', '{}')
Write-Output $result
"#,
            message.replace("'", "''"),
            title.replace("'", "''"),
            default.replace("'", "''")
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    #[cfg(windows)]
    fn show_choice_dialog(
        title: &str,
        message: &str,
        options: &[&str],
        default_idx: usize,
    ) -> Option<String> {
        use native_dialog::{DialogBuilder, MessageLevel};

        // Build options string
        let options_text = options
            .iter()
            .enumerate()
            .map(|(i, opt)| format!("{}. {}", i + 1, opt))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "{}\n\n{}\n\nEnter number (1-{}):",
            message,
            options_text,
            options.len()
        );

        let default_val = (default_idx + 1).to_string();
        let input = Self::show_input_dialog(title, &prompt, &default_val)?;

        let idx: usize = input.trim().parse().ok()?;
        if idx >= 1 && idx <= options.len() {
            Some(options[idx - 1].to_string())
        } else {
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Invalid Selection")
                .set_text(&format!(
                    "Please enter a number between 1 and {}",
                    options.len()
                ))
                .alert()
                .show();
            None
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
