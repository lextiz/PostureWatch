mod alert;
mod camera;
mod config;
mod posture;
mod posture_monitor;
mod tray;

use config::Config;
use posture::PostureAnalyzer;
use posture_monitor::{AlertEvent, MonitorLogic, Strictness};
use tray::{APP_RUNNING, MONITORING_ENABLED};

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Load configuration - check for API key without prompting (GUI will handle it)
    let config = Config::load();

    let strictness = Strictness::from_str(&config.strictness);

    // Initialize components
    let camera_state = TokioMutex::new(camera::CameraState::new());
    let config_arc = Arc::new(TokioMutex::new(config.clone()));
    let analyzer = PostureAnalyzer::new(Config::load());
    let monitor = TokioMutex::new(MonitorLogic::new(strictness));
    let mut last_desk_raise = Instant::now();

    // Setup system tray with Slint GUI
    tray::TrayManager::setup_tray(config_arc.clone());

    // Main monitoring loop
    while APP_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
        // Check if monitoring is enabled
        if !MONITORING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        // Check desk raise interval
        if last_desk_raise.elapsed().as_secs() >= config.desk_raise_interval_secs {
            alert::notify_desk_raise(&config);
            last_desk_raise = Instant::now();
        }

        // Capture and analyze

        {
            let mut camera_guard = camera_state.lock().await;
            match camera_guard.capture_frame() {
                Ok(frame) => {
                    match analyzer.analyze(&frame).await {
                        Ok(status) => {
                            let mut monitor_guard = monitor.lock().await;
                            match monitor_guard.process_status(status) {
                                AlertEvent::NotifyBadPosture => {
                                    alert::notify_bad_posture(&config);
                                }
                                AlertEvent::FirstWarning => {
                                    // Silently log warning
                                }
                                AlertEvent::PostureImproved => {
                                    // Silently log improvement
                                }
                                AlertEvent::None => {}
                            }
                        }
                        Err(_e) => {
                            // Silently handle analysis errors
                        }
                    }
                }
                Err(_e) => {
                    // Silently handle camera errors
                }
            }
        }

        // Reload config to check for changes (tray menu updates)
        let new_config = Config::load();

        // Update strictness if changed in config
        let new_strictness = Strictness::from_str(&new_config.strictness);
        {
            let mut monitor_guard = monitor.lock().await;
            monitor_guard.set_strictness(new_strictness);
        }

        sleep(Duration::from_secs(new_config.cycle_time_secs)).await;
    }
}
