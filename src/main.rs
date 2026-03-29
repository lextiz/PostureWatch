#![windows_subsystem = "windows"]

mod alert;
mod camera;
mod config;
mod logging;
mod posture;
mod posture_monitor;
mod tray;

use config::Config;
use posture::PostureAnalyzer;
use posture_monitor::{AlertEvent, MonitorLogic};
use tray::{APP_RUNNING, MONITORING_ENABLED};

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[derive(Default)]
struct BreakReminderState {
    present_since: Option<Instant>,
    away_since: Option<Instant>,
    last_notified: Option<Instant>,
}

#[tokio::main]
async fn main() {
    logging::init();
    log_info!("PostureWatch starting");

    let config = Config::load();

    let mut camera_state = camera::CameraState::new();
    let config_arc = Arc::new(tokio::sync::Mutex::new(config.clone()));
    let analyzer = PostureAnalyzer::new();
    let mut monitor = MonitorLogic::new(config.posture_threshold, config.alert_threshold);
    let mut last_desk_raise = Instant::now();
    let mut break_reminder_state = BreakReminderState::default();
    let mut was_monitoring_enabled = true;

    tray::TrayManager::setup_tray(config_arc);

    while APP_RUNNING.load(Ordering::SeqCst) {
        if !MONITORING_ENABLED.load(Ordering::SeqCst) {
            if was_monitoring_enabled {
                camera_state.shutdown();
                break_reminder_state = BreakReminderState::default();
            }
            was_monitoring_enabled = false;
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        was_monitoring_enabled = true;

        let current_config = Config::load();

        if should_notify_desk_raise(&current_config, last_desk_raise) {
            alert::notify_desk_raise();
            last_desk_raise = Instant::now();
        }

        if let Ok(frame) = camera_state.capture_frame() {
            match analyzer.analyze(&frame, &current_config).await {
                Ok(status) => {
                    tray::set_current_posture_status(&status);
                    process_break_reminder(&current_config, &status, &mut break_reminder_state);
                    if let AlertEvent::NotifyBadPosture = monitor.process_status(status) {
                        alert::notify_bad_posture();
                    }
                    monitor.set_thresholds(
                        current_config.posture_threshold,
                        current_config.alert_threshold,
                    );
                }
                Err(e) => {
                    log_error!("Analysis failed: {}", e);
                }
            }
        } else {
            log_error!("Failed to capture frame");
        }

        sleep(Duration::from_secs(current_config.cycle_time_secs)).await;
    }

    camera_state.shutdown();

    log_info!("PostureWatch exiting");
}

fn should_notify_desk_raise(config: &Config, last_desk_raise: Instant) -> bool {
    if !config.desk_raise_enabled {
        return false;
    }
    let interval_secs = config.desk_raise_interval_mins * 60;
    last_desk_raise.elapsed().as_secs() >= interval_secs
}

fn process_break_reminder(
    config: &Config,
    status: &posture::PostureStatus,
    state: &mut BreakReminderState,
) {
    if !config.break_reminder_enabled {
        *state = BreakReminderState::default();
        return;
    }

    let now = Instant::now();
    let break_after = Duration::from_secs(config.break_reminder_after_mins.max(1) * 60);
    let repeat_every = Duration::from_secs(config.break_reminder_repeat_secs.max(1));
    let reset_after = Duration::from_secs(config.break_reset_after_mins.max(1) * 60);

    match status {
        posture::PostureStatus::Score(_) => {
            state.away_since = None;
            if state.present_since.is_none() {
                state.present_since = Some(now);
            }

            if let Some(present_since) = state.present_since {
                if now.duration_since(present_since) >= break_after {
                    let should_notify = match state.last_notified {
                        Some(last_notified) => now.duration_since(last_notified) >= repeat_every,
                        None => true,
                    };
                    if should_notify {
                        alert::notify_break_reminder();
                        state.last_notified = Some(now);
                    }
                }
            }
        }
        posture::PostureStatus::NoPerson => {
            if state.away_since.is_none() {
                state.away_since = Some(now);
            }

            if let Some(away_since) = state.away_since {
                if now.duration_since(away_since) >= reset_after {
                    *state = BreakReminderState::default();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn break_reminder_disabled_resets_state() {
        let config = Config {
            break_reminder_enabled: false,
            ..Config::default()
        };
        let mut state = BreakReminderState {
            present_since: Some(Instant::now()),
            away_since: Some(Instant::now()),
            last_notified: Some(Instant::now()),
        };

        process_break_reminder(&config, &posture::PostureStatus::Score(6), &mut state);

        assert!(state.present_since.is_none());
        assert!(state.away_since.is_none());
        assert!(state.last_notified.is_none());
    }

    #[test]
    fn no_person_for_long_enough_resets_timer() {
        let config = Config {
            break_reset_after_mins: 1,
            ..Config::default()
        };
        let mut state = BreakReminderState {
            present_since: Some(Instant::now() - Duration::from_secs(120)),
            away_since: Some(Instant::now() - Duration::from_secs(61)),
            last_notified: Some(Instant::now() - Duration::from_secs(30)),
        };

        process_break_reminder(&config, &posture::PostureStatus::NoPerson, &mut state);

        assert!(state.present_since.is_none());
        assert!(state.away_since.is_none());
        assert!(state.last_notified.is_none());
    }
}
