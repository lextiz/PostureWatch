use super::*;
use std::time::Duration;

#[test]
fn desk_raise_notification_disabled() {
    let config = Config {
        desk_raise_enabled: false,
        ..Config::default()
    };
    let last = Instant::now() - Duration::from_secs(60 * 60 * 24);
    assert!(!should_notify_desk_raise(&config, last));
}

#[test]
fn desk_raise_notification_enabled_and_due() {
    let config = Config {
        desk_raise_enabled: true,
        desk_raise_interval_mins: 1,
        ..Config::default()
    };
    let last = Instant::now() - Duration::from_secs(61);
    assert!(should_notify_desk_raise(&config, last));
}

#[test]
fn desk_raise_notification_enabled_but_not_due() {
    let config = Config {
        desk_raise_enabled: true,
        desk_raise_interval_mins: 2,
        ..Config::default()
    };
    let last = Instant::now() - Duration::from_secs(30);
    assert!(!should_notify_desk_raise(&config, last));
}
