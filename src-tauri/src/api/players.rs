use super::*;

#[tauri::command]
pub fn get_player_list(state: State<AppState>) -> Value {
    with_manager(&state, |dm| {
        json!({"players": dm.list_players()})
    })
}

#[tauri::command]
pub fn get_player_basic(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_basic_info(uid) {
            Some(info) => json!({
                "nickname": zon_str(&info, "nickname", "Unknown"),
                "level": zon_int(&info, "level", 60),
                "exp": zon_int(&info, "exp", 0),
                "avatar_id": zon_int(&info, "avatar_id", 2011),
                "control_avatar_id": zon_int(&info, "control_avatar_id", 2011),
                "control_guise_avatar_id": zon_int(&info, "control_guise_avatar_id", 1541),
            }),
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_player_basic(state: State<AppState>, uid: i64, data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        if let Some(v) = data.get("level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_LEVEL, MAX_LEVEL, "level") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("exp").and_then(|v| v.as_i64()) {
            if v < 0 { return json!({"ok": false, "error": "经验值不能为负数"}); }
        }
        if let Some(v) = data.get("avatar_id").and_then(|v| v.as_i64()) {
            if v < 0 { return json!({"ok": false, "error": "角色 ID 不能为负数"}); }
        }
        dm.update_basic_info(uid, &data);
        json!({"ok": true})
    })
}
