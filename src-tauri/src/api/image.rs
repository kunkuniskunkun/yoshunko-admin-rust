use super::*;

#[tauri::command]
pub fn read_image_data_url(path: String) -> Value {
    if path.contains("..") {
        return json!({"ok": false, "error": "路径不合法"});
    }
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return json!({"ok": false, "error": "文件不存在"});
    }
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return json!({"ok": false, "error": "不支持的图片格式"}),
    };
    match std::fs::read(p) {
        Ok(bytes) => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            json!({"ok": true, "url": format!("data:{};base64,{}", mime, b64)})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
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
