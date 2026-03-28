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
    let mut was_monitoring_enabled = true;

    tray::TrayManager::setup_tray(config_arc);

    while APP_RUNNING.load(Ordering::SeqCst) {
        if !MONITORING_ENABLED.load(Ordering::SeqCst) {
            if was_monitoring_enabled {
                camera_state.shutdown();
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
}
