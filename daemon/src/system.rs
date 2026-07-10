use crate::models::*;
use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

pub fn get_or_create_config() -> Result<AppConfig> {
    let mut path = dirs::config_dir().context("Could not find config dir")?;
    path.push("command-center");
    fs::create_dir_all(&path)?;
    path.push("config.toml");

    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let default_config = AppConfig {
            general: GeneralConfig {
                theme: "glass".to_string(),
            },
            layout: vec![
                ModuleConfig::Telemetry {},
                ModuleConfig::Notifications {},
                ModuleConfig::Workspaces {},
                ModuleConfig::Slider {
                    endpoint: "audio".to_string(),
                    label: "Master Volume".to_string(),
                    color: "bg-white".to_string(),
                },
                ModuleConfig::Slider {
                    endpoint: "brightness".to_string(),
                    label: "Display Brightness".to_string(),
                    color: "bg-yellow-400".to_string(),
                },
                ModuleConfig::MediaSystem {},
            ],
        };
        let toml_string = toml::to_string(&default_config)?;
        fs::write(&path, toml_string)?;
        Ok(default_config)
    }
}

pub fn get_system_state(notifications: Vec<Notification>) -> Result<SystemState> {
    let audio_out = String::from_utf8(
        Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()?
            .stdout,
    )?;
    let is_muted = audio_out.contains("[MUTED]");
    let vol_str = audio_out
        .replace("Volume: ", "")
        .replace("[MUTED]", "")
        .trim()
        .to_string();
    let volume = (vol_str.parse::<f32>().unwrap_or(0.0) * 100.0) as u32;

    let bright_out = String::from_utf8(
        Command::new("brightnessctl")
            .arg("-m")
            .output()?
            .stdout,
    )?;
    let parts: Vec<&str> = bright_out.split(',').collect();
    let brightness = parts
        .get(3)
        .unwrap_or(&"0%")
        .replace("%", "")
        .trim()
        .parse::<u32>()
        .unwrap_or(0);

    let ws_out = String::from_utf8(
        Command::new("hyprctl")
            .args(["workspaces", "-j"])
            .output()?
            .stdout,
    )?;
    let parsed_ws: serde_json::Value = serde_json::from_str(&ws_out)?;
    let mut workspaces = Vec::new();
    if let Some(arr) = parsed_ws.as_array() {
        for w in arr {
            if let Some(id) = w.get("id").and_then(|v| v.as_i64()) {
                workspaces.push(id as i32);
            }
        }
    }
    workspaces.sort();

    let active_ws_out = String::from_utf8(
        Command::new("hyprctl")
            .args(["activeworkspace", "-j"])
            .output()?
            .stdout,
    )?;
    let parsed_active: serde_json::Value = serde_json::from_str(&active_ws_out)?;
    let active_workspace = parsed_active
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    let (battery, wifi_ssid, bluetooth_on, bt_device) = get_telemetry();

    Ok(SystemState {
        volume,
        is_muted,
        brightness,
        workspaces,
        active_workspace,
        battery,
        wifi_ssid,
        bluetooth_on,
        bt_device,
        notifications,
    })
}

pub fn switch_theme(theme_name: &str) -> Result<()> {
    let script_path = format!("{}/.config/hypr/scripts/switch_theme.sh", std::env::var("HOME")?);
    Command::new(&script_path).arg(theme_name).status()?;
    Ok(())
}

pub fn control_media(action: &str, player: Option<String>) -> Result<()> {
    let mut cmd = Command::new("playerctl");
    if let Some(p) = player {
        cmd.args(["-p", &p]); // Target the specific app!
    }
    cmd.arg(action).status()?;
    Ok(())
}

pub fn control_system(action: &str) -> Result<()> {
    if action == "lock" {
        Command::new("hyprlock").status()?;
    }
    Ok(())
}

pub fn control_workspace(id: i32) -> Result<()> {
    let lua_dispatcher = format!("hl.dsp.focus({{ workspace = \"{}\" }})", id);
    let status = Command::new("hyprctl")
        .args(["dispatch", &lua_dispatcher])
        .status()?;
    if !status.success() {
        anyhow::bail!("Hyprland workspace switch failed.");
    }
    Ok(())
}

pub fn control_audio(action: &str, value: Option<u32>) -> Result<()> {
    match action {
        "set" => {
            if let Some(v) = value {
                let vol_str = format!("{}%", v);
                Command::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str])
                    .status()?;
            }
        }
        "mute" => {
            Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .status()?;
        }
        _ => {}
    }
    Ok(())
}

