use super::*;

#[tauri::command]
pub fn get_hadal_zone(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_hadal_zone(uid) {
            Some(hz) => json!({
                "entrances": hz.get("entrances").map(|v| serde_json::to_value(v).unwrap_or_default()).unwrap_or_default(),
                "saved_rooms": hz.get("saved_rooms").map(|v| serde_json::to_value(v).unwrap_or_default()).unwrap_or(json!([])),
            }),
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_hadal_zone(state: State<AppState>, uid: i64, data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        // Merge with existing data to preserve saved_rooms not sent by frontend
        if let Some(mut existing) = dm.get_hadal_zone(uid) {
            for (k, v) in data {
                existing.insert(k, v);
            }
            dm.update_hadal_zone(uid, &existing);
        } else {
            dm.update_hadal_zone(uid, &data);
        }
        json!({"ok": true})
    })
}
