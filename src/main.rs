mod alert;
mod camera;
mod config;
mod posture;
mod posture_monitor;
mod tray;

use config::Config;
use posture::PostureAnalyzer;
use posture_monitor::{AlertEvent, MonitorLogic, Strictness};

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Load configuration
    let mut config = Config::load();
    config.prompt_for_api_key();
    
    let strictness = Strictness::from_str(&config.strictness);
    
    // Initialize components
    let camera_state = TokioMutex::new(camera::CameraState::new());
    let config_arc = Arc::new(TokioMutex::new(config.clone()));
    let analyzer = PostureAnalyzer::new(Config::load());
    let monitor = TokioMutex::new(MonitorLogic::new(strictness));
    let mut last_desk_raise = Instant::now();
    
    println!("PostureWatch started");
    println!("Cycle time: {} seconds", config.cycle_time_secs);
    println!("Strictness: {}", config.strictness);

    // Setup system tray (on Windows) or web UI (on other platforms)
    tray::TrayManager::setup_tray(config_arc.clone());

    loop {
        // Check desk raise interval
        if last_desk_raise.elapsed().as_secs() >= config.desk_raise_interval_secs {
            alert::notify_desk_raise(&config);
            last_desk_raise = Instant::now();
        }

        // Capture and analyze
        let mut next_sleep = 10;
        
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
                                    next_sleep = 10;
                                }
                                AlertEvent::FirstWarning => {
                                    println!("Warning: Posture degraded.");
                                    next_sleep = 10;
                                }
                                AlertEvent::PostureImproved => {
                                    println!("Posture improved. Good job!");
                                }
                                AlertEvent::None => {}
                            }
                        }
                        Err(e) => {
                            eprintln!("Analysis error: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Camera error: {:?}", e);
                    next_sleep = 10;
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
        
        config = new_config;
        next_sleep = config.cycle_time_secs;

        sleep(Duration::from_secs(next_sleep)).await;
    }
}