use crate::config::Config;
use notify_rust::Notification;
use std::time::Duration;

pub fn notify_bad_posture(config: &Config) {
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

pub fn notify_desk_raise(config: &Config) {
    let _ = Notification::new()
        .summary("Posture Watch - Stand up!")
        .body("It's time to raise your desk or stretch your legs.")
        .timeout(notify_rust::Timeout::Never)
        .show();
}
