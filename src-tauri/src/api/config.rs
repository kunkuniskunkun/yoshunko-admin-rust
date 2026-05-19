use super::*;
use crate::data_manager::DataManager;

// ─── Validation constants ──────────────────────────────────

pub const MIN_LEVEL: i64 = 1;
pub const MAX_LEVEL: i64 = 60;
pub const MIN_STAR: i64 = 1;
pub const MAX_STAR: i64 = 5;
pub const MIN_REFINE: i64 = 1;
pub const MAX_REFINE: i64 = 5;
pub const MIN_RANK: i64 = 0;
pub const MAX_RANK: i64 = 6;
pub const MIN_PASSIVE: i64 = 0;
pub const MAX_PASSIVE: i64 = 6;
pub const MIN_EQUIP_LEVEL: i64 = 0;
pub const MAX_EQUIP_LEVEL: i64 = 15;
pub const MAX_EQUIP_STAR: i64 = 5;

pub fn check_range(value: i64, min: i64, max: i64, name: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{} 必须在 {} 到 {} 之间，当前值: {}", name, min, max, value))
    } else {
        Ok(())
    }
}

pub fn format_version() -> String {
    let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let version = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| {
            let v: serde_json::Value = serde_json::from_str(&s).ok()?;
            v.get("version")?.as_str().map(|s| s.to_string())
        })
        .unwrap_or_else(|| "0.0.0".to_string());
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0];
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        format!("V{}.{:03}", major, minor)
    } else {
        format!("V{}", version)
    }
}

// Helper: call f with mutable DataManager reference, return error JSON on failure
pub fn with_manager<F>(state: &AppState, f: F) -> Value
where
    F: FnOnce(&mut DataManager) -> Value,
{
    let mut guard = state.data_manager.lock().unwrap_or_else(|poisoned| {
        eprintln!("[with_manager] Mutex was poisoned, recovering...");
        poisoned.into_inner()
    });
    match guard.as_mut() {
        Some(dm) => f(dm),
        None => json!({"ok": false, "error": "状态目录未配置"}),
    }
}

/// 原子写入配置文件：tmp + write + sync + rename
pub fn atomic_write_config(config_path: &str, config: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let tmp = format!("{}.tmp", config_path);
    let mut f = std::fs::File::create(&tmp).map_err(|e| format!("创建临时文件失败: {}", e))?;
    serde_json::to_writer_pretty(&mut f, config).map_err(|e| format!("写入配置失败: {}", e))?;
    f.sync_all().map_err(|e| format!("同步磁盘失败: {}", e))?;
    std::fs::rename(&tmp, config_path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(())
}

// ─── Config commands ─────────────────────────────────────

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Value {
    let dm_configured = state.data_manager.lock().ok().map_or(false, |g| g.is_some());
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let launch_config = config.get("launch").cloned().unwrap_or(json!({}));
    let state_dir = config.get("state_dir").and_then(|v| v.as_str()).unwrap_or("");
    let background = config.get("background").cloned();
    json!({
        "configured": dm_configured,
        "state_dir": state_dir,
        "version": format_version(),
        "config_exists": std::fs::metadata(&state.config_path).is_ok(),
        "launch_config": launch_config,
        "background": background
    })
}

#[tauri::command]
pub fn get_version() -> Value {
    json!({"version": format_version()})
}

#[tauri::command]
pub fn set_state_dir(state: State<AppState>, path: String) -> Value {
    let path = path.trim().trim_matches('"').trim_matches('\'');
    if path.is_empty() {
        return json!({"ok": false, "error": "路径不能为空"});
    }
    if path.contains("..") {
        return json!({"ok": false, "error": "路径不能包含 .."});
    }
    let player_dir = std::path::Path::new(path).join("player");
    if !player_dir.is_dir() {
        return json!({"ok": false, "error": format!("目录下未找到 player/ 子目录: {}", path)});
    }
    let dm = DataManager::new(path);
    // Read existing config to preserve launch fields
    let mut config_map: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    config_map.insert("state_dir".to_string(), json!(path));
    config_map.insert("version".to_string(), json!(format_version()));
    if let Err(e) = atomic_write_config(&state.config_path, &config_map) {
        return json!({"ok": false, "error": e});
    }
    if let Ok(mut guard) = state.data_manager.lock() {
        *guard = Some(dm);
    }
    json!({"ok": true})
}

#[tauri::command]
pub fn set_background(state: State<AppState>, path: String, opacity: f64) -> Value {
    let opacity = opacity.clamp(0.3, 0.95);
    let mut config_map: serde_json::Map<String, serde_json::Value> =
        std::fs::read_to_string(&state.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    if path.is_empty() {
        config_map.remove("background");
    } else {
        let mut bg = serde_json::Map::new();
        bg.insert("path".to_string(), json!(path));
        bg.insert("opacity".to_string(), json!(opacity));
        config_map.insert("background".to_string(), serde_json::Value::Object(bg));
    }
    if let Err(e) = atomic_write_config(&state.config_path, &config_map) {
        return json!({"ok": false, "error": e});
    }
    json!({"ok": true})
}

#[tauri::command]
pub fn auto_detect_paths() -> Value {
    let mut candidates = Vec::new();
    if let Ok(distro) = std::fs::read_dir(r"\\wsl.localhost") {
        for entry in distro.flatten() {
            let p = entry.path().join("root").join("yoshunko").join("state");
            if p.join("player").is_dir() {
                candidates.push(p.to_string_lossy().to_string());
            }
        }
    }
    for p in [r"D:\3.0.1\state", "/root/yoshunko/state"] {
        if std::path::Path::new(p).join("player").is_dir() {
            candidates.push(p.to_string());
        }
    }
    json!({"candidates": candidates})
}

#[tauri::command]
pub fn debug_list_dir(path: String) -> Value {
    let dir = std::path::Path::new(&path);
    let mut entries = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            entries.push(json!({"name": name, "is_dir": is_dir}));
        }
    }
    json!({"path": path, "exists": dir.exists(), "is_dir": dir.is_dir(), "entries": entries})
}

#[tauri::command]
pub fn debug_avatar_ids(state: State<AppState>) -> Value {
    let guard = match state.data_manager.lock() {
        Ok(g) => g,
        Err(e) => return json!({"error": format!("Lock poisoned: {}", e)}),
    };
    if let Some(dm) = guard.as_ref() {
        let ids = dm.list_avatars(1);
        let count = ids.len();
        // Try reading and parsing first avatar with error capture
        let first_result = if let Some(first_id) = ids.first() {
            let path = dm._debug_avatar_path(1, *first_id);
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let preview: String = content.chars().take(300).collect();
            match crate::zon::parse_zon(&content) {
                Ok(data) => {
                    let keys: Vec<String> = match &data {
                        crate::zon::ZonValue::Object(obj) => obj.keys().cloned().collect(),
                        _ => vec![format!("{:?}", data)],
                    };
                    json!({"id": first_id, "parsed": true, "keys": keys})
                }
                Err(e) => {
                    json!({"id": first_id, "parsed": false, "error": e, "preview": preview})
                }
            }
        } else {
            json!(null)
        };
        json!({"count": count, "first_result": first_result})
    } else {
        json!({"error": "no data manager"})
    }
}

// ─── Window Controls ────────────────────────────────────

#[tauri::command]
pub fn window_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_toggle_max(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn window_close(window: tauri::Window) {
    let _ = window.close();
}
