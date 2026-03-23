use crate::config::Config;

#[cfg(windows)]
pub fn notify_bad_posture(config: &Config) {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch")
        .text1(&format!(
            "Please sit up straight! Alert level: {}",
            config.alert_color
        ))
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show();
}

#[cfg(not(windows))]
pub fn notify_bad_posture(config: &Config) {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch")
        .body(&format!(
            "Please sit up straight! Alert level: {}",
            config.alert_color
        ))
        .timeout(notify_rust::Timeout::Milliseconds(
            config.alert_duration_secs as u32 * 1000,
        ))
        .show();
}

#[cfg(windows)]
pub fn notify_desk_raise(_config: &Config) {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Stand up!")
        .text1("It's time to raise your desk or stretch your legs.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(not(windows))]
pub fn notify_desk_raise(_config: &Config) {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Stand up!")
        .body("It's time to raise your desk or stretch your legs.")
        .timeout(notify_rust::Timeout::Never)
        .show();
}
