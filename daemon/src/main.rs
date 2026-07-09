use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode, Method},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use rand::{distr::Alphanumeric, RngExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Theme { name: String },
    Media { action: String },
    Serve { #[arg(short, long, default_value_t = 4000)] port: u16 },
    Pair,
}

// --- NEW: Added Notifications Array to AppState ---
#[derive(Clone)] 
struct AppState { 
    token: String,
    notifications: Arc<Mutex<Vec<Notification>>>,
}
#[derive(Serialize, Deserialize)] struct AuthConfig { token: String }

#[derive(Deserialize)] struct ThemePayload { name: String }
#[derive(Deserialize)] struct MediaPayload { action: String }
#[derive(Deserialize)] struct AudioPayload { action: String, value: Option<u32> }
#[derive(Deserialize)] struct BrightnessPayload { action: String, value: Option<u32> }
#[derive(Deserialize)] struct SystemPayload { action: String }
#[derive(Deserialize)] struct WorkspacePayload { id: i32 }
#[derive(Deserialize)] struct NotificationPayload { action: String, id: Option<String> }

// --- State Response ---
#[derive(Serialize, Clone)]
struct Notification {
    id: String,
    app_name: String,
    summary: String,
    body: String,
}

#[derive(Serialize)]
struct SystemState {
    volume: u32,
    is_muted: bool,
    brightness: u32,
    workspaces: Vec<i32>,
    active_workspace: i32,
    battery: u8,
    wifi_ssid: String,
    bluetooth_on: bool,
    bt_device: String,
    notifications: Vec<Notification>, // Sent to React
}

// --- NEW CONFIGURATION SCHEMA ---
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ModuleConfig {
    Telemetry {},      // <-- Added {}
    Notifications {},  // <-- Added {}
    Workspaces {},     // <-- Added {}
    MediaSystem {},    // <-- Added {}
    Slider { endpoint: String, label: String, color: String },
}

#[derive(Serialize, Deserialize, Clone)]
struct GeneralConfig {
    theme: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    general: GeneralConfig,
    layout: Vec<ModuleConfig>,
}

// Read or generate the default config.toml
fn get_or_create_config() -> Result<AppConfig> {
    let mut path = dirs::config_dir().context("Could not find config dir")?;
    path.push("command-center");
    fs::create_dir_all(&path)?;
    path.push("config.toml");

    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        // Generate the default Arch Linux layout
        let default_config = AppConfig {
            general: GeneralConfig { theme: "glass".to_string() },
            layout: vec![
                ModuleConfig::Telemetry {},
                ModuleConfig::Notifications {},
                ModuleConfig::Workspaces {},
                ModuleConfig::Slider { 
                    endpoint: "audio".to_string(), 
                    label: "Master Volume".to_string(), 
                    color: "bg-white".to_string() 
                },
                ModuleConfig::Slider { 
                    endpoint: "brightness".to_string(), 
                    label: "Display Brightness".to_string(), 
                    color: "bg-yellow-400".to_string() 
                },
                ModuleConfig::MediaSystem {},
            ],
        };
        let toml_string = toml::to_string(&default_config)?;
        fs::write(&path, toml_string)?;
        Ok(default_config)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Theme { name } => { switch_theme(name)?; }
        Commands::Media { action } => { control_media(action)?; }
        Commands::Pair => {
            let token = get_current_token().context("⚠️ Daemon is not running!")?;
            let my_local_ip = local_ip_address::local_ip().context("Failed to get local IP.")?;
            let app_url = format!("http://{}:3000/?token={}", my_local_ip, token);
            println!("\n📱 Scan this QR code to instantly connect:\n");
            qr2term::print_qr(&app_url)?;
            println!("\n🔗 Fallback URL: {}\n", app_url);
        }
        Commands::Serve { port } => {
            let token = generate_new_token()?;
            
            // --- THE DBUS EAVESDROPPER ENGINE ---
            let notifications = Arc::new(Mutex::new(Vec::new()));
            let notifs_clone = notifications.clone();
            
            tokio::spawn(async move {
                let mut child = tokio::process::Command::new("dbus-monitor")
                    .arg("interface='org.freedesktop.Notifications',member='Notify'")
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .expect("Failed to start dbus-monitor");

                let stdout = child.stdout.take().unwrap();
                let mut reader = BufReader::new(stdout).lines();

                let mut current_app = String::new();
                let mut current_summary = String::new();
                let mut current_body = String::new();
                let mut string_count = 0;

                while let Ok(Some(line)) = reader.next_line().await {
                    let line = line.trim();
                    if line.starts_with("method call") {
                        string_count = 0;
                        current_app.clear();
                        current_summary.clear();
                        current_body.clear();
                    } else if line.starts_with("string \"") {
                        string_count += 1;
                        
                        let content = if let (Some(start), Some(end)) = (line.find('"'), line.rfind('"')) {
                            if start != end {
                                // Strip HTML formatting some apps send, and format newlines
                                line[start + 1..end].replace("\\n", "\n")
                                    .replace("<b>", "").replace("</b>", "")
                                    .replace("<i>", "").replace("</i>", "")
                            } else { String::new() }
                        } else { String::new() };

                        match string_count {
                            1 => current_app = content,
                            3 => current_summary = content,
                            4 => {
                                current_body = content;
                                let id: String = rand::rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect();
                                let mut hist = notifs_clone.lock().unwrap();
                                // Add to top of feed
                                hist.insert(0, Notification {
                                    id,
                                    app_name: current_app.clone(),
                                    summary: current_summary.clone(),
                                    body: current_body.clone(),
                                });
                                // Keep memory clean, only store last 20
                                if hist.len() > 20 { hist.pop(); }
                            },
                            _ => {}
                        }
                    }
                }
            });

            let state = AppState { token, notifications };

            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_headers(Any);

            let app = Router::new()
                .route("/api/state", get(handle_get_state))
                .route("/api/theme", post(handle_theme))
                .route("/api/media", post(handle_media))
                .route("/api/audio", post(handle_audio))
                .route("/api/brightness", post(handle_brightness))
                .route("/api/system", post(handle_system))
                .route("/api/workspace", post(handle_workspace))
                .route("/api/bluetooth", post(handle_bluetooth))
                .route("/api/notifications", post(handle_notifications)) // NEW ROUTE
                .route("/api/layout", get(handle_get_layout))
                .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .layer(cors)
                .with_state(state);

            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            println!("🚀 Command Center Daemon running on http://{}", addr);
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

// --- Security ---
fn generate_new_token() -> Result<String> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("command-center");
    fs::create_dir_all(&path)?;
    path.push("auth.json");
    let new_token: String = rand::rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();
    let auth = AuthConfig { token: new_token.clone() };
    fs::write(&path, serde_json::to_string_pretty(&auth)?)?;
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&path, perms)?;
    Ok(new_token)
}

fn get_current_token() -> Result<String> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("command-center");
    path.push("auth.json");
    if path.exists() {
        let auth: AuthConfig = serde_json::from_str(&fs::read_to_string(&path)?)?;
        Ok(auth.token)
    } else { anyhow::bail!("auth.json not found."); }
}

