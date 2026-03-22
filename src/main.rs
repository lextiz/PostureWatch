mod alert;
mod camera;
mod config;
mod posture;
mod posture_monitor;

use crate::alert::{notify_bad_posture, notify_desk_raise};
use crate::camera::CameraState;
use crate::config::Config;
use crate::posture::PostureAnalyzer;
use crate::posture_monitor::{AlertEvent, MonitorLogic};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::load();
    
    // Prompt for API key if not set
    config.prompt_for_api_key();
    
    let config = Arc::new(config);
    let mut camera_state = CameraState::new();
    let analyzer = PostureAnalyzer::new(config.clone().as_ref().clone());

    println!("PostureWatch active. Settings loaded.");

    let mut last_desk_raise = Instant::now();
    let mut monitor = MonitorLogic::new();

    loop {
        // Desk raise reminder
        if last_desk_raise.elapsed().as_secs() >= config.desk_raise_interval_secs {
            notify_desk_raise(&config);
            last_desk_raise = Instant::now();
        }

        let mut next_sleep = config.cycle_time_secs;

        // Capture frame
        match camera_state.capture_frame() {
            Ok(frame) => {
                // Analyze posture
                match analyzer.analyze(&frame).await {
                    Ok(status) => {
                        match monitor.process_status(status) {
                            AlertEvent::NotifyBadPosture => {
                                notify_bad_posture(&config);
                                next_sleep = 10; // Check frequently until posture improves
                            }
                            AlertEvent::FirstWarning => {
                                println!("Warning: Posture degraded.");
                                next_sleep = 10; // Check frequently until posture improves
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
                // Pause longer on camera error
                next_sleep = 10;
            }
        }

        sleep(Duration::from_secs(next_sleep)).await;
    }
}
