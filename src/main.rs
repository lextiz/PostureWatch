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
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    logging::init();
    log_info!("PostureWatch starting");

    let config = Config::load();

    let camera_state = TokioMutex::new(camera::CameraState::new());
    let config_arc = Arc::new(TokioMutex::new(config.clone()));
    let analyzer = PostureAnalyzer::new();
    let monitor = TokioMutex::new(MonitorLogic::new(
        config.posture_threshold,
        config.alert_threshold,
    ));
    let mut last_desk_raise = Instant::now();
    let mut was_monitoring_enabled = true;

    tray::TrayManager::setup_tray(config_arc);

    while APP_RUNNING.load(Ordering::SeqCst) {
        if !MONITORING_ENABLED.load(Ordering::SeqCst) {
            if was_monitoring_enabled {
                let mut camera_guard = camera_state.lock().await;
                camera_guard.shutdown();
            }
            was_monitoring_enabled = false;
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        was_monitoring_enabled = true;

        let current_config = Config::load();

        if current_config.desk_raise_enabled {
            let interval_secs = current_config.desk_raise_interval_mins * 60;
            if last_desk_raise.elapsed().as_secs() >= interval_secs {
                alert::notify_desk_raise();
                last_desk_raise = Instant::now();
            }
        }

        let mut camera_guard = camera_state.lock().await;
        if let Ok(frame) = camera_guard.capture_frame() {
            drop(camera_guard);
            match analyzer.analyze(&frame, &current_config).await {
                Ok(status) => {
                    let mut monitor_guard = monitor.lock().await;
                    if let AlertEvent::NotifyBadPosture = monitor_guard.process_status(status) {
                        alert::notify_bad_posture();
                    }
                    monitor_guard.set_thresholds(
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

    let mut camera_guard = camera_state.lock().await;
    camera_guard.shutdown();

    log_info!("PostureWatch exiting");
}
