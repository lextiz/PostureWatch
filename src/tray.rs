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
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        std::thread::spawn(|| {
            let _ = Self::run_tray_loop();
        });
    }

    #[cfg(windows)]
    fn run_tray_loop() -> Result<(), Box<dyn std::error::Error>> {
        use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
        use tray_icon::TrayIconBuilder;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };

        let icon = Self::create_icon()?;
        let pause_item = MenuItem::with_id("pause", "Pause", true, None);

        let menu = Menu::with_items(&[
            &MenuItem::with_id("configure", "Configure...", true, None),
            &pause_item,
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("about", "About", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("exit", "Exit", true, None),
        ])?;

        let _tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("PostureWatch")
            .build()?;

        let menu_channel = MenuEvent::receiver();

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            loop {
                if let Ok(event) = menu_channel.try_recv() {
                    match event.id.0.as_str() {
                        "configure" => Self::show_configure_dialog(),
                        "pause" => {
                            let enabled = MONITORING_ENABLED.load(Ordering::SeqCst);
                            MONITORING_ENABLED.store(!enabled, Ordering::SeqCst);
                            pause_item.set_text(if enabled { "Resume" } else { "Pause" });
                        }
                        "about" => Self::show_about_dialog(),
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
    fn show_configure_dialog() {
        use native_windows_gui as nwg;
        use std::cell::RefCell;
        use std::rc::Rc;

        if nwg::init().is_err() {
            return;
        }

        let cfg = Config::load();
        let mut window = nwg::Window::default();
        let mut api_key_input = nwg::TextInput::default();
        let mut strictness_combo = nwg::ComboBox::<&str>::default();
        let mut threshold_input = nwg::TextInput::default();
        let mut interval_input = nwg::TextInput::default();
        let mut desk_raise_check = nwg::CheckBox::default();
        let mut desk_raise_input = nwg::TextInput::default();
        let mut save_button = nwg::Button::default();
        let mut cancel_button = nwg::Button::default();

        nwg::Window::builder()
            .size((420, 260))
            .position((300, 200))
            .title("PostureWatch Settings")
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut window)
            .ok();

        let mut label = nwg::Label::default();

        // API Key
        nwg::Label::builder()
            .text("API Key:")
            .position((20, 20))
            .size((110, 22))
            .parent(&window)
            .build(&mut label)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.api_key)
            .position((140, 18))
            .size((260, 22))
            .parent(&window)
            .build(&mut api_key_input)
            .ok();

        // Strictness + Threshold on same row
        nwg::Label::builder()
            .text("Strictness:")
            .position((20, 55))
            .size((110, 22))
            .parent(&window)
            .build(&mut label)
            .ok();
        nwg::ComboBox::builder()
            .collection(vec!["Low", "Medium", "High"])
            .position((140, 53))
            .size((100, 22))
            .parent(&window)
            .build(&mut strictness_combo)
            .ok();
        strictness_combo.set_selection(Some(match cfg.strictness.as_str() {
            "Low" => 0,
            "High" => 2,
            _ => 1,
        }));

        nwg::Label::builder()
            .text("Threshold:")
            .position((250, 55))
            .size((70, 22))
            .parent(&window)
            .build(&mut label)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.alert_threshold.to_string())
            .position((330, 53))
            .size((70, 22))
            .parent(&window)
            .build(&mut threshold_input)
            .ok();

        // Check interval
        nwg::Label::builder()
            .text("Check interval:")
            .position((20, 90))
            .size((110, 22))
            .parent(&window)
            .build(&mut label)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.cycle_time_secs.to_string())
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
            .build(&mut label)
            .ok();

        // Stand reminder
        nwg::CheckBox::builder()
            .text("Stand reminder")
            .position((20, 125))
            .size((130, 22))
            .parent(&window)
            .check_state(if cfg.desk_raise_enabled {
                nwg::CheckBoxState::Checked
            } else {
                nwg::CheckBoxState::Unchecked
            })
            .build(&mut desk_raise_check)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.desk_raise_interval_mins.to_string())
            .position((160, 125))
            .size((60, 22))
            .parent(&window)
            .build(&mut desk_raise_input)
            .ok();
        nwg::Label::builder()
            .text("minutes (1-480)")
            .position((230, 127))
            .size((170, 22))
            .parent(&window)
            .build(&mut label)
            .ok();

        // Buttons
        nwg::Button::builder()
            .text("Save")
            .position((200, 200))
            .size((90, 32))
            .parent(&window)
            .build(&mut save_button)
            .ok();
        nwg::Button::builder()
            .text("Cancel")
            .position((310, 200))
            .size((90, 32))
            .parent(&window)
            .build(&mut cancel_button)
            .ok();

        let window_handle = window.handle;
        let save_handle = save_button.handle;
        let cancel_handle = cancel_button.handle;

        let api_key_input = Rc::new(RefCell::new(api_key_input));
        let strictness_combo = Rc::new(RefCell::new(strictness_combo));
        let threshold_input = Rc::new(RefCell::new(threshold_input));
        let interval_input = Rc::new(RefCell::new(interval_input));
        let desk_raise_check = Rc::new(RefCell::new(desk_raise_check));
        let desk_raise_input = Rc::new(RefCell::new(desk_raise_input));

        let (ak, sc, th, ii, drc, dri) = (
            api_key_input.clone(),
            strictness_combo.clone(),
            threshold_input.clone(),
            interval_input.clone(),
            desk_raise_check.clone(),
            desk_raise_input.clone(),
        );

        let handler = nwg::full_bind_event_handler(&window_handle, move |evt, _, handle| {
            if evt == nwg::Event::OnButtonClick && handle == save_handle {
                let interval: u64 = match ii.borrow().text().parse() {
                    Ok(v) if (5..=300).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(&window_handle, "Error", "Interval must be 5-300");
                        return;
                    }
                };
                let threshold: u32 = match th.borrow().text().parse() {
                    Ok(v) if (1..=10).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(&window_handle, "Error", "Threshold must be 1-10");
                        return;
                    }
                };
                let desk_mins: u64 = match dri.borrow().text().parse() {
                    Ok(v) if (1..=480).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            "Error",
                            "Stand reminder must be 1-480 minutes",
                        );
                        return;
                    }
                };

                let mut new_cfg = Config::load();
                new_cfg.api_key = ak.borrow().text();
                new_cfg.strictness =
                    ["Low", "Medium", "High"][sc.borrow().selection().unwrap_or(1)].to_string();
                new_cfg.alert_threshold = threshold;
                new_cfg.cycle_time_secs = interval;
                new_cfg.desk_raise_enabled =
                    drc.borrow().check_state() == nwg::CheckBoxState::Checked;
                new_cfg.desk_raise_interval_mins = desk_mins;

                if new_cfg.save().is_ok() {
                    nwg::modal_info_message(&window_handle, "Saved", "Settings saved.");
                }
                nwg::stop_thread_dispatch();
            } else if (evt == nwg::Event::OnButtonClick && handle == cancel_handle)
                || evt == nwg::Event::OnWindowClose
            {
                nwg::stop_thread_dispatch();
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
            .set_text("PostureWatch v1.0\n\nAI-powered posture monitoring.")
            .alert()
            .show();
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {}
}