pub fn control_bluetooth(action: &str) -> Result<()> {
    if action == "toggle" {
        let output = Command::new("bluetoothctl").arg("show").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let powered = stdout
            .lines()
            .find_map(|l| {
                let l = l.trim();
                if l.starts_with("Powered:") {
                    Some(l.split(':').nth(1).unwrap_or("").trim().to_lowercase())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "no".to_string());

        let turn_on = match powered.as_str() {
            "yes" | "true" => false,
            _ => true,
        };
        let arg = if turn_on { "on" } else { "off" };
        Command::new("bluetoothctl").arg("power").arg(arg).status()?;
    }
    Ok(())
}

pub fn control_brightness(action: &str, value: Option<u32>) -> Result<()> {
    if action == "set" {
        if let Some(v) = value {
            let b_str = format!("{}%", v);
            Command::new("brightnessctl")
                .args(["set", &b_str])
                .status()?;
        }
    }
    Ok(())
}

pub fn get_telemetry() -> (u8, String, bool, String) {
    let battery = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .unwrap_or_else(|_| "100".to_string())
        .trim()
        .parse::<u8>()
        .unwrap_or(100);

    let wifi_cmd = Command::new("sh")
        .arg("-c")
        .arg("nmcli -t -f active,ssid dev wifi | grep '^yes' | cut -d':' -f2")
        .output();
    let wifi_ssid = match wifi_cmd {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                "Disconnected".to_string()
            } else {
                s
            }
        }
        Err(_) => "Unknown".to_string(),
    };

    let bt_info = Command::new("bluetoothctl").arg("show").output();
    let bluetooth_on = match bt_info {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains("Powered: yes"),
        Err(_) => false,
    };

    let bt_devices = Command::new("sh")
        .arg("-c")
        .arg("bluetoothctl devices Connected | cut -d' ' -f3-")
        .output();
    let bt_device = match bt_devices {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                "None".to_string()
            } else {
                s
            }
        }
        Err(_) => "None".to_string(),
    };

    (battery, wifi_ssid, bluetooth_on, bt_device)
}

pub fn resolve_icon_path(icon_name: &str) -> Option<String> {
    if icon_name.starts_with('/') {
        return Some(icon_name.to_string());
    }

    let mut search_paths = vec![
        format!("/usr/share/icons/hicolor/scalable/apps/{}.svg", icon_name),
        format!("/usr/share/icons/hicolor/128x128/apps/{}.png", icon_name),
        format!("/usr/share/icons/hicolor/64x64/apps/{}.png", icon_name),
        format!("/usr/share/icons/hicolor/48x48/apps/{}.png", icon_name),
        format!("/usr/share/icons/hicolor/256x256/apps/{}.png", icon_name),
        format!("/usr/share/pixmaps/{}.png", icon_name),
        format!("/usr/share/pixmaps/{}.svg", icon_name),
        format!(
            "/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps/{}.svg",
            icon_name
        ),
        format!(
            "/var/lib/flatpak/exports/share/icons/hicolor/128x128/apps/{}.png",
            icon_name
        ),
        format!(
            "/var/lib/flatpak/exports/share/icons/hicolor/64x64/apps/{}.png",
            icon_name
        ),
    ];

    if let Ok(home) = std::env::var("HOME") {
        search_paths.push(format!(
            "{}/.local/share/icons/hicolor/scalable/apps/{}.svg",
            home, icon_name
        ));
        search_paths.push(format!(
            "{}/.local/share/icons/hicolor/128x128/apps/{}.png",
            home, icon_name
        ));
        search_paths.push(format!(
            "{}/.local/share/icons/hicolor/64x64/apps/{}.png",
            home, icon_name
        ));
    }

    for path in search_paths {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

pub async fn get_media_metadata() -> Result<Vec<MediaMetadata>> {
    let mut media_list = Vec::new();
    
    // 1. Get standard MPRIS players (Brave, Spotify, etc.) using tokio
    let output = tokio::process::Command::new("playerctl").arg("-l").output().await?;
    let players = String::from_utf8_lossy(&output.stdout);
    
    for player in players.lines() {
        let player = player.trim();
        if player.is_empty() { continue; }
        
        let meta_out = tokio::process::Command::new("playerctl")
            .args(["-p", player, "metadata", "--format", "{{title}}|~|{{artist}}|~|{{mpris:artUrl}}|~|{{status}}"])
            .output().await;
            
        // Removed the stray .unwrap() here!
        if let Ok(out) = meta_out {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stdout.is_empty() { continue; }
            
            let parts: Vec<&str> = stdout.split("|~|").collect();
            let title = parts.get(0).unwrap_or(&"").trim().to_string();
            let status = parts.get(3).unwrap_or(&"Stopped").trim().to_string();
            
            if !title.is_empty() || status == "Playing" || status == "Paused" {
                media_list.push(MediaMetadata {
                    player_name: player.to_string(),
                    title: if title.is_empty() { "Unknown Media".to_string() } else { title },
                    artist: parts.get(1).unwrap_or(&"Unknown").trim().to_string(),
                    art_url: parts.get(2).unwrap_or(&"").trim().to_string(),
                    status,
                });
            }
        }
    }

    // 2. Custom Stremio Local Engine Poller
    let client = reqwest::Client::new();
    if let Ok(res) = client.get("http://127.0.0.1:11470/stats.json").timeout(std::time::Duration::from_millis(500)).send().await {
        if let Ok(stats) = res.json::<serde_json::Value>().await {
            if let Some(stream_name) = stats.get("streamName").and_then(|s| s.as_str()) {
                let speed = stats.get("speed").and_then(|s| s.as_f64()).unwrap_or(0.0);
                
                media_list.push(MediaMetadata {
                    player_name: "Stremio Native".to_string(),
                    title: stream_name.to_string(),
                    artist: "Local Engine".to_string(),
                    art_url: "".to_string(), 
                    status: if speed > 1024.0 { "Playing".to_string() } else { "Paused".to_string() },
                });
            }
        }
    }
    
    Ok(media_list)
}