use crate::config::Config;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use std::sync::atomic::{AtomicU32, AtomicU64};

#[cfg(windows)]
use crate::i18n::{self, Key, Language};

pub static APP_RUNNING: AtomicBool = AtomicBool::new(true);
pub static MONITORING_ENABLED: AtomicBool = AtomicBool::new(true);
static LAST_POSTURE_SCORE: AtomicU32 = AtomicU32::new(0);
static DAY_SCREEN_TIME_SECS: AtomicU64 = AtomicU64::new(0);
static SESSION_SCREEN_TIME_SECS: AtomicU64 = AtomicU64::new(0);

pub fn set_current_posture_status(status: &crate::posture::PostureStatus) {
    let score = match status {
        crate::posture::PostureStatus::Score(score) => *score,
        crate::posture::PostureStatus::NoPerson => 0,
    };
    LAST_POSTURE_SCORE.store(score, Ordering::SeqCst);
}

pub fn set_screen_time(day_secs: u64, session_secs: u64) {
    DAY_SCREEN_TIME_SECS.store(day_secs, Ordering::SeqCst);
    SESSION_SCREEN_TIME_SECS.store(session_secs, Ordering::SeqCst);
}

#[cfg(windows)]
fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
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

        let language = Language::from_config();
        let icon = Self::create_icon()?;
        let pause_item = MenuItem::with_id("pause", i18n::text(language, Key::Pause), true, None);

        let menu = Menu::with_items(&[
            &MenuItem::with_id(
                "configure",
                i18n::text(language, Key::Configure),
                true,
                None,
            ),
            &pause_item,
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("about", i18n::text(language, Key::About), true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("exit", i18n::text(language, Key::Exit), true, None),
        ])?;

        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("PostureWatch")
            .build()?;
        let mut last_tooltip_state = (u32::MAX, false, u64::MAX, u64::MAX);

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
                            let language = Language::from_config();
                            pause_item.set_text(if enabled {
                                i18n::text(language, Key::Resume)
                            } else {
                                i18n::text(language, Key::Pause)
                            });
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
                let day_secs = DAY_SCREEN_TIME_SECS.load(Ordering::SeqCst);
                let session_secs = SESSION_SCREEN_TIME_SECS.load(Ordering::SeqCst);
                if last_tooltip_state != (score, monitoring_enabled, day_secs, session_secs) {
                    let language = Language::from_config();
                    let tooltip = if monitoring_enabled {
                        let times = format!(
                            " | {}: {} | {}: {}",
                            i18n::text(language, Key::TrayToday),
                            format_duration(day_secs),
                            i18n::text(language, Key::TraySession),
                            format_duration(session_secs)
                        );
                        if score == 0 {
                            format!(
                                "PostureWatch | {}: n/a{}",
                                i18n::text(language, Key::TrayScore),
                                times
                            )
                        } else {
                            format!(
                                "PostureWatch | {}: {score}/10{}",
                                i18n::text(language, Key::TrayScore),
                                times
                            )
                        }
                    } else {
                        i18n::text(language, Key::TrayPaused).to_string()
                    };
                    let _ = tray.set_tooltip(Some(tooltip));
                    last_tooltip_state = (score, monitoring_enabled, day_secs, session_secs);
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
        let language = Language::from_code(&cfg.language);
        let mut window = nwg::Window::default();
        let mut api_key_input = nwg::TextInput::default();
        let mut model_input = nwg::TextInput::default();
        let mut posture_threshold_input = nwg::TextInput::default();
        let mut alert_threshold_input = nwg::TextInput::default();
        let mut interval_input = nwg::TextInput::default();
        let mut camera_index_input = nwg::TextInput::default();
        let mut keep_camera_on_check = nwg::CheckBox::default();
        let mut desk_raise_check = nwg::CheckBox::default();
        let mut desk_raise_input = nwg::TextInput::default();
        let mut break_reminder_check = nwg::CheckBox::default();
        let mut break_after_input = nwg::TextInput::default();
        let mut day_limit_input = nwg::TextInput::default();
        let mut break_repeat_input = nwg::TextInput::default();
        let mut language_input = nwg::ComboBox::<String>::default();
        let mut llm_prompt_input = nwg::TextBox::default();
        let mut save_button = nwg::Button::default();
        let mut cancel_button = nwg::Button::default();
        let mut config_tooltips = nwg::Tooltip::default();

        // Each label needs its own variable to persist
        let mut lbl1 = nwg::Label::default();
        let mut lbl2 = nwg::Label::default();
        let mut lbl3 = nwg::Label::default();
        let mut lbl4 = nwg::Label::default();
        let mut lbl5 = nwg::Label::default();
        let mut lbl6 = nwg::Label::default();
        let mut lbl7 = nwg::Label::default();
        let mut lbl8 = nwg::Label::default();
        let mut lbl10 = nwg::Label::default();
        let mut lbl11 = nwg::Label::default();
        let mut lbl12 = nwg::Label::default();
        let mut lbl13 = nwg::Label::default();
        let mut lbl14 = nwg::Label::default();
        let mut lbl15 = nwg::Label::default();
        let mut lbl16 = nwg::Label::default();

        nwg::Window::builder()
            .size((560, 760))
            .position((300, 200))
            .title(i18n::text(language, Key::SettingsTitle))
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut window)
            .ok();

        // API Key
        nwg::Label::builder()
            .text(i18n::text(language, Key::ApiKeyLabel))
            .position((20, 20))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl1)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.api_key)
            .position((210, 18))
            .size((330, 24))
            .parent(&window)
            .build(&mut api_key_input)
            .ok();

        // Model
        nwg::Label::builder()
            .text(i18n::text(language, Key::ModelLabel))
            .position((20, 55))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl8)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.model)
            .position((210, 53))
            .size((330, 24))
            .parent(&window)
            .build(&mut model_input)
            .ok();

        // Posture threshold (1-10) + Alert threshold
        nwg::Label::builder()
            .text(i18n::text(language, Key::PostureThresholdLabel))
            .position((20, 90))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl2)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.posture_threshold.to_string())
            .position((210, 88))
            .size((70, 24))
            .parent(&window)
            .build(&mut posture_threshold_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::PostureRangeHint))
            .position((288, 90))
            .size((60, 24))
            .parent(&window)
            .build(&mut lbl3)
            .ok();

        nwg::Label::builder()
            .text(i18n::text(language, Key::AlertsAfterLabel))
            .position((360, 90))
            .size((110, 24))
            .parent(&window)
            .build(&mut lbl4)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.alert_threshold.to_string())
            .position((470, 88))
            .size((70, 24))
            .parent(&window)
            .build(&mut alert_threshold_input)
            .ok();

        // Check interval
        nwg::Label::builder()
            .text(i18n::text(language, Key::CheckIntervalLabel))
            .position((20, 125))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl5)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.cycle_time_secs.to_string())
            .position((210, 123))
            .size((80, 24))
            .parent(&window)
            .build(&mut interval_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::CheckIntervalHint))
            .position((300, 125))
            .size((220, 24))
            .parent(&window)
            .build(&mut lbl6)
            .ok();

        nwg::CheckBox::builder()
            .text(i18n::text(language, Key::KeepCameraOnLabel))
            .position((20, 185))
            .size((520, 24))
            .parent(&window)
            .check_state(if cfg.keep_camera_on {
                nwg::CheckBoxState::Checked
            } else {
                nwg::CheckBoxState::Unchecked
            })
            .build(&mut keep_camera_on_check)
            .ok();

        nwg::Label::builder()
            .text(i18n::text(language, Key::CameraIndexLabel))
            .position((20, 155))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl13)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.camera_index.map(|v| v.to_string()).unwrap_or_default())
            .position((210, 153))
            .size((80, 24))
            .parent(&window)
            .build(&mut camera_index_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::CameraIndexHint))
            .position((300, 155))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl14)
            .ok();

        // Stand reminder
        nwg::CheckBox::builder()
            .text(i18n::text(language, Key::StandReminderLabel))
            .position((20, 220))
            .size((180, 24))
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
            .position((210, 220))
            .size((80, 24))
            .parent(&window)
            .build(&mut desk_raise_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::StandReminderHint))
            .position((220, 222))
            .size((220, 24))
            .parent(&window)
            .build(&mut lbl7)
            .ok();

        // Break reminder
        nwg::CheckBox::builder()
            .text(i18n::text(language, Key::BreakReminderLabel))
            .position((20, 255))
            .size((180, 24))
            .parent(&window)
            .check_state(if cfg.break_reminder_enabled {
                nwg::CheckBoxState::Checked
            } else {
                nwg::CheckBoxState::Unchecked
            })
            .build(&mut break_reminder_check)
            .ok();
        nwg::TextInput::builder()
            .text(&cfg.max_session_screen_time_mins.to_string())
            .position((210, 255))
            .size((80, 24))
            .parent(&window)
            .build(&mut break_after_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::SessionMaxHint))
            .position((300, 257))
            .size((230, 24))
            .parent(&window)
            .build(&mut lbl10)
            .ok();

        nwg::TextInput::builder()
            .text(&cfg.max_daily_screen_time_mins.to_string())
            .position((210, 287))
            .size((80, 24))
            .parent(&window)
            .build(&mut day_limit_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::DayMaxHint))
            .position((300, 289))
            .size((230, 24))
            .parent(&window)
            .build(&mut lbl11)
            .ok();

        nwg::TextInput::builder()
            .text(&cfg.break_reminder_repeat_secs.to_string())
            .position((210, 319))
            .size((80, 24))
            .parent(&window)
            .build(&mut break_repeat_input)
            .ok();
        nwg::Label::builder()
            .text(i18n::text(language, Key::NotifyEveryHint))
            .position((300, 321))
            .size((240, 24))
            .parent(&window)
            .build(&mut lbl12)
            .ok();

        // Advanced prompt
        nwg::Label::builder()
            .text(i18n::text(language, Key::LanguageLabel))
            .position((20, 354))
            .size((180, 24))
            .parent(&window)
            .build(&mut lbl15)
            .ok();
        let language_options = vec!["en".to_string(), "ru".to_string()];
        let language_selected_index = if language == Language::Ru { 1 } else { 0 };
        nwg::ComboBox::builder()
            .collection(language_options)
            .selected_index(Some(language_selected_index))
            .position((210, 352))
            .size((80, 24))
            .parent(&window)
            .build(&mut language_input)
            .ok();

        // Advanced prompt
        nwg::Label::builder()
            .text(i18n::text(language, Key::AdvancedPromptLabel))
            .position((20, 390))
            .size((520, 24))
            .parent(&window)
            .build(&mut lbl16)
            .ok();
        nwg::TextBox::builder()
            .text(&cfg.llm_prompt)
            .position((20, 418))
            .size((520, 240))
            .parent(&window)
            .focus(true)
            .build(&mut llm_prompt_input)
            .ok();

        // Buttons
        nwg::Button::builder()
            .text(i18n::text(language, Key::Save))
            .position((340, 678))
            .size((96, 36))
            .parent(&window)
            .build(&mut save_button)
            .ok();
        nwg::Button::builder()
            .text(i18n::text(language, Key::Cancel))
            .position((444, 678))
            .size((96, 36))
            .parent(&window)
            .build(&mut cancel_button)
            .ok();

        nwg::Tooltip::builder()
            .register(
                &api_key_input,
                i18n::text(language, Key::TooltipApiCredentials),
            )
            .register(&model_input, i18n::text(language, Key::TooltipModel))
            .register(
                &posture_threshold_input,
                i18n::text(language, Key::TooltipPostureThreshold),
            )
            .register(
                &alert_threshold_input,
                i18n::text(language, Key::TooltipAlertThreshold),
            )
            .register(&interval_input, i18n::text(language, Key::TooltipInterval))
            .register(
                &camera_index_input,
                i18n::text(language, Key::TooltipCameraIndex),
            )
            .register(
                &keep_camera_on_check,
                i18n::text(language, Key::TooltipKeepCameraOn),
            )
            .register(
                &desk_raise_check,
                i18n::text(language, Key::TooltipStandReminderEnabled),
            )
            .register(
                &desk_raise_input,
                i18n::text(language, Key::TooltipStandReminderInterval),
            )
            .register(
                &break_reminder_check,
                i18n::text(language, Key::TooltipBreakReminderEnabled),
            )
            .register(
                &break_after_input,
                i18n::text(language, Key::TooltipSessionLimit),
            )
            .register(
                &day_limit_input,
                i18n::text(language, Key::TooltipDailyLimit),
            )
            .register(
                &break_repeat_input,
                i18n::text(language, Key::TooltipBreakRepeat),
            )
            .register(&language_input, i18n::text(language, Key::TooltipLanguage))
            .register(&llm_prompt_input, i18n::text(language, Key::TooltipPrompt))
            .build(&mut config_tooltips)
            .ok();

        let window_handle = window.handle;
        let save_handle = save_button.handle;
        let cancel_handle = cancel_button.handle;

        let api_key_input = Rc::new(RefCell::new(api_key_input));
        let model_input = Rc::new(RefCell::new(model_input));
        let posture_threshold_input = Rc::new(RefCell::new(posture_threshold_input));
        let alert_threshold_input = Rc::new(RefCell::new(alert_threshold_input));
        let interval_input = Rc::new(RefCell::new(interval_input));
        let camera_index_input = Rc::new(RefCell::new(camera_index_input));
        let desk_raise_check = Rc::new(RefCell::new(desk_raise_check));
        let keep_camera_on_check = Rc::new(RefCell::new(keep_camera_on_check));
        let desk_raise_input = Rc::new(RefCell::new(desk_raise_input));
        let break_reminder_check = Rc::new(RefCell::new(break_reminder_check));
        let break_after_input = Rc::new(RefCell::new(break_after_input));
        let day_limit_input = Rc::new(RefCell::new(day_limit_input));
        let break_repeat_input = Rc::new(RefCell::new(break_repeat_input));
        let language_input = Rc::new(RefCell::new(language_input));
        let llm_prompt_input = Rc::new(RefCell::new(llm_prompt_input));

        let (ak, mi, pt, at, ii, cii, kco, drc, dri, brc, bai, dli, bri, li, lpi) = (
            api_key_input.clone(),
            model_input.clone(),
            posture_threshold_input.clone(),
            alert_threshold_input.clone(),
            interval_input.clone(),
            camera_index_input.clone(),
            keep_camera_on_check.clone(),
            desk_raise_check.clone(),
            desk_raise_input.clone(),
            break_reminder_check.clone(),
            break_after_input.clone(),
            day_limit_input.clone(),
            break_repeat_input.clone(),
            language_input.clone(),
            llm_prompt_input.clone(),
        );

        let handler = nwg::full_bind_event_handler(&window_handle, move |evt, _, handle| {
            if evt == nwg::Event::OnButtonClick && handle == save_handle {
                let posture_th: u32 = match pt.borrow().text().parse() {
                    Ok(v) if (1..=10).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationPostureThreshold),
                        );
                        return;
                    }
                };
                let alert_th: u32 = match at.borrow().text().parse() {
                    Ok(v) if (1..=10).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationAlertThreshold),
                        );
                        return;
                    }
                };
                let interval: u64 = match ii.borrow().text().parse() {
                    Ok(v) if (5..=300).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationInterval),
                        );
                        return;
                    }
                };
                let camera_index = {
                    let raw = cii.borrow().text();
                    if raw.trim().is_empty() {
                        None
                    } else {
                        match raw.parse::<u32>() {
                            Ok(v) => Some(v),
                            Err(_) => {
                                nwg::modal_info_message(
                                    &window_handle,
                                    i18n::text(language, Key::DialogErrorTitle),
                                    i18n::text(language, Key::ValidationCameraIndex),
                                );
                                return;
                            }
                        }
                    }
                };
                let desk_mins: u64 = match dri.borrow().text().parse() {
                    Ok(v) if (1..=480).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationStandReminder),
                        );
                        return;
                    }
                };
                let break_after_mins: u64 = match bai.borrow().text().parse() {
                    Ok(v) if (1..=480).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationBreakAfter),
                        );
                        return;
                    }
                };
                let day_limit_mins: u64 = match dli.borrow().text().parse() {
                    Ok(v) if (30..=1440).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationDailyMax),
                        );
                        return;
                    }
                };
                let break_repeat_secs: u64 = match bri.borrow().text().parse() {
                    Ok(v) if (5..=600).contains(&v) => v,
                    _ => {
                        nwg::modal_info_message(
                            &window_handle,
                            i18n::text(language, Key::DialogErrorTitle),
                            i18n::text(language, Key::ValidationBreakRepeat),
                        );
                        return;
                    }
                };
                let model = mi.borrow().text();
                if model.trim().is_empty() {
                    nwg::modal_info_message(
                        &window_handle,
                        i18n::text(language, Key::DialogErrorTitle),
                        i18n::text(language, Key::ValidationModelEmpty),
                    );
                    return;
                }
                let llm_prompt = lpi.borrow().text();
                if llm_prompt.trim().is_empty() {
                    nwg::modal_info_message(
                        &window_handle,
                        i18n::text(language, Key::DialogErrorTitle),
                        i18n::text(language, Key::ValidationPromptEmpty),
                    );
                    return;
                }
                let selected_index = li.borrow().selection();
                let language_code = match selected_index {
                    Some(1) => "ru".to_string(),
                    _ => "en".to_string(),
                };
                if language_code != "en" && language_code != "ru" {
                    let selected_language = Language::from_code(&language_code);
                    nwg::modal_info_message(
                        &window_handle,
                        i18n::text(selected_language, Key::DialogErrorTitle),
                        i18n::text(selected_language, Key::LanguageValidationError),
                    );
                    return;
                }

                let mut new_cfg = Config::load();
                new_cfg.api_key = ak.borrow().text();
                new_cfg.model = model;
                new_cfg.llm_prompt = llm_prompt;
                new_cfg.posture_threshold = posture_th;
                new_cfg.alert_threshold = alert_th;
                new_cfg.cycle_time_secs = interval;
                new_cfg.camera_index = camera_index;
                new_cfg.keep_camera_on = kco.borrow().check_state() == nwg::CheckBoxState::Checked;
                new_cfg.desk_raise_enabled =
                    drc.borrow().check_state() == nwg::CheckBoxState::Checked;
                new_cfg.desk_raise_interval_mins = desk_mins;
                new_cfg.break_reminder_enabled =
                    brc.borrow().check_state() == nwg::CheckBoxState::Checked;
                new_cfg.max_session_screen_time_mins = break_after_mins;
                new_cfg.max_daily_screen_time_mins = day_limit_mins;
                new_cfg.break_reminder_repeat_secs = break_repeat_secs;
                new_cfg.language = language_code;

                if new_cfg.save().is_ok() {
                    let saved_language = Language::from_code(&new_cfg.language);
                    nwg::modal_info_message(
                        &window_handle,
                        i18n::text(saved_language, Key::DialogSavedTitle),
                        i18n::text(saved_language, Key::DialogSettingsSaved),
                    );
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
        let language = Language::from_config();
        let _ = DialogBuilder::message()
            .set_level(MessageLevel::Info)
            .set_title(i18n::text(language, Key::AboutTitle))
            .set_text(i18n::text(language, Key::AboutBody))
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
