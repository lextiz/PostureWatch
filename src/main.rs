mod alert;
mod camera;
mod config;
mod gui;
mod posture;
mod posture_monitor;

use config::Config;
use posture::PostureAnalyzer;
use posture_monitor::{AlertEvent, MonitorLogic, Strictness};

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

// App state
pub struct AppState {
    config: Config,
    camera_state: TokioMutex<camera::CameraState>,
    analyzer: PostureAnalyzer,
    monitor: TokioMutex<MonitorLogic>,
    is_paused: bool,
    last_desk_raise: Instant,
}

impl AppState {
    pub fn new() -> Self {
        let mut config = Config::load();
        config.prompt_for_api_key();

        let strictness = Strictness::from_str(&config.strictness);

        Self {
            config,
            camera_state: TokioMutex::new(camera::CameraState::new()),
            analyzer: PostureAnalyzer::new(Config::load()),
            monitor: TokioMutex::new(MonitorLogic::new(strictness)),
            is_paused: false,
            last_desk_raise: Instant::now(),
        }
    }

    pub fn toggle_pause(&mut self) -> bool {
        self.is_paused = !self.is_paused;
        self.is_paused
    }

    pub fn update_config(&mut self, new_config: Config) {
        self.config = new_config;
        let strictness = Strictness::from_str(&self.config.strictness);
        self.monitor = TokioMutex::new(MonitorLogic::new(strictness));
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

#[tokio::main]
async fn main() {
    // Initialize GUI (system tray, etc.)
    gui::init_system_tray();

    // Run app in background thread
    let app_state = Arc::new(TokioMutex::new(AppState::new()));

    // Spawn the monitoring task
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        run_monitor(state_clone).await;
    });

    // Keep main thread alive - would run GUI event loop here
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn run_monitor(state: Arc<TokioMutex<AppState>>) {
    println!("PostureWatch started in background mode");
    println!("Configure via tray icon or config file");

    // Initial config
    {
        let guard = state.lock().await;
        println!("Cycle time: {} seconds", guard.config.cycle_time_secs);
        println!("Strictness: {}", guard.config.strictness);
    }

    loop {
        // Check if paused
        let should_skip = {
            let guard = state.lock().await;
            if guard.is_paused {
                println!("Paused - waiting...");
                true
            } else {
                false
            }
        };

        if should_skip {
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Check desk raise interval
        let mut should_notify_desk = false;
        {
            let guard = state.lock().await;
            if guard.last_desk_raise.elapsed().as_secs() >= guard.config.desk_raise_interval_secs {
                should_notify_desk = true;
            }
        }

        if should_notify_desk {
            {
                let guard = state.lock().await;
                alert::notify_desk_raise(&guard.config);
            }

            let mut guard = state.lock().await;
            guard.last_desk_raise = Instant::now();
        }

        // Capture and analyze
        let mut next_sleep = 10;
        let mut status_opt = None;

        {
            let guard = state.lock().await;
            let mut camera_guard = guard.camera_state.lock().await;
            match camera_guard.capture_frame() {
                Ok(frame) => match guard.analyzer.analyze(&frame).await {
                    Ok(status) => {
                        status_opt = Some(status);
                    }
                    Err(e) => {
                        eprintln!("Analysis error: {:?}", e);
                        let mut monitor = guard.monitor.lock().await;
                        *monitor =
                            MonitorLogic::new(Strictness::from_str(&guard.config.strictness));
                    }
                },
                Err(e) => {
                    eprintln!("Camera error: {:?}", e);
                    next_sleep = 10;
                }
            }
        }

        if let Some(status) = status_opt {
            let guard = state.lock().await;
            let mut monitor = guard.monitor.lock().await;

            match monitor.process_status(status) {
                AlertEvent::NotifyBadPosture => {
                    alert::notify_bad_posture(&guard.config);
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

        // Get cycle time from config
        {
            let guard = state.lock().await;
            next_sleep = guard.config.cycle_time_secs;
        }

        sleep(Duration::from_secs(next_sleep)).await;
    }
}
