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
use tower_http::cors::{Any, CorsLayer};

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

#[derive(Clone)] struct AppState { token: String }
#[derive(Serialize, Deserialize)] struct AuthConfig { token: String }

// --- API Payloads ---
#[derive(Deserialize)] struct ThemePayload { name: String }
#[derive(Deserialize)] struct MediaPayload { action: String }
#[derive(Deserialize)] struct AudioPayload { action: String, value: Option<u32> }
#[derive(Deserialize)] struct BrightnessPayload { action: String, value: Option<u32> }
#[derive(Deserialize)] struct SystemPayload { action: String }
#[derive(Deserialize)] struct WorkspacePayload { id: i32 }

// --- State Response ---
#[derive(Serialize)]
struct SystemState {
    volume: u32,
    is_muted: bool,
    brightness: u32,
    workspaces: Vec<i32>,
    active_workspace: i32,
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
            let state = AppState { token };

            // Allow GET for our new state polling
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
async fn handle_get_state() -> impl IntoResponse {
    match get_system_state() {
        Ok(state) => (StatusCode::OK, Json(state)).into_response(),
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

// --- Core Functions ---
fn get_system_state() -> Result<SystemState> {
    // 1. Audio
    let audio_out = String::from_utf8(Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output()?.stdout)?;
    let is_muted = audio_out.contains("[MUTED]");
    let vol_str = audio_out.replace("Volume: ", "").replace("[MUTED]", "").trim().to_string();
    let volume = (vol_str.parse::<f32>().unwrap_or(0.0) * 100.0) as u32;

    // 2. Brightness
    let bright_out = String::from_utf8(Command::new("brightnessctl").arg("-m").output()?.stdout)?;
    let parts: Vec<&str> = bright_out.split(',').collect();
    let brightness = parts.get(3).unwrap_or(&"0%").replace("%", "").trim().parse::<u32>().unwrap_or(0);

    // 3. Workspaces
    let ws_out = String::from_utf8(Command::new("hyprctl").args(["workspaces", "-j"]).output()?.stdout)?;
    let parsed_ws: serde_json::Value = serde_json::from_str(&ws_out)?;
    let mut workspaces = Vec::new();
    if let Some(arr) = parsed_ws.as_array() {
        for w in arr {
            // FIX: Using .get() instead of bracket indexing
            if let Some(id) = w.get("id").and_then(|v| v.as_i64()) { workspaces.push(id as i32); }
        }
    }
    workspaces.sort();

    // 4. Active Workspace
    let active_ws_out = String::from_utf8(Command::new("hyprctl").args(["activeworkspace", "-j"]).output()?.stdout)?;
    let parsed_active: serde_json::Value = serde_json::from_str(&active_ws_out)?;
    
    // FIX: Using .get() instead of bracket indexing
    let active_workspace = parsed_active.get("id").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

    Ok(SystemState { volume, is_muted, brightness, workspaces, active_workspace })
}

fn switch_theme(theme_name: &str) -> Result<()> {
    let script_path = format!("{}/.config/hypr/scripts/switch_theme.sh", std::env::var("HOME")?);
    Command::new(&script_path).arg(theme_name).status()?; Ok(())
}
fn control_media(action: &str) -> Result<()> { Command::new("playerctl").arg(action).status()?; Ok(()) }
fn control_system(action: &str) -> Result<()> { if action == "lock" { Command::new("hyprlock").status()?; } Ok(()) }
fn control_workspace(id: i32) -> Result<()> {
    // The exact, official Hyprland 0.55+ Lua API syntax
    // Note the double braces {{ }} which Rust requires to print a literal { }
    let lua_dispatcher = format!("hl.dsp.focus({{ workspace = \"{}\" }})", id);
    
    let status = Command::new("hyprctl")
        .args(["dispatch", &lua_dispatcher])
        .status()?;
        
    if !status.success() { 
        anyhow::bail!("Hyprland workspace switch failed."); 
    }
    
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

fn control_brightness(action: &str, value: Option<u32>) -> Result<()> {
    if action == "set" {
        if let Some(v) = value {
            let b_str = format!("{}%", v);
            Command::new("brightnessctl").args(["set", &b_str]).status()?;
        }
    }
    Ok(())
}