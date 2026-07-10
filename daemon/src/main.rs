mod handlers;
mod models;
mod system;

use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{Response},
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use rand::{distr::Alphanumeric, RngExt};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use handlers::*;
use models::{AppState, AuthConfig, Notification};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Theme {
        name: String,
    },
    Media {
        action: String,
    },
    Serve {
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
    Pair,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Theme { name } => {
            system::switch_theme(name)?;
        }
        Commands::Media { action } => {
            system::control_media(action)?;
        }
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

            if let Ok(my_local_ip) = local_ip_address::local_ip() {
                let app_url = format!("http://{}:{}/?token={}", my_local_ip, port, token);
                println!("\n📱 Scan this QR code to instantly connect:\n");
                let _ = qr2term::print_qr(&app_url);
                println!("\n🔗 Fallback URL: {}\n", app_url);
            } else {
                println!("\n⚠️ Could not determine local IP for auto-pairing.");
            }

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
                                line[start + 1..end]
                                    .replace("\\n", "\n")
                                    .replace("<b>", "")
                                    .replace("</b>", "")
                                    .replace("<i>", "")
                                    .replace("</i>", "")
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        match string_count {
                            1 => current_app = content,
                            3 => current_summary = content,
                            4 => {
                                current_body = content;
                                let id: String = rand::rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(8)
                                    .map(char::from)
                                    .collect();
                                let mut hist = notifs_clone.lock().unwrap();
                                hist.insert(
                                    0,
                                    Notification {
                                        id,
                                        app_name: current_app.clone(),
                                        summary: current_summary.clone(),
                                        body: current_body.clone(),
                                    },
                                );
                                if hist.len() > 20 {
                                    hist.pop();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });

            let state = AppState {
                token,
                notifications,
            };

            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_headers(Any);

            let auth_routes = Router::new()
                .route("/api/state", get(handle_get_state))
                .route("/api/layout", get(handle_get_layout))
                .route("/api/theme", post(handle_theme))
                .route("/api/media", post(handle_media))
                .route("/api/audio", post(handle_audio))
                .route("/api/brightness", post(handle_brightness))
                .route("/api/system", post(handle_system))
                .route("/api/workspace", post(handle_workspace))
                .route("/api/bluetooth", post(handle_bluetooth))
                .route("/api/notifications", post(handle_notifications))
                .route("/api/execute", post(handle_execute))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ));

            let app = Router::new()
                .merge(auth_routes)
                .route("/api/apps", get(handle_get_apps))
                .route("/api/icon/{name}", get(handle_get_icon))
                .layer(cors)
                .fallback_service(
                    ServeDir::new("../frontend/out")
                        .fallback(ServeFile::new("../frontend/out/index.html")),
                )
                .with_state(state);

            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            println!("🚀 Command Center Daemon running on http://{}", addr);
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

fn generate_new_token() -> Result<String> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("command-center");
    fs::create_dir_all(&path)?;
    path.push("auth.json");
    let new_token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let auth = AuthConfig {
        token: new_token.clone(),
    };
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
    } else {
        anyhow::bail!("auth.json not found.");
    }
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected_header = format!("Bearer {}", state.token);
    match req.headers().get(header::AUTHORIZATION) {
        Some(value) if value.as_bytes() == expected_header.as_bytes() => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}