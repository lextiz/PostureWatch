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

        unsafe {
            let mut msg: MSG = std::mem::zeroed();

            loop {
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

                if !APP_RUNNING.load(Ordering::SeqCst) {
                    break;
                }

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
            rgba[i] = 0;
            rgba[i + 1] = 180;
            rgba[i + 2] = 200;
            rgba[i + 3] = 255;
        }
        Ok(tray_icon::Icon::from_rgba(rgba, size, size)?)
    }

    #[cfg(windows)]
    fn show_configure_dialog(_config: &Arc<TokioMutex<Config>>) {
        use native_windows_gui as nwg;
        use std::cell::RefCell;
        use std::rc::Rc;

        if nwg::init().is_err() {
            return;
        }

        let current_config = Config::load();

        let mut window = nwg::Window::default();
        let mut api_key_label = nwg::Label::default();
        let mut api_key_input = nwg::TextInput::default();
        let mut strictness_label = nwg::Label::default();
        let mut strictness_combo = nwg::ComboBox::<&str>::default();
        let mut interval_label = nwg::Label::default();
        let mut interval_input = nwg::TextInput::default();
        let mut interval_hint = nwg::Label::default();
        let mut save_button = nwg::Button::default();
        let mut cancel_button = nwg::Button::default();

        nwg::Window::builder()
            .size((420, 200))
            .position((300, 200))
            .title("PostureWatch Settings")
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut window)
            .ok();

        nwg::Label::builder()
            .text("OpenAI API Key:")
            .position((20, 20))
            .size((110, 22))
            .parent(&window)
            .build(&mut api_key_label)
            .ok();

        nwg::TextInput::builder()
            .text(&current_config.api_key)
            .position((140, 18))
            .size((260, 22))
            .parent(&window)
            .build(&mut api_key_input)
            .ok();

        nwg::Label::builder()
            .text("Strictness:")
            .position((20, 55))
            .size((110, 22))
            .parent(&window)
            .build(&mut strictness_label)
            .ok();

        nwg::ComboBox::builder()
            .collection(vec!["Low", "Medium", "High"])
            .position((140, 53))
            .size((260, 22))
            .parent(&window)
            .build(&mut strictness_combo)
            .ok();

        let strictness_idx = match current_config.strictness.as_str() {
            "Low" => 0,
            "Medium" => 1,
            "High" => 2,
            _ => 1,
        };
        strictness_combo.set_selection(Some(strictness_idx));

        nwg::Label::builder()
            .text("Check Interval:")
            .position((20, 90))
            .size((110, 22))
            .parent(&window)
            .build(&mut interval_label)
            .ok();

        nwg::TextInput::builder()
            .text(&current_config.cycle_time_secs.to_string())
            .position((140, 88))
            .size((80, 22))
            .parent(&window)
            .build(&mut interval_input)
            .ok();

        nwg::Label::builder()
            .text("seconds (5-300)")
            .position((230, 90))
            .size((170, 22))
            .parent(&window)
            .build(&mut interval_hint)
            .ok();

        nwg::Button::builder()
            .text("Save")
            .position((200, 140))
            .size((90, 32))
            .parent(&window)
            .build(&mut save_button)
            .ok();

        nwg::Button::builder()
            .text("Cancel")
            .position((310, 140))
            .size((90, 32))
            .parent(&window)
            .build(&mut cancel_button)
            .ok();

        let window_handle = window.handle;
        let save_handle = save_button.handle;
        let cancel_handle = cancel_button.handle;

        let api_key_input = Rc::new(RefCell::new(api_key_input));
        let strictness_combo = Rc::new(RefCell::new(strictness_combo));
        let interval_input = Rc::new(RefCell::new(interval_input));

        let api_key_clone = api_key_input.clone();
        let strictness_clone = strictness_combo.clone();
        let interval_clone = interval_input.clone();

        let handler =
            nwg::full_bind_event_handler(&window_handle, move |evt, _evt_data, handle| {
                use nwg::Event;

                match evt {
                    Event::OnButtonClick => {
                        if handle == save_handle {
                            let api_key = api_key_clone.borrow().text();
                            let strictness = match strictness_clone.borrow().selection() {
                                Some(0) => "Low",
                                Some(1) => "Medium",
                                Some(2) => "High",
                                _ => "Medium",
                            };
                            let interval_text = interval_clone.borrow().text();

                            let interval: u64 = match interval_text.parse() {
                                Ok(v) if (5..=300).contains(&v) => v,
                                _ => {
                                    nwg::modal_info_message(
                                        &window_handle,
                                        "Invalid Input",
                                        "Interval must be between 5 and 300.",
                                    );
                                    return;
                                }
                            };

                            let mut new_config = Config::load();
                            new_config.api_key = api_key;
                            new_config.strictness = strictness.to_string();
                            new_config.cycle_time_secs = interval;

                            if let Err(e) = new_config.save() {
                                nwg::modal_info_message(
                                    &window_handle,
                                    "Error",
                                    &format!("Failed to save: {}", e),
                                );
                            } else {
                                nwg::modal_info_message(
                                    &window_handle,
                                    "Saved",
                                    "Settings saved successfully.",
                                );
                                nwg::stop_thread_dispatch();
                            }
                        } else if handle == cancel_handle {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    Event::OnWindowClose => {
                        nwg::stop_thread_dispatch();
                    }
                    _ => {}
                }
            });

        nwg::dispatch_thread_events();
        nwg::unbind_event_handler(&handler);
    }

    #[cfg(windows)]
    fn show_about_dialog() {
        use native_dialog::{DialogBuilder, MessageLevel};

        let _ = DialogBuilder::message()
            .set_level(MessageLevel::Info)
            .set_title("About PostureWatch")
            .set_text(
                "PostureWatch v1.0.7\n\n\
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
