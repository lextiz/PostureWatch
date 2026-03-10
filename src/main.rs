mod config;
mod camera;
mod posture;
mod alert;
mod posture_monitor;

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::time::Instant;
use crate::config::Config;
use crate::camera::CameraState;
use crate::posture::PostureAnalyzer;
use crate::alert::{notify_bad_posture, notify_desk_raise};
use crate::posture_monitor::{MonitorLogic, AlertEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::load());
    let mut camera_state = CameraState::new();
    let analyzer = PostureAnalyzer::new(config.clone().as_ref().clone());
    
    println!("PostureWatch active. Settings loaded.");
    if config.privacy_mode {
        println!("Privacy mode ON: Images are analyzed locally when possible.");
    }
    
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
                            AlertEvent::Unknown => {
                                println!("Fallback or unknown status. Please ensure good posture manually.");
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
