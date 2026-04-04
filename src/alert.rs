use crate::i18n::{self, Key, Language};

#[cfg(windows)]
pub fn notify_bad_posture() {
    use winrt_notification::{Duration, Sound, Toast};
    let language = Language::from_config();
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::NotificationApp))
        .text1(i18n::text(language, Key::BadPosture))
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show();
}

#[cfg(not(windows))]
pub fn notify_bad_posture() {
    use notify_rust::Notification;
    let language = Language::from_config();
    let _ = Notification::new()
        .summary(i18n::text(language, Key::NotificationApp))
        .body(i18n::text(language, Key::BadPosture))
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();
}

#[cfg(windows)]
pub fn notify_desk_raise() {
    use winrt_notification::{Duration, Sound, Toast};
    let language = Language::from_config();
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::StandTitle))
        .text1(i18n::text(language, Key::StandBody))
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn notify_break_reminder() {
    use winrt_notification::{Duration, Sound, Toast};
    let language = Language::from_config();
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::BreakTitle))
        .text1(i18n::text(language, Key::BreakBody))
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
pub fn notify_session_screen_time_limit() {
    use winrt_notification::{Duration, Sound, Toast};
    let language = Language::from_config();
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::SessionLimitTitle))
        .text1(i18n::text(language, Key::SessionLimitBody))
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
pub fn notify_daily_screen_time_limit() {
    use winrt_notification::{Duration, Sound, Toast};
    let language = Language::from_config();
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::DailyLimitTitle))
        .text1(i18n::text(language, Key::DailyLimitBody))
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn notify_break_reminder() {
    use notify_rust::Notification;
    let language = Language::from_config();
    let _ = Notification::new()
        .summary(i18n::text(language, Key::BreakTitle))
        .body(i18n::text(language, Key::BreakBody))
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_session_screen_time_limit() {
    use notify_rust::Notification;
    let language = Language::from_config();
    let _ = Notification::new()
        .summary(i18n::text(language, Key::SessionLimitTitle))
        .body(i18n::text(language, Key::SessionLimitBody))
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_daily_screen_time_limit() {
    use notify_rust::Notification;
    let language = Language::from_config();
    let _ = Notification::new()
        .summary(i18n::text(language, Key::DailyLimitTitle))
        .body(i18n::text(language, Key::DailyLimitBody))
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_desk_raise() {
    use notify_rust::Notification;
    let language = Language::from_config();
    let _ = Notification::new()
        .summary(i18n::text(language, Key::StandTitle))
        .body(i18n::text(language, Key::StandBody))
        .timeout(notify_rust::Timeout::Never)
        .show();
}

#[cfg(windows)]
pub fn notify_api_setup_needed(config_path: &str, details: &str) {
    use crate::log_error;
    use winrt_notification::{Duration, Sound, Toast};

    let language = Language::from_config();
    let line2 = match language {
        Language::En => format!("Open: {config_path} | Get key: platform.openai.com/api-keys"),
        Language::Ru => format!("Откройте: {config_path} | Ключ: platform.openai.com/api-keys"),
    };

    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(i18n::text(language, Key::ApiSetupTitle))
        .text1(i18n::text(language, Key::ApiSetupBody))
        .text2(&line2)
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
    log_error!("API setup required: {}", details);
}

#[cfg(not(windows))]
pub fn notify_api_setup_needed(config_path: &str, details: &str) {
    use notify_rust::Notification;

    let language = Language::from_config();
    let body = match language {
        Language::En => format!(
            "Your API key is missing or invalid.\n\n1) Open config: {config_path}\n2) Add api_key = \"sk-...\"\n3) Save and restart Posture Watch\n4) Create key: https://platform.openai.com/api-keys\n\n{}: {details}",
            i18n::text(language, Key::ApiSetupDetails)
        ),
        Language::Ru => format!(
            "API-ключ отсутствует или неверный.\n\n1) Откройте config: {config_path}\n2) Добавьте api_key = \"sk-...\"\n3) Сохраните и перезапустите Posture Watch\n4) Создайте ключ: https://platform.openai.com/api-keys\n\n{}: {details}",
            i18n::text(language, Key::ApiSetupDetails)
        ),
    };

    let _ = Notification::new()
        .summary(i18n::text(language, Key::ApiSetupSummary))
        .body(&body)
        .timeout(notify_rust::Timeout::Never)
        .show();
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("posturewatch-{name}-{nanos}"))
    }

    fn with_language(language: &str, run: impl FnOnce()) {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let appdata_root = unique_temp_dir("alert-i18n");
        let config_path = appdata_root
            .join("com.posturewatch")
            .join("PostureWatch")
            .join("config.toml");
        fs::create_dir_all(
            config_path
                .parent()
                .expect("config should always have parent directory"),
        )
        .expect("create appdata dirs");

        let mut cfg = Config::default();
        cfg.language = language.to_string();
        fs::write(
            &config_path,
            toml::to_string(&cfg).expect("serialize config"),
        )
        .expect("write config");

        let previous_appdata = std::env::var("APPDATA").ok();
        std::env::set_var("APPDATA", &appdata_root);
        run();

        if let Some(previous) = previous_appdata {
            std::env::set_var("APPDATA", previous);
        } else {
            std::env::remove_var("APPDATA");
        }
    }

    #[test]
    fn notifications_do_not_panic_in_english_and_russian() {
        with_language("en", || {
            super::notify_bad_posture();
            super::notify_desk_raise();
            super::notify_break_reminder();
            super::notify_session_screen_time_limit();
            super::notify_daily_screen_time_limit();
            super::notify_api_setup_needed("config.toml", "test details");
        });

        with_language("ru", || {
            super::notify_bad_posture();
            super::notify_desk_raise();
            super::notify_break_reminder();
            super::notify_session_screen_time_limit();
            super::notify_daily_screen_time_limit();
            super::notify_api_setup_needed("config.toml", "test details");
        });
    }
}
