mod alert;
mod camera;
mod config;
mod posture;
mod posture_monitor;

use crate::alert::{notify_bad_posture, notify_desk_raise};
use crate::camera::CameraState;
use crate::config::Config;
use crate::posture::PostureAnalyzer;
use crate::posture_monitor::{AlertEvent, MonitorLogic, Strictness};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Fatal error: {:?}", e);
    }
}

async fn run_app() -> anyhow::Result<()> {
    let mut config = Config::load();
    
    // Prompt for API key if not set
    config.prompt_for_api_key();
    
    let config = Arc::new(config);
    let mut camera_state = CameraState::new();
    let analyzer = PostureAnalyzer::new((*config).clone());
    
    let strictness = Strictness::from_str(&config.strictness);
    let mut monitor = MonitorLogic::new(strictness);
    
    println!("PostureWatch active. Settings loaded.");
    println!("Cycle time: {} seconds", config.cycle_time_secs);
    println!("Strictness: {}", config.strictness);
    println!("Camera warming up...");
    
    // Warmup
    for i in 1..=3 {
        println!("Warmup capture {}...", i);
        match camera_state.capture_frame() {
            Ok(_) => println!("  Warmup frame {} OK", i),
            Err(e) => eprintln!("  Warmup frame {} error: {:?}", i, e),
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    println!("Warmup complete. Starting monitoring...");
    
    let mut last_desk_raise = Instant::now();
    
    loop {
        // Desk raise reminder
        if last_desk_raise.elapsed().as_secs() >= config.desk_raise_interval_secs {
            notify_desk_raise(&config);
            last_desk_raise = Instant::now();
        }
        
        println!("Capturing frame...");
        
        let mut next_sleep = config.cycle_time_secs;
        
        match camera_state.capture_frame() {
            Ok(frame) => {
                match analyzer.analyze(&frame).await {
                    Ok(status) => {
                        match monitor.process_status(status) {
                            AlertEvent::NotifyBadPosture => {
                                notify_bad_posture(&config);
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
                        monitor = MonitorLogic::new(strictness);
                    }
                }
            }
            Err(e) => {
                eprintln!("Camera error: {:?}", e);
                next_sleep = 10;
            }
        }
        
        sleep(Duration::from_secs(next_sleep)).await;
    }
}
