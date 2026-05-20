use super::*;
use crate::data_manager::DataManager;

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
    for p in ["/root/yoshunko/state"] {
        if std::path::Path::new(p).join("player").is_dir() {
            candidates.push(p.to_string());
        }
    }
    json!({"candidates": candidates})
}

#[tauri::command]
pub fn open_release_page() -> Value {
    let repo = "https://github.com/kunkuniskunkun/yoshunko-admin-rust/releases";
    if let Err(e) = std::process::Command::new("cmd")
        .args(["/c", "start", repo])
        .spawn()
    {
        return json!({"ok": false, "error": e.to_string()});
    }
    json!({"ok": true})
}
