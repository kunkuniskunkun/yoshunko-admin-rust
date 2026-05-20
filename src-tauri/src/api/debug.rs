use super::*;

#[allow(dead_code)]
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

#[allow(dead_code)]
#[tauri::command]
pub fn debug_avatar_ids(state: State<AppState>) -> Value {
    let guard = match state.data_manager.lock() {
        Ok(g) => g,
        Err(e) => return json!({"error": format!("Lock poisoned: {}", e)}),
    };
    if let Some(dm) = guard.as_ref() {
        let ids = dm.list_avatars(1);
        let count = ids.len();
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
