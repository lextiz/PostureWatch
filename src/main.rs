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
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

const SESSION_RESET_GAP: Duration = Duration::from_secs(60);

#[derive(Default)]
struct ScreenTimeState {
    day_key: u64,
    day_screen_time_secs: u64,
    session_screen_time_secs: u64,
    away_since: Option<Instant>,
    last_presence_tick: Option<Instant>,
    last_session_limit_notification: Option<Instant>,
    last_day_limit_notification: Option<Instant>,
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
    let mut screen_time_state = ScreenTimeState::default();
    let mut was_monitoring_enabled = true;

    tray::TrayManager::setup_tray(config_arc);

    if let Err(e) = analyzer.validate_api_access(&config).await {
        let config_path = Config::config_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "your PostureWatch config.toml".to_string());
        alert::notify_api_setup_needed(&config_path, &e.to_string());
    }

    while APP_RUNNING.load(Ordering::SeqCst) {
        if !MONITORING_ENABLED.load(Ordering::SeqCst) {
            if was_monitoring_enabled {
                camera_state.shutdown();
                screen_time_state.last_presence_tick = None;
                screen_time_state.away_since = None;
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
                    process_screen_time(&current_config, &status, &mut screen_time_state);
                    tray::set_screen_time(
                        screen_time_state.day_screen_time_secs,
                        screen_time_state.session_screen_time_secs,
                    );
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

        if !current_config.keep_camera_on {
            camera_state.shutdown();
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

fn process_screen_time(
    config: &Config,
    status: &posture::PostureStatus,
    state: &mut ScreenTimeState,
) {
    let now = Instant::now();
    let day_key = current_day_key_utc();
    if state.day_key != day_key {
        state.day_key = day_key;
        state.day_screen_time_secs = 0;
        state.last_day_limit_notification = None;
    }

    let session_limit = Duration::from_secs(config.max_session_screen_time_mins.max(1) * 60);
    let day_limit = Duration::from_secs(config.max_daily_screen_time_mins.max(1) * 60);
    let repeat_every = Duration::from_secs(config.break_reminder_repeat_secs.max(1));

    match status {
        posture::PostureStatus::Score(_) => {
            state.away_since = None;
            let elapsed = state
                .last_presence_tick
                .map(|last| now.duration_since(last).as_secs())
                .unwrap_or(0);
            state.last_presence_tick = Some(now);
            state.session_screen_time_secs = state.session_screen_time_secs.saturating_add(elapsed);
            state.day_screen_time_secs = state.day_screen_time_secs.saturating_add(elapsed);

            if !config.break_reminder_enabled {
                return;
            }

            if Duration::from_secs(state.session_screen_time_secs) >= session_limit
                && should_notify_limit(state.last_session_limit_notification, now, repeat_every)
            {
                alert::notify_session_screen_time_limit();
                state.last_session_limit_notification = Some(now);
            }

            if Duration::from_secs(state.day_screen_time_secs) >= day_limit
                && should_notify_limit(state.last_day_limit_notification, now, repeat_every)
            {
                alert::notify_daily_screen_time_limit();
                state.last_day_limit_notification = Some(now);
            }
        }
        posture::PostureStatus::NoPerson => {
            state.last_presence_tick = None;
            if state.away_since.is_none() {
                state.away_since = Some(now);
            }

            if let Some(away_since) = state.away_since {
                if now.duration_since(away_since) >= SESSION_RESET_GAP {
                    state.session_screen_time_secs = 0;
                    state.last_session_limit_notification = None;
                }
            }
        }
    }
}

fn should_notify_limit(
    last_notified: Option<Instant>,
    now: Instant,
    repeat_every: Duration,
) -> bool {
    match last_notified {
        Some(last) => now.duration_since(last) >= repeat_every,
        None => true,
    }
}

fn current_day_key_utc() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0)
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
    fn no_person_for_long_enough_resets_session_timer() {
        let config = Config {
            max_session_screen_time_mins: 1,
            ..Config::default()
        };
        let mut state = ScreenTimeState {
            day_key: current_day_key_utc(),
            day_screen_time_secs: 90,
            session_screen_time_secs: 90,
            away_since: Some(Instant::now()),
            last_presence_tick: Some(Instant::now()),
            last_session_limit_notification: Some(Instant::now()),
            last_day_limit_notification: Some(Instant::now()),
        };

        state.away_since = Some(Instant::now() - Duration::from_secs(61));
        process_screen_time(&config, &posture::PostureStatus::NoPerson, &mut state);

        assert_eq!(state.session_screen_time_secs, 0);
        assert!(state.last_session_limit_notification.is_none());
    }

    #[test]
    fn day_rollover_resets_day_counter() {
        let config = Config::default();
        let mut state = ScreenTimeState {
            day_key: current_day_key_utc().saturating_sub(1),
            day_screen_time_secs: 120,
            session_screen_time_secs: 90,
            away_since: None,
            last_presence_tick: None,
            last_session_limit_notification: None,
            last_day_limit_notification: Some(Instant::now()),
        };

        process_screen_time(&config, &posture::PostureStatus::NoPerson, &mut state);
        assert_eq!(state.day_screen_time_secs, 0);
        assert!(state.last_day_limit_notification.is_none());
    }
}
