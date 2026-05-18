use super::*;

#[tauri::command]
pub fn list_logs(state: State<AppState>, key: String) -> Value {
    let logs = state.log_manager.list_logs(&key);
    let entries: Vec<Value> = logs.iter().map(|e| json!({
        "filename": e.filename,
        "display_name": e.display_name,
        "size": e.size,
    })).collect();
    json!({"logs": entries})
}

#[tauri::command]
pub fn read_log(state: State<AppState>, filename: String, offset: u64) -> Value {
    let (content, new_offset) = state.log_manager.read_log(&filename, offset);
    json!({"content": content, "offset": new_offset})
}

#[tauri::command]
pub fn get_log_dir(state: State<AppState>) -> Value {
    json!({"path": state.log_manager.log_dir().to_string_lossy().to_string()})
}

#[tauri::command]
pub fn open_log_dir(state: State<AppState>) -> Value {
    let dir = state.log_manager.log_dir();
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer").arg(dir).spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
    json!({"ok": true})
}
