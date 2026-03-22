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

#[cfg(feature = "system-tray")]
use tao::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg(feature = "system-tray")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "system-tray")]
static PAUSED: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    // Run with explicit error handling to prevent silent crashes
    if let Err(e) = run_app().await {
        eprintln!("Fatal error: {:?}", e);
    }
}

async fn run_app() -> anyhow::Result<()> {
    let mut config = Config::load();
    
    // Prompt for API key if not set
    config.prompt_for_api_key();
    
    let config = Arc::new(config);
    
    #[cfg(feature = "system-tray")]
    {
        run_with_tray(config).await?;
    }
    
    #[cfg(not(feature = "system-tray"))]
    {
        run_console(config).await?;
    }
    
    Ok(())
}

#[cfg(feature = "system-tray")]
async fn run_with_tray(config: Arc<Config>) -> anyhow::Result<()> {
    use tao::menu::{MenuBuilder, MenuItemBuilder};
    
    let app = tao::AppBuilder::new()
        .build(&tao::AppHandle::default())
        .map_err(|e| anyhow::anyhow!("Failed to build app: {}", e))?;
    
    let handle = app.handle();
    
    // Build tray menu
    let quit = MenuItemBuilder::with_id("quit", "Exit").build(&handle)?;
    let pause = MenuItemBuilder::with_id("pause", "Pause Monitoring").build(&handle)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings...").build(&handle)?;
    
    let menu = MenuBuilder::new(&handle)
        .item(&settings)
        .item(&pause)
        .separator()
        .item(&quit)
        .build()?;
    
    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("PostureWatch - Monitoring Active")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "pause" => {
                    let current = PAUSED.load(Ordering::SeqCst);
                    PAUSED.store(!current, Ordering::SeqCst);
                }
                "settings" => {
                    println!("Settings - config path: {:?}", Config::config_path());
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(&handle)?;
    
    // Run the monitoring loop
    run_monitoring_loop(config, handle).await;
    
    Ok(())
}

#[cfg(feature = "system-tray")]
async fn run_monitoring_loop(config: Arc<Config>, _handle: tao::AppHandle) {
    let mut camera_state = CameraState::new();
    let analyzer = PostureAnalyzer::new((*config).clone());
    
    let strictness = Strictness::from_str(&config.strictness);
    let mut monitor = MonitorLogic::new(strictness);
    
    println!("PostureWatch active in system tray.");
    println!("Cycle time: {} seconds", config.cycle_time_secs);
    println!("Strictness: {}", config.strictness);
    
    // Warmup
    for i in 1..=3 {
        let _ = camera_state.capture_frame();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    let mut last_desk_raise = Instant::now();
    
    loop {
        if PAUSED.load(Ordering::SeqCst) {
            sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        // Desk raise reminder
        if last_desk_raise.elapsed().as_secs() >= config.desk_raise_interval_secs {
            notify_desk_raise(&config);
            last_desk_raise = Instant::now();
        }
        
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
                                next_sleep = 10;
                            }
                            AlertEvent::PostureImproved => {}
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

#[cfg(not(feature = "system-tray"))]
async fn run_console(config: Arc<Config>) -> anyhow::Result<()> {
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
