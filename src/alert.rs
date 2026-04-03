#[cfg(windows)]
pub fn notify_bad_posture() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch")
        .text1("Please sit up straight!")
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show();
}

#[cfg(not(windows))]
pub fn notify_bad_posture() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch")
        .body("Please sit up straight!")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();
}

#[cfg(windows)]
pub fn notify_desk_raise() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Stand up!")
        .text1("Time to raise your desk or stretch your legs.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn notify_break_reminder() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Break reminder")
        .text1("You've been at your desk for a while. Please take a break.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
pub fn notify_session_screen_time_limit() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Session limit reached")
        .text1("Screen time session limit reached. Please take a break.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(windows)]
pub fn notify_daily_screen_time_limit() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Daily limit reached")
        .text1("Daily screen time limit reached. Please rest your eyes.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn notify_break_reminder() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Break reminder")
        .body("You've been at your desk for a while. Please take a break.")
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_session_screen_time_limit() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Session limit reached")
        .body("Screen time session limit reached. Please take a break.")
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_daily_screen_time_limit() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Daily limit reached")
        .body("Daily screen time limit reached. Please rest your eyes.")
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

#[cfg(not(windows))]
pub fn notify_desk_raise() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Stand up!")
        .body("Time to raise your desk or stretch your legs.")
        .timeout(notify_rust::Timeout::Never)
        .show();
}

#[cfg(windows)]
pub fn notify_api_setup_needed(config_path: &str, details: &str) {
    use crate::log_error;
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch setup required")
        .text1("Your API key is missing or not working.")
        .text2(&format!(
            "Open: {config_path} | Get key: platform.openai.com/api-keys"
        ))
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
    log_error!("API setup required: {}", details);
}

#[cfg(not(windows))]
pub fn notify_api_setup_needed(config_path: &str, details: &str) {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch setup required")
        .body(&format!(
            "Your API key is missing or invalid.\n\n1) Open config: {config_path}\n2) Add api_key = \"sk-...\"\n3) Save and restart Posture Watch\n4) Create key: https://platform.openai.com/api-keys\n\nDetails: {details}"
        ))
        .timeout(notify_rust::Timeout::Never)
        .show();
}

#[cfg(test)]
mod tests {
    #[test]
    fn notifications_do_not_panic() {
        super::notify_bad_posture();
        super::notify_desk_raise();
        super::notify_break_reminder();
        super::notify_session_screen_time_limit();
        super::notify_daily_screen_time_limit();
        super::notify_api_setup_needed("config.toml", "test details");
    }
}
