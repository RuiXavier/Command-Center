use crate::models::*;
use crate::system::*;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::fs;
use std::process::Command;

pub async fn handle_get_state(State(state): State<AppState>) -> impl IntoResponse {
    let notifs = state.notifications.lock().unwrap().clone();
    match get_system_state(notifs) {
        Ok(sys_state) => (StatusCode::OK, Json(sys_state)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read system state",
        )
            .into_response(),
    }
}

pub async fn handle_theme(Json(payload): Json<ThemePayload>) -> impl IntoResponse {
    match switch_theme(&payload.name) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_media(Json(payload): Json<MediaPayload>) -> impl IntoResponse {
    match control_media(&payload.action, payload.player) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_audio(Json(payload): Json<AudioPayload>) -> impl IntoResponse {
    match control_audio(&payload.action, payload.value) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_brightness(Json(payload): Json<BrightnessPayload>) -> impl IntoResponse {
    match control_brightness(&payload.action, payload.value) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_system(Json(payload): Json<SystemPayload>) -> impl IntoResponse {
    match control_system(&payload.action) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_workspace(Json(payload): Json<WorkspacePayload>) -> impl IntoResponse {
    match control_workspace(payload.id) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_bluetooth(Json(payload): Json<SystemPayload>) -> impl IntoResponse {
    match control_bluetooth(&payload.action) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn handle_notifications(
    State(state): State<AppState>,
    Json(payload): Json<NotificationPayload>,
) -> impl IntoResponse {
    let mut notifs = state.notifications.lock().unwrap();
    if payload.action == "clear" {
        notifs.clear();
        let _ = Command::new("swaync-client").arg("-C").status();
    } else if payload.action == "clear_one" {
        if let Some(id) = payload.id {
            notifs.retain(|n| n.id != id);
        }
    }
    StatusCode::OK.into_response()
}

pub async fn handle_get_layout() -> impl IntoResponse {
    match get_or_create_config() {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            eprintln!("💥 Config Error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Config error: {}", e),
            )
                .into_response()
        }
    }
}

pub async fn handle_execute(Json(payload): Json<ExecutePayload>) -> impl IntoResponse {
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&payload.command)
        .spawn();
    StatusCode::OK.into_response()
}

pub async fn handle_get_apps() -> Json<Vec<SystemApp>> {
    let mut apps = Vec::new();

    let mut dirs = vec![
        "/usr/share/applications".to_string(),
        "/var/lib/flatpak/exports/share/applications".to_string(),
    ];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(format!("{}/.local/share/applications", home));
    }

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let mut name = String::new();
                        let mut exec = String::new();
                        let mut icon = String::new();
                        let mut no_display = false;

                        for line in content.lines() {
                            if line.starts_with("Name=") && name.is_empty() {
                                name = line["Name=".len()..].to_string();
                            } else if line.starts_with("Exec=") && exec.is_empty() {
                                let raw_exec = line["Exec=".len()..].to_string();
                                exec = raw_exec
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                            } else if line.starts_with("Icon=") && icon.is_empty() {
                                icon = line["Icon=".len()..].to_string();
                            } else if line.starts_with("NoDisplay=true") {
                                no_display = true;
                            }
                        }

                        if !name.is_empty() && !exec.is_empty() && !no_display {
                            apps.push(SystemApp {
                                name,
                                command: exec,
                                icon,
                            });
                        }
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);

    Json(apps)
}

pub async fn handle_get_icon(
    Path(icon_name): Path<String>,
) -> impl IntoResponse {
    if let Some(path) = resolve_icon_path(&icon_name) {
        if let Ok(bytes) = std::fs::read(&path) {
            let mime = if path.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "image/png"
            };
            return (StatusCode::OK, [(header::CONTENT_TYPE, mime)], bytes).into_response();
        }
    }
    
    // Instead of a 404 error, return a blank, invisible SVG so the browser console stays clean!
    let empty_svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>";
    (StatusCode::OK, [(header::CONTENT_TYPE, "image/svg+xml")], empty_svg.as_bytes()).into_response()
}

pub async fn handle_get_media_metadata() -> impl IntoResponse {
    // Add .await here:
    match get_media_metadata().await {
        Ok(metadata) => (StatusCode::OK, Json(metadata)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read media metadata",
        )
            .into_response(),
    }
}


// Add `Query` to your axum::extract import at the top:
// use axum::extract::{Path, State, Query};

pub async fn handle_media_art(
    axum::extract::Query(query): axum::extract::Query<ArtQuery>,
) -> impl IntoResponse {
    let url = query.url;

    if url.starts_with("file://") {
        // Strip the file URI and do a basic decode for spaces
        let path = url.trim_start_matches("file://").replace("%20", " ");

        if let Ok(bytes) = std::fs::read(&path) {
            let mime = if path.to_lowercase().ends_with(".png") {
                "image/png"
            } else if path.to_lowercase().ends_with(".svg") {
                "image/svg+xml"
            } else {
                "image/jpeg"
            };
            return (StatusCode::OK, [(header::CONTENT_TYPE, mime)], bytes).into_response();
        }
    } else if url.starts_with("http") {
        // If the player passed a direct web URL (like some Spotify clients do), 
        // just redirect the browser to it directly.
        return (
            StatusCode::FOUND,
            [(header::LOCATION, url)],
        ).into_response();
    }

    // Invisible pixel fallback if the art doesn't exist
    let empty_svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>";
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        empty_svg.as_bytes(),
    )
        .into_response()
}

