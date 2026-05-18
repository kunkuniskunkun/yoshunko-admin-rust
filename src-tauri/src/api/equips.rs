use super::*;

#[tauri::command]
pub fn get_equips(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        let tl = &state.template_loader;
        let equips: Vec<Value> = dm.list_equips(uid).iter().filter_map(|eid| {
            let e = dm.get_equip(uid, *eid)?;
            let item_id = zon_int(&e, "id", 0);
            Some(json!({
                "uid": *eid,
                "id": item_id,
                "suit_name": tl.suit_name(item_id),
                "suit_en_name": tl.suit_en_name(item_id),
                "slot": tl.equip_slot(item_id),
                "slot_name": slot_name(tl.equip_slot(item_id)),
                "level": zon_int(&e, "level", 0),
                "star": zon_int(&e, "star", 0),
            }))
        }).collect();
        json!({"equips": equips})
    })
}

#[tauri::command]
pub fn get_equip(state: State<AppState>, uid: i64, equip_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_equip(uid, equip_uid) {
            Some(e) => {
                let tl = &state.template_loader;
                let item_id = zon_int(&e, "id", 0);
                json!({
                    "uid": equip_uid,
                    "id": item_id,
                    "suit_name": tl.suit_name(item_id),
                    "suit_en_name": tl.suit_en_name(item_id),
                    "slot": tl.equip_slot(item_id),
                    "slot_name": slot_name(tl.equip_slot(item_id)),
                    "level": zon_int(&e, "level", 0),
                    "exp": zon_int(&e, "exp", 0),
                    "star": zon_int(&e, "star", 0),
                    "lock": zon_bool(&e, "lock", false),
                    "properties": extract_equip_properties(&e, "properties"),
                    "sub_properties": extract_equip_properties(&e, "sub_properties"),
                })
            }
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_equip(state: State<AppState>, uid: i64, equip_uid: i64, mut data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        if let Some(v) = data.get("level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_EQUIP_LEVEL, MAX_EQUIP_LEVEL, "level") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("star").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_STAR, MAX_EQUIP_STAR, "star") { return json!({"ok": false, "error": e}); }
        }
        // Validate structure before writing
        if let Err(e) = validate_equip_data(&data) {
            return json!({"ok": false, "error": e});
        }
        clean_equip_data(&mut data);
        // Merge with existing data to preserve fields not sent by frontend (id, exp, lock)
        if let Some(mut existing) = dm.get_equip(uid, equip_uid) {
            for (k, v) in data {
                existing.insert(k, v);
            }
            existing.entry("exp".to_string()).or_insert(ZonValue::Int(0));
            existing.entry("lock".to_string()).or_insert(ZonValue::Bool(false));
            dm.update_equip(uid, equip_uid, &existing);
        } else {
            data.entry("exp".to_string()).or_insert(ZonValue::Int(0));
            data.entry("lock".to_string()).or_insert(ZonValue::Bool(false));
            dm.update_equip(uid, equip_uid, &data);
        }
        json!({"ok": true})
    })
}

#[tauri::command]
pub fn create_equip(state: State<AppState>, uid: i64, mut data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        if let Some(v) = data.get("level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_EQUIP_LEVEL, MAX_EQUIP_LEVEL, "level") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("star").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_STAR, MAX_EQUIP_STAR, "star") { return json!({"ok": false, "error": e}); }
        }
        // Validate structure before writing
        if let Err(e) = validate_equip_data(&data) {
            return json!({"ok": false, "error": e});
        }
        clean_equip_data(&mut data);
        // Ensure exp and lock fields exist (server expects them)
        data.entry("exp".to_string()).or_insert(ZonValue::Int(0));
        data.entry("lock".to_string()).or_insert(ZonValue::Bool(false));
        match dm.create_equip(uid, &data) {
            Ok(new_uid) => json!({"ok": true, "uid": new_uid}),
            Err(e) => json!({"ok": false, "error": e}),
        }
    })
}

#[tauri::command]
pub fn delete_equip(state: State<AppState>, uid: i64, equip_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.delete_equip(uid, equip_uid) {
            Ok(()) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e}),
        }
    })
}

#[tauri::command]
pub fn copy_equip(state: State<AppState>, uid: i64, equip_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.copy_equip(uid, equip_uid) {
            Ok(new_uid) => json!({"ok": true, "uid": new_uid}),
            Err(e) => json!({"ok": false, "error": e}),
        }
    })
}
