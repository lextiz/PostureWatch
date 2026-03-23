// System tray and GUI module for PostureWatch

use crate::config::Config;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub struct TrayManager {
    // Tray state
}

impl TrayManager {
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(windows)]
    pub fn setup_tray(config: Arc<TokioMutex<Config>>) {
        use std::thread;
        
        // Spawn tray in a separate thread since it requires blocking API
        let config_clone = config.clone();
        thread::spawn(move || {
            if let Err(e) = Self::run_tray(config_clone) {
                eprintln!("Failed to setup tray: {}", e);
            }
        });
        
        // Also start HTTP server for config UI
        let config_clone2 = config.clone();
        thread::spawn(move || {
            if let Err(e) = Self::start_http_server(config_clone2) {
                eprintln!("HTTP server error: {}", e);
            }
        });
    }

    #[cfg(windows)]
    fn run_tray(config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        // For Windows, we'll use a simple console-based menu for now
        // The HTTP server provides the GUI configuration
        println!("PostureWatch running - open http://localhost:8080 to configure");
        
        // Keep the thread alive to handle system tray events
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
    
    #[cfg(windows)]
    fn start_http_server(config: Arc<TokioMutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        
        let listener = TcpListener::bind("0.0.0.0:8080")?;
        println!("HTTP config server listening on http://localhost:8080");
        
        for stream in listener.incoming() {
            let mut stream = stream?;
            let config_clone = config.clone();
            
            let mut buffer = [0u8; 1024];
            if let Ok(_) = stream.read(&mut buffer) {
                let request = String::from_utf8_lossy(&buffer);
                
                if request.starts_with("GET / ") {
                    // Serve HTML config page
                    let html = Self::get_config_html(&config_clone);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                        html.len(), html
                    );
                    let _ = stream.write_all(response.as_bytes());
                } else if request.starts_with("POST /set_strictness") {
                    // Parse the strictness from request
                    if let Some(pos) = request.find("strictness=") {
                        let strictness_raw = &request[pos + 11..].split_whitespace().next().unwrap_or("Medium");
                        let strictness = match *strictness_raw {
                            "Low" => "Low",
                            "High" => "High", 
                            _ => "Medium",
                        };
                        
                        // Update config
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        runtime.block_on(async {
                            let mut cfg = config_clone.lock().await;
                            cfg.strictness = strictness.to_string();
                            let _ = cfg.save();
                        });
                        
                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nOK";
                        let _ = stream.write_all(response.as_bytes());
                    } else {
                        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
                        let _ = stream.write_all(response.as_bytes());
                    }
                } else {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(windows)]
    fn get_config_html(_config: &Arc<TokioMutex<Config>>) -> String {
        // Read config synchronously from disk
        let current_strictness = Config::load().strictness;
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>PostureWatch Configuration</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        .setting {{ margin: 20px 0; }}
        select {{ padding: 10px; font-size: 16px; }}
        button {{ padding: 10px 20px; font-size: 16px; cursor: pointer; background: #007bff; color: white; border: none; border-radius: 4px; }}
        button:hover {{ background: #0056b3; }}
        .status {{ color: green; margin-top: 10px; }}
    </style>
</head>
<body>
    <h1>PostureWatch Configuration</h1>
    <p>Open http://localhost:8080 to configure PostureWatch</p>
    <form id="configForm">
        <div class="setting">
            <label for="strictness">Strictness Level:</label><br>
            <select id="strictness" name="strictness">
                <option value="Low" {}>Low</option>
                <option value="Medium" {}>Medium</option>
                <option value="High" {}>High</option>
            </select>
        </div>
        <button type="submit">Save</button>
    </form>
    <div id="status" class="status"></div>
    <script>
        document.getElementById('configForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const strictness = document.getElementById('strictness').value;
            const formData = new FormData();
            formData.append('strictness', strictness);
            
            try {{
                const response = await fetch('/set_strictness', {{
                    method: 'POST',
                    body: new URLSearchParams(formData)
                }});
                if (response.ok) {{
                    document.getElementById('status').textContent = 'Settings saved! Reload to see changes.';
                }}
            }} catch (e) {{
                document.getElementById('status').textContent = 'Error saving settings';
            }}
        }});
    </script>
</body>
</html>
"#, 
            if current_strictness == "Low" { "selected" } else { "" },
            if current_strictness == "Medium" { "selected" } else { "" },
            if current_strictness == "High" { "selected" } else { "" }
        )
    }

    #[cfg(not(windows))]
    pub fn setup_tray(_config: Arc<TokioMutex<Config>>) {
        println!("Starting web UI on http://localhost:8080");
        
        std::thread::spawn(|| {
            if let Err(e) = Self::start_http_server() {
                eprintln!("Web server error: {}", e);
            }
        });
    }
    
    #[cfg(not(windows))]
    fn start_http_server() -> Result<(), Box<dyn std::error::Error>> {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        
        let listener = TcpListener::bind("0.0.0.0:8080")?;
        println!("HTTP config server listening on http://localhost:8080");
        
        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut buffer = [0u8; 1024];
            if let Ok(_) = stream.read(&mut buffer) {
                let request = String::from_utf8_lossy(&buffer);
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nPostureWatch Config Server";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        
        Ok(())
    }
}