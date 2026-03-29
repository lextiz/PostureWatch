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
pub fn notify_break_reminder() {
    use winrt_notification::{Duration, Sound, Toast};
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Posture Watch - Break reminder")
        .text1("You've been at your desk for a while. Please take a break.")
        .sound(Some(Sound::Default))
        .duration(Duration::Long)
        .show();
}

#[cfg(not(windows))]
pub fn notify_break_reminder() {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary("Posture Watch - Break reminder")
        .body("You've been at your desk for a while. Please take a break.")
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

#[cfg(test)]
mod tests {
    #[test]
    fn notifications_do_not_panic() {
        super::notify_bad_posture();
        super::notify_desk_raise();
        super::notify_break_reminder();
    }
}
