use super::*;

#[tauri::command]
pub fn get_avatars(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        let tl = &state.template_loader;
        let avatars: Vec<Value> = dm.list_avatars(uid).iter().filter_map(|aid| {
            let av = dm.get_avatar(uid, *aid)?;
            Some(json!({
                "avatar_id": *aid,
                "name": tl.avatar_name(*aid),
                "en_name": tl.avatar_en_name(*aid),
                "rarity": tl.avatar_rarity(*aid),
                "profession": tl.avatar_profession(*aid),
                "level": zon_int(&av, "level", 60),
                "unlocked_talent_num": zon_int(&av, "unlocked_talent_num", 0),
                "is_favorite": zon_bool(&av, "is_favorite", false),
                "camp_id": tl.avatar_camp(*aid),
            }))
        }).collect();
        json!({"avatars": avatars})
    })
}

#[tauri::command]
pub fn get_avatar(state: State<AppState>, uid: i64, avatar_id: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_avatar(uid, avatar_id) {
            Some(av) => {
                let tl = &state.template_loader;
                json!({
                    "avatar": {
                        "avatar_id": avatar_id,
                        "name": tl.avatar_name(avatar_id),
                        "en_name": tl.avatar_en_name(avatar_id),
                        "rarity": tl.avatar_rarity(avatar_id),
                        "profession": tl.avatar_profession(avatar_id),
                        "level": zon_int(&av, "level", 60),
                        "exp": zon_int(&av, "exp", 0),
                        "rank": zon_int(&av, "rank", 6),
                        "unlocked_talent_num": zon_int(&av, "unlocked_talent_num", 0),
                        "talent_switch_list": extract_talent_switches(&av),
                        "passive_skill_level": zon_int(&av, "passive_skill_level", 0),
                        "cur_weapon_uid": zon_int(&av, "cur_weapon_uid", 0),
                        "is_favorite": zon_bool(&av, "is_favorite", false),
                        "avatar_skin_id": zon_int(&av, "avatar_skin_id", 0),
                        "is_awake_available": zon_bool(&av, "is_awake_available", false),
                        "awake_id": zon_int(&av, "awake_id", 0),
                        "cur_form_id": zon_int(&av, "cur_form_id", 0),
                        "is_awake_enabled": zon_bool(&av, "is_awake_enabled", false),
                        "dressed_equip": extract_dressed_equip(&av),
                        "show_weapon_type": extract_show_weapon_type(&av),
                        "skill_type_level": extract_skills(&av),
                    },
                    "forms": []
                })
            }
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_avatar(state: State<AppState>, uid: i64, avatar_id: i64, mut data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        // Validate ranges
        if let Some(v) = data.get("level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_LEVEL, MAX_LEVEL, "level") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("rank").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_RANK, MAX_RANK, "rank") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("unlocked_talent_num").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_RANK, MAX_RANK, "unlocked_talent_num") { return json!({"ok": false, "error": e}); }
        }
        if let Some(v) = data.get("passive_skill_level").and_then(|v| v.as_i64()) {
            if let Err(e) = check_range(v, MIN_PASSIVE, MAX_PASSIVE, "passive_skill_level") { return json!({"ok": false, "error": e}); }
        }
        // Ensure enum fields are ZonEnum (not String) for correct ZON serialization
        if let Some(v) = data.get("show_weapon_type").and_then(|v| v.as_str()) {
            let val = v.to_string();
            data.insert("show_weapon_type".to_string(), ZonValue::Enum(crate::zon::ZonEnum(val)));
        }
        // Fix skill_type_level[].type to be ZonEnum
        if let Some(ZonValue::Array(skills)) = data.get_mut("skill_type_level") {
            for skill in skills.iter_mut() {
                if let ZonValue::Object(obj) = skill {
                    if let Some(v) = obj.get("type").and_then(|v| v.as_str()) {
                        let val = v.to_string();
                        obj.insert("type".to_string(), ZonValue::Enum(crate::zon::ZonEnum(val)));
                    }
                }
            }
        }
        // Validate structure before writing
        if let Err(e) = validate_avatar_data(&data) {
            return json!({"ok": false, "error": e});
        }
        // Merge with existing data: read existing, overlay updates, write full object
        // This matches Python version which always writes ALL fields
        if let Some(mut existing) = dm.get_avatar(uid, avatar_id) {
            for (k, v) in data {
                existing.insert(k, v);
            }
            dm.update_avatar(uid, avatar_id, &existing);
        } else {
            dm.update_avatar(uid, avatar_id, &data);
        }
        json!({"ok": true})
    })
}
