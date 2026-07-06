use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use clap::{Parser, Subcommand};
use rand::{distr::Alphanumeric, RngExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Theme { name: String },
    Media { action: String },
    /// Starts the background API server
    Serve {
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
    /// Generate a QR code to pair your phone
    Pair,
}

// --- Shared State ---
#[derive(Clone)]
struct AppState {
    token: String,
}

// --- Config Structures ---
#[derive(Serialize, Deserialize)]
struct AuthConfig {
    token: String,
}

#[derive(Deserialize)]
struct ThemePayload {
    name: String,
}

#[derive(Deserialize)]
struct MediaPayload {
    action: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Theme { name } => {
            switch_theme(name)?;
            println!("Successfully switched theme.");
        }
        Commands::Media { action } => {
            control_media(action)?;
            println!("Successfully controlled media.");
        }
        Commands::Pair => {
            let token = get_or_create_token()?;
            
            // Automatically grab your laptop's local Wi-Fi IP address
            let my_local_ip = local_ip_address::local_ip()
                .context("Failed to get local IP address. Are you connected to Wi-Fi?")?;
            
            // Format the full URL pointing to your Next.js app (Port 3000)
            let app_url = format!("http://{}:3000/?token={}", my_local_ip, token);
            
            println!("\n📱 Scan this QR code to instantly connect:\n");
            qr2term::print_qr(&app_url)?;
            
            println!("\n🔗 Fallback URL: {}\n", app_url);
            println!("⚠️  Keep this secure. Anyone with this link can control your system.\n");
        }
        Commands::Serve { port } => {
            // Load or generate the token before starting the server
            let token = get_or_create_token()?;
            let state = AppState { token };

            let app = Router::new()
                .route("/api/theme", post(handle_theme))
                .route("/api/media", post(handle_media))
                // Pass our state into the middleware
                .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .with_state(state); // Provide state to the router

            // Bind to all network interfaces (0.0.0.0) so your phone can connect!
            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            
            println!("🚀 Command Center Daemon running on http://{}", addr);
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}

// --- Security & Token Management ---

fn get_or_create_token() -> Result<String> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("command-center");
    fs::create_dir_all(&path)?;
    path.push("auth.json");

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let auth: AuthConfig = serde_json::from_str(&content)?;
        Ok(auth.token)
    } else {
        println!("⚙️  No token found. Generating a secure 256-bit authentication token...");
        let new_token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let auth = AuthConfig { token: new_token.clone() };
        let content = serde_json::to_string_pretty(&auth)?;
        
        fs::write(&path, &content)?;
        
        // SECURITY: Set strict file permissions (rw-------)
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;

        Ok(new_token)
    }
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected_header = format!("Bearer {}", state.token);
    let auth_header = req.headers().get(header::AUTHORIZATION);

    match auth_header {
        Some(value) if value.as_bytes() == expected_header.as_bytes() => Ok(next.run(req).await),
        _ => {
            println!("⚠️ Blocked unauthorized API request.");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// --- Axum Route Handlers ---

async fn handle_theme(Json(payload): Json<ThemePayload>) -> impl IntoResponse {
    match switch_theme(&payload.name) {
        Ok(_) => (StatusCode::OK, "Theme switched successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn handle_media(Json(payload): Json<MediaPayload>) -> impl IntoResponse {
    match control_media(&payload.action) {
        Ok(_) => (StatusCode::OK, "Media controlled successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

// --- Secure Core Functions ---

fn switch_theme(theme_name: &str) -> Result<()> {
    if !theme_name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        anyhow::bail!("Invalid theme name.");
    }
    let script_path = format!("{}/.config/hypr/scripts/switch_theme.sh", std::env::var("HOME")?);
    let status = Command::new(&script_path).arg(theme_name).status()?;
    if !status.success() { anyhow::bail!("Theme script error."); }
    Ok(())
}

fn control_media(action: &str) -> Result<()> {
    let valid_actions = ["play-pause", "next", "previous"];
    if !valid_actions.contains(&action) { anyhow::bail!("Invalid media action."); }
    let status = Command::new("playerctl").arg(action).status()?;
    if !status.success() { anyhow::bail!("Playerctl error."); }
    Ok(())
}