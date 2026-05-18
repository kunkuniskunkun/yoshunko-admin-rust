use super::*;

#[tauri::command]
pub fn get_weapons(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        let tl = &state.template_loader;
        let weapons: Vec<Value> = dm.list_weapons(uid).iter().filter_map(|wid| {
            let w = dm.get_weapon(uid, *wid)?;
            let item_id = zon_int(&w, "id", 0);
            let rarity = if item_id >= 14000 { "S" } else if item_id >= 13000 { "A" } else { "B" };
            Some(json!({
                "uid": *wid,
                "id": item_id,
                "name": tl.weapon_name(item_id),
                "en_name": tl.weapon_en_name(item_id),
                "profession": tl.weapon_profession(item_id),
                "rarity": rarity,
                "level": zon_int(&w, "level", 60),
                "star": zon_int(&w, "star", 1),
                "refine_level": zon_int(&w, "refine_level", 1),
                "max_star": tl.weapon_max_star(item_id),
                "max_refine": tl.weapon_max_refine(item_id),
            }))
        }).collect();
        json!({"weapons": weapons})
    })
}

#[tauri::command]
pub fn get_weapon(state: State<AppState>, uid: i64, weapon_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_weapon(uid, weapon_uid) {
            Some(w) => {
                let tl = &state.template_loader;
                let item_id = zon_int(&w, "id", 0);
                let rarity = if item_id >= 14000 { "S" } else if item_id >= 13000 { "A" } else { "B" };
                json!({
                    "uid": weapon_uid,
                    "id": item_id,
                    "name": tl.weapon_name(item_id),
                    "en_name": tl.weapon_en_name(item_id),
                    "profession": tl.weapon_profession(item_id),
                    "rarity": rarity,
                    "level": zon_int(&w, "level", 60),
                    "star": zon_int(&w, "star", 1),
                    "refine_level": zon_int(&w, "refine_level", 1),
                    "lock": zon_bool(&w, "lock", false),
                    "max_star": tl.weapon_max_star(item_id),
                    "max_refine": tl.weapon_max_refine(item_id),
                })
            }
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_weapon(state: State<AppState>, uid: i64, weapon_uid: i64, mut data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        if let Some(v) = data.get("level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_LEVEL, MAX_LEVEL, "level") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("star").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_STAR, MAX_STAR, "star") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("refine_level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_REFINE, MAX_REFINE, "refine_level") { return json!({"ok": false, "error": e}); }
        }
        // Merge with existing data to preserve fields not sent by frontend (id, exp, star, lock)
        if let Some(mut existing) = dm.get_weapon(uid, weapon_uid) {
            for (k, v) in data {
                existing.insert(k, v);
            }
            // Ensure exp and lock fields exist
            existing.entry("exp".to_string()).or_insert(ZonValue::Int(0));
            existing.entry("lock".to_string()).or_insert(ZonValue::Bool(false));
            dm.update_weapon(uid, weapon_uid, &existing);
        } else {
            data.entry("exp".to_string()).or_insert(ZonValue::Int(0));
            data.entry("lock".to_string()).or_insert(ZonValue::Bool(false));
            dm.update_weapon(uid, weapon_uid, &data);
        }
        json!({"ok": true})
    })
}

#[tauri::command]
pub fn delete_weapon(state: State<AppState>, uid: i64, weapon_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.delete_weapon(uid, weapon_uid) {
            Ok(()) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e}),
        }
    })
}

#[tauri::command]
pub fn copy_weapon(state: State<AppState>, uid: i64, weapon_uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.copy_weapon(uid, weapon_uid) {
            Ok(new_uid) => json!({"ok": true, "uid": new_uid}),
            Err(e) => json!({"ok": false, "error": e}),
        }
    })
}
