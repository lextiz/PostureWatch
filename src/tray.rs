use crate::config::Config;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use std::sync::atomic::AtomicU32;

pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);
pub static MONITORING_ENABLED: AtomicBool = AtomicBool::new(true);
static LAST_POSTURE_SCORE: AtomicU32 = AtomicU32::new(0);

pub fn set_current_posture_status(status: &crate::posture::PostureStatus) {
    let score = match status {
        crate::posture::PostureStatus::Score(score) => *score,
        crate::posture::PostureStatus::NoPerson => 0,
    };
    LAST_POSTURE_SCORE.store(score, Ordering::SeqCst);
}

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

        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("PostureWatch")
            .build()?;
        let mut last_tooltip_state = (u32::MAX, false);

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

                let score = LAST_POSTURE_SCORE.load(Ordering::SeqCst);
                let monitoring_enabled = MONITORING_ENABLED.load(Ordering::SeqCst);
                if last_tooltip_state != (score, monitoring_enabled) {
                    let tooltip = if monitoring_enabled {
                        if score == 0 {
                            "PostureWatch | Score: n/a".to_string()
                        } else {
                            format!("PostureWatch | Score: {score}/10")
                        }
                    } else {
                        "PostureWatch (Paused)".to_string()
                    };
                    let _ = tray.set_tooltip(Some(tooltip));
                    last_tooltip_state = (score, monitoring_enabled);
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
        let mut model_input = nwg::TextInput::default();
        let mut posture_threshold_input = nwg::TextInput::default();
        let mut alert_threshold_input = nwg::TextInput::default();
        let mut interval_input = nwg::TextInput::default();
        let mut desk_raise_check = nwg::CheckBox::default();
        let mut desk_raise_input = nwg::TextInput::default();
        let mut llm_prompt_input = nwg::TextBox::default();
        let mut save_button = nwg::Button::default();
        let mut cancel_button = nwg::Button::default();

        // Each label needs its own variable to persist
        let mut lbl1 = nwg::Label::default();
        let mut lbl2 = nwg::Label::default();
        let mut lbl3 = nwg::Label::default();
        let mut lbl4 = nwg::Label::default();
        let mut lbl5 = nwg::Label::default();
        let mut lbl6 = nwg::Label::default();
        let mut lbl7 = nwg::Label::default();
        let mut lbl8 = nwg::Label::default();
        let mut lbl9 = nwg::Label::default();

        nwg::Window::builder()
            .size((420, 460))
            .position((300, 200))
            .title("PostureWatch Settings")
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut window)
            .ok();

        // API Key
        nwg::Label::builder()
            .text("API Key:")
            .position((20, 20))
            .size((120, 22))
            .parent(&window)
            .build(&mut lbl1)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.api_key)
            .position((150, 18))
            .size((250, 22))
            .parent(&window)
            .build(&mut api_key_input)
            .ok();

        // Model
        nwg::Label::builder()
            .text("Model:")
            .position((20, 55))
            .size((120, 22))
            .parent(&window)
            .build(&mut lbl8)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.model)
            .position((150, 53))
            .size((250, 22))
            .parent(&window)
            .build(&mut model_input)
            .ok();

        // Posture threshold (1-10) + Alert threshold
        nwg::Label::builder()
            .text("Posture threshold:")
            .position((20, 90))
            .size((120, 22))
            .parent(&window)
            .build(&mut lbl2)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.posture_threshold.to_string())
            .position((150, 88))
            .size((50, 22))
            .parent(&window)
            .build(&mut posture_threshold_input)
            .ok();
        nwg::Label::builder()
            .text("(1-10)")
            .position((205, 90))
            .size((45, 22))
            .parent(&window)
            .build(&mut lbl3)
            .ok();

        nwg::Label::builder()
            .text("Alerts after:")
            .position((260, 90))
            .size((80, 22))
            .parent(&window)
            .build(&mut lbl4)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.alert_threshold.to_string())
            .position((345, 88))
            .size((50, 22))
            .parent(&window)
            .build(&mut alert_threshold_input)
            .ok();

        // Check interval
        nwg::Label::builder()
            .text("Check interval:")
            .position((20, 125))
            .size((120, 22))
            .parent(&window)
            .build(&mut lbl5)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.cycle_time_secs.to_string())
            .position((150, 123))
            .size((60, 22))
            .parent(&window)
            .build(&mut interval_input)
            .ok();
        nwg::Label::builder()
            .text("seconds (5-300)")
            .position((220, 125))
            .size((170, 22))
            .parent(&window)
            .build(&mut lbl6)
            .ok();

        // Stand reminder
        nwg::CheckBox::builder()
            .text("Stand reminder")
            .position((20, 160))
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
            .position((160, 160))
            .size((50, 22))
            .parent(&window)
            .build(&mut desk_raise_input)
            .ok();
        nwg::Label::builder()
            .text("minutes (1-480)")
            .position((220, 162))
            .size((170, 22))
            .parent(&window)
            .build(&mut lbl7)
            .ok();

        // Advanced prompt
        nwg::Label::builder()
            .text("Advanced: LLM prompt")
            .position((20, 195))
            .size((380, 22))
            .parent(&window)
            .build(&mut lbl9)
            .ok();
        nwg::TextBox::builder()
            .text(&cfg.llm_prompt)
            .position((20, 220))
            .size((380, 180))
            .parent(&window)
            .focus(true)
            .build(&mut llm_prompt_input)
            .ok();

        // Buttons
        nwg::Button::builder()
            .text("Save")
            .position((200, 410))
            .size((90, 32))
            .parent(&window)
            .build(&mut save_button)
            .ok();
        nwg::Button::builder()
            .text("Cancel")
            .position((310, 410))
            .size((90, 32))
            .parent(&window)
            .build(&mut cancel_button)
            .ok();

        let window_handle = window.handle;
        let save_handle = save_button.handle;
        let cancel_handle = cancel_button.handle;

        let api_key_input = Rc::new(RefCell::new(api_key_input));
        let model_input = Rc::new(RefCell::new(model_input));
        let posture_threshold_input = Rc::new(RefCell::new(posture_threshold_input));
        let alert_threshold_input = Rc::new(RefCell::new(alert_threshold_input));
        let interval_input = Rc::new(RefCell::new(interval_input));
        let desk_raise_check = Rc::new(RefCell::new(desk_raise_check));
        let desk_raise_input = Rc::new(RefCell::new(desk_raise_input));
        let llm_prompt_input = Rc::new(RefCell::new(llm_prompt_input));

        let (ak, mi, pt, at, ii, drc, dri, lpi) = (
            api_key_input.clone(),
            model_input.clone(),
            posture_threshold_input.clone(),
            alert_threshold_input.clone(),
            interval_input.clone(),
            desk_raise_check.clone(),
            desk_raise_input.clone(),
            llm_prompt_input.clone(),
        );

        let handler = nwg::full_bind_event_handler(&window_handle, move |evt, _, handle| {
            if evt == nwg::Event::OnButtonClick && handle == save_handle {
                let posture_th: u32 = match pt.borrow().text().parse() {
                    Ok(v) if (1..=10).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            "Error",
                            "Posture threshold must be 1-10",
                        );
                        return;
                    }
                };
                let alert_th: u32 = match at.borrow().text().parse() {
                    Ok(v) if (1..=10).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            "Error",
                            "Alert threshold must be 1-10",
                        );
                        return;
                    }
                };
                let interval: u64 = match ii.borrow().text().parse() {
                    Ok(v) if (5..=300).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(&window_handle, "Error", "Interval must be 5-300");
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

                let model = mi.borrow().text();
                if model.trim().is_empty() {
                    nwg::modal_info_message(&window_handle, "Error", "Model cannot be empty");
                    return;
                }
                let llm_prompt = lpi.borrow().text();
                if llm_prompt.trim().is_empty() {
                    nwg::modal_info_message(&window_handle, "Error", "LLM prompt cannot be empty");
                    return;
                }

                let mut new_cfg = Config::load();
                new_cfg.api_key = ak.borrow().text();
                new_cfg.model = model;
                new_cfg.llm_prompt = llm_prompt;
                new_cfg.posture_threshold = posture_th;
                new_cfg.alert_threshold = alert_th;
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

#[cfg(all(test, not(windows)))]
mod tests {
    use super::{set_current_posture_status, TrayManager, LAST_POSTURE_SCORE};
    use crate::config::Config;
    use crate::posture::PostureStatus;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    #[test]
    fn setup_tray_is_noop_on_non_windows() {
        let config = Arc::new(TokioMutex::new(Config::default()));
        TrayManager::setup_tray(config);
    }

    #[test]
    fn set_current_posture_status_updates_score_store() {
        set_current_posture_status(&PostureStatus::Score(8));
        assert_eq!(LAST_POSTURE_SCORE.load(Ordering::SeqCst), 8);

        set_current_posture_status(&PostureStatus::NoPerson);
        assert_eq!(LAST_POSTURE_SCORE.load(Ordering::SeqCst), 0);
    }
}
