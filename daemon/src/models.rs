use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub token: String,
    pub notifications: Arc<Mutex<Vec<Notification>>>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ThemePayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct MediaPayload {
    pub action: String,
    pub player: Option<String>,
}

#[derive(Deserialize)]
pub struct AudioPayload {
    pub action: String,
    pub value: Option<u32>,
}

#[derive(Deserialize)]
pub struct BrightnessPayload {
    pub action: String,
    pub value: Option<u32>,
}

#[derive(Deserialize)]
pub struct SystemPayload {
    pub action: String,
}

#[derive(Deserialize)]
pub struct WorkspacePayload {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct NotificationPayload {
    pub action: String,
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct ExecutePayload {
    pub command: String,
}

#[derive(Serialize, Clone)]
pub struct Notification {
    pub id: String,
    pub app_name: String,
    pub summary: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct SystemState {
    pub volume: u32,
    pub is_muted: bool,
    pub brightness: u32,
    pub workspaces: Vec<i32>,
    pub active_workspace: i32,
    pub battery: u8,
    pub wifi_ssid: String,
    pub bluetooth_on: bool,
    pub bt_device: String,
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModuleConfig {
    Telemetry {},
    Notifications {},
    Workspaces {},
    MediaSystem {},
    Slider {
        endpoint: String,
        label: String,
        color: String,
    },
    AppLauncher {
        apps: Vec<AppShortcut>,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppShortcut {
    pub name: String,
    pub command: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub theme: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub layout: Vec<ModuleConfig>,
}

#[derive(Serialize)]
pub struct SystemApp {
    pub name: String,
    pub command: String,
    pub icon: String,
}

#[derive(Serialize)]
pub struct MediaMetadata {
    pub player_name: String,
    pub title: String,
    pub artist: String,
    pub art_url: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ArtQuery {
    pub url: String,
}