async fn auth_middleware(State(state): State<AppState>, req: Request, next: Next) -> Result<Response, StatusCode> {
    let expected_header = format!("Bearer {}", state.token);
    match req.headers().get(header::AUTHORIZATION) {
        Some(value) if value.as_bytes() == expected_header.as_bytes() => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED)
    }
}

// --- Route Handlers ---
async fn handle_get_state(State(state): State<AppState>) -> impl IntoResponse {
    let notifs = state.notifications.lock().unwrap().clone();
    match get_system_state(notifs) {
        Ok(sys_state) => (StatusCode::OK, Json(sys_state)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read system state").into_response(),
    }
}
async fn handle_theme(Json(payload): Json<ThemePayload>) -> impl IntoResponse {
    match switch_theme(&payload.name) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_media(Json(payload): Json<MediaPayload>) -> impl IntoResponse {
    match control_media(&payload.action) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_audio(Json(payload): Json<AudioPayload>) -> impl IntoResponse {
    match control_audio(&payload.action, payload.value) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_brightness(Json(payload): Json<BrightnessPayload>) -> impl IntoResponse {
    match control_brightness(&payload.action, payload.value) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_system(Json(payload): Json<SystemPayload>) -> impl IntoResponse {
    match control_system(&payload.action) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_workspace(Json(payload): Json<WorkspacePayload>) -> impl IntoResponse {
    match control_workspace(payload.id) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_bluetooth(Json(payload): Json<SystemPayload>) -> impl IntoResponse {
    match control_bluetooth(&payload.action) {
        Ok(_) => StatusCode::OK.into_response(), Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn handle_notifications(State(state): State<AppState>, Json(payload): Json<NotificationPayload>) -> impl IntoResponse {
    let mut notifs = state.notifications.lock().unwrap();
    if payload.action == "clear" {
        notifs.clear();
        // Cross-talk: Also clear SwayNC on the actual laptop!
        let _ = Command::new("swaync-client").arg("-C").status();
    } else if payload.action == "clear_one" {
        if let Some(id) = payload.id {
            notifs.retain(|n| n.id != id);
        }
    }
    StatusCode::OK.into_response()
}

async fn handle_get_layout() -> impl IntoResponse {
    match get_or_create_config() {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            eprintln!("💥 Config Error: {:?}", e); // Logs to terminal!
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Config error: {}", e)).into_response()
        }
    }
}

// --- Core Functions ---
fn get_system_state(notifications: Vec<Notification>) -> Result<SystemState> {
    let audio_out = String::from_utf8(Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output()?.stdout)?;
    let is_muted = audio_out.contains("[MUTED]");
    let vol_str = audio_out.replace("Volume: ", "").replace("[MUTED]", "").trim().to_string();
    let volume = (vol_str.parse::<f32>().unwrap_or(0.0) * 100.0) as u32;

    let bright_out = String::from_utf8(Command::new("brightnessctl").arg("-m").output()?.stdout)?;
    let parts: Vec<&str> = bright_out.split(',').collect();
    let brightness = parts.get(3).unwrap_or(&"0%").replace("%", "").trim().parse::<u32>().unwrap_or(0);

    let ws_out = String::from_utf8(Command::new("hyprctl").args(["workspaces", "-j"]).output()?.stdout)?;
    let parsed_ws: serde_json::Value = serde_json::from_str(&ws_out)?;
    let mut workspaces = Vec::new();
    if let Some(arr) = parsed_ws.as_array() {
        for w in arr {
            if let Some(id) = w.get("id").and_then(|v| v.as_i64()) { workspaces.push(id as i32); }
        }
    }
    workspaces.sort();

    let active_ws_out = String::from_utf8(Command::new("hyprctl").args(["activeworkspace", "-j"]).output()?.stdout)?;
    let parsed_active: serde_json::Value = serde_json::from_str(&active_ws_out)?;
    let active_workspace = parsed_active.get("id").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

    let (battery, wifi_ssid, bluetooth_on, bt_device) = get_telemetry();

    Ok(SystemState { 
        volume, is_muted, brightness, workspaces, active_workspace,
        battery, wifi_ssid, bluetooth_on, bt_device,
        notifications
    })
}

fn switch_theme(theme_name: &str) -> Result<()> {
    let script_path = format!("{}/.config/hypr/scripts/switch_theme.sh", std::env::var("HOME")?);
    Command::new(&script_path).arg(theme_name).status()?; Ok(())
}
fn control_media(action: &str) -> Result<()> { Command::new("playerctl").arg(action).status()?; Ok(()) }
fn control_system(action: &str) -> Result<()> { if action == "lock" { Command::new("hyprlock").status()?; } Ok(()) }
fn control_workspace(id: i32) -> Result<()> {
    let lua_dispatcher = format!("hl.dsp.focus({{ workspace = \"{}\" }})", id);
    let status = Command::new("hyprctl").args(["dispatch", &lua_dispatcher]).status()?;
    if !status.success() { anyhow::bail!("Hyprland workspace switch failed."); }
    Ok(())
}

fn control_audio(action: &str, value: Option<u32>) -> Result<()> {
    match action {
        "set" => {
            if let Some(v) = value {
                let vol_str = format!("{}%", v);
                Command::new("wpctl").args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str]).status()?;
            }
        },
        "mute" => { Command::new("wpctl").args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).status()?; },
        _ => {}
    }
    Ok(())
}

fn control_bluetooth(action: &str) -> Result<()> {
    if action == "toggle" {
        let output = Command::new("bluetoothctl").arg("show").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let powered = stdout.lines().find_map(|l| {
            let l = l.trim();
            if l.starts_with("Powered:") {
                Some(l.split(':').nth(1).unwrap_or("").trim().to_lowercase())
            } else { None }
        }).unwrap_or_else(|| "no".to_string());

        let turn_on = match powered.as_str() { "yes" | "true" => false, _ => true };
        let arg = if turn_on { "on" } else { "off" };
        Command::new("bluetoothctl").arg("power").arg(arg).status()?;
    }
    Ok(())
}

fn control_brightness(action: &str, value: Option<u32>) -> Result<()> {
    if action == "set" {
        if let Some(v) = value {
            let b_str = format!("{}%", v);
            Command::new("brightnessctl").args(["set", &b_str]).status()?;
        }
    }
    Ok(())
}

fn get_telemetry() -> (u8, String, bool, String) {
    let battery = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .unwrap_or_else(|_| "100".to_string())
        .trim().parse::<u8>().unwrap_or(100);

    let wifi_cmd = Command::new("sh").arg("-c").arg("nmcli -t -f active,ssid dev wifi | grep '^yes' | cut -d':' -f2").output();
    let wifi_ssid = match wifi_cmd {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() { "Disconnected".to_string() } else { s }
        },
        Err(_) => "Unknown".to_string(),
    };

   let bt_info = Command::new("bluetoothctl").arg("show").output();
    let bluetooth_on = match bt_info {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains("Powered: yes"),
        Err(_) => false,
    };

    let bt_devices = Command::new("sh").arg("-c").arg("bluetoothctl devices Connected | cut -d' ' -f3-").output();
    let bt_device = match bt_devices {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() { "None".to_string() } else { s }
        },
        Err(_) => "None".to_string(),
    };

    (battery, wifi_ssid, bluetooth_on, bt_device)
}