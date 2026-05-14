// Tauri command handlers — JS Bridge API
// Returns data directly (no wrapper), matching Python api.py behavior

use crate::data_manager::DataManager;
use crate::template_loader::TemplateLoader;
use crate::zon::ZonValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub data_manager: Mutex<Option<DataManager>>,
    pub template_loader: TemplateLoader,
    pub config_path: String,
}

// ─── Validation constants ──────────────────────────────────
const MIN_LEVEL: i64 = 1;
const MAX_LEVEL: i64 = 60;
const MIN_STAR: i64 = 1;
const MAX_STAR: i64 = 5;
const MIN_REFINE: i64 = 1;
const MAX_REFINE: i64 = 5;
const MIN_RANK: i64 = 0;
const MAX_RANK: i64 = 6;
const MIN_PASSIVE: i64 = 0;
const MAX_PASSIVE: i64 = 6;
const MIN_EQUIP_LEVEL: i64 = 0;
const MAX_EQUIP_LEVEL: i64 = 15;
const MAX_EQUIP_STAR: i64 = 5;

fn check_range(value: i64, min: i64, max: i64, name: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{} must be between {} and {}, got {}", name, min, max, value))
    } else {
        Ok(())
    }
}

// Helper: call f with DataManager reference, return error JSON on failure
fn with_manager<F>(state: &AppState, f: F) -> Value
where
    F: FnOnce(&DataManager) -> Value,
{
    match state.data_manager.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(dm) => f(dm),
            None => json!({"ok": false, "error": "状态目录未配置"}),
        },
        Err(_) => json!({"ok": false, "error": "内部错误"}),
    }
}

// ─── Config ─────────────────────────────────────────────

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Value {
    let dm_configured = state.data_manager.lock().ok().map_or(false, |g| g.is_some());
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let launch_config = config.get("launch").cloned().unwrap_or(json!({}));
    let state_dir = config.get("state_dir").and_then(|v| v.as_str()).unwrap_or("");
    json!({
        "configured": dm_configured,
        "state_dir": state_dir,
        "version": "V0.700",
        "config_exists": std::fs::metadata(&state.config_path).is_ok(),
        "launch_config": launch_config
    })
}

#[tauri::command]
pub fn get_version() -> Value {
    let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let version = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| {
            let v: serde_json::Value = serde_json::from_str(&s).ok()?;
            v.get("version")?.as_str().map(|s| s.to_string())
        })
        .unwrap_or_else(|| "0.0.0".to_string());
    // Format as "Vx.yyy" — e.g. "0.615.0" → "V0.615"
    let parts: Vec<&str> = version.split('.').collect();
    let formatted = if parts.len() >= 2 {
        let major = parts[0];
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        format!("V{}.{:03}", major, minor)
    } else {
        format!("V{}", version)
    };
    json!({"version": formatted})
}

#[tauri::command]
pub fn set_state_dir(state: State<AppState>, path: String) -> Value {
    let path = path.trim().trim_matches('"').trim_matches('\'');
    if path.is_empty() {
        return json!({"ok": false, "error": "路径不能为空"});
    }
    let player_dir = std::path::Path::new(path).join("player");
    if !player_dir.is_dir() {
        return json!({"ok": false, "error": format!("目录下未找到 player/ 子目录: {}", path)});
    }
    let dm = DataManager::new(path);
    let config = json!({"state_dir": path, "version": "V0.700"});
    let tmp = format!("{}.tmp", state.config_path);
    if let Ok(mut f) = std::fs::File::create(&tmp) {
        if serde_json::to_writer_pretty(&mut f, &config).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return json!({"ok": false, "error": "Failed to write config"});
        }
        if std::fs::rename(&tmp, &state.config_path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return json!({"ok": false, "error": "Failed to save config"});
        }
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

// ─── Templates ─────────────────────────────────────────

#[tauri::command]
pub fn get_templates(state: State<AppState>) -> Value {
    let tl = &state.template_loader;
    let avatars: Vec<Value> = tl.avatar_names.iter().map(|(id, name)| {
        let camp_id = tl.avatar_camp(*id);
        json!({
            "id": id,
            "name": name,
            "rarity": tl.avatar_rarity(*id),
            "camp_id": camp_id,
            "camp_name": tl.avatar_camp_names.get(&camp_id).cloned().unwrap_or_else(|| format!("营{}", camp_id)),
        })
    }).collect();

    let weapons: Vec<Value> = tl.weapon_names.iter().map(|(id, name)| {
        json!({
            "id": id,
            "name": name,
            "rarity": "A",
            "profession": tl.weapon_professions.get(id).cloned().unwrap_or_default(),
            "max_star": tl.weapon_max_star(*id),
            "max_refine": tl.weapon_max_refine(*id),
        })
    }).collect();

    // Profession names
    let profession_names: serde_json::Map<String, Value> = tl.avatar_professions.iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();

    // Stat names
    let stat_names: serde_json::Map<String, Value> = tl.stat_names.iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();

    // Suit groups
    let mut suit_groups = serde_json::Map::new();
    for (suit_type, name) in &tl.suit_names {
        let en_name = tl.suit_en.get(suit_type).cloned().unwrap_or_default();
        // Find equip IDs belonging to this suit, include id + slot + slot_name
        let slots: Vec<Value> = tl.equip_suit_types.iter()
            .filter(|(_, &st)| st == *suit_type)
            .map(|(equip_id, _)| {
                let slot = tl.equip_slot(*equip_id);
                json!({
                    "id": equip_id,
                    "slot": slot,
                    "slot_name": slot_name(slot),
                })
            })
            .collect();
        suit_groups.insert(suit_type.to_string(), json!({
            "suit_type": suit_type,
            "suit_name": name,
            "suit_en_name": en_name,
            "slots": slots,
        }));
    }

    // Main stat options (slots 1-3 have fixed stats, 4-6 have selectable)
    let main_stat_options = json!({
        "1": [{"key": 11103, "name": "生命值"}],
        "2": [{"key": 12103, "name": "攻击力"}],
        "3": [{"key": 13103, "name": "防御力"}],
        "4": [
            {"key": 11102, "name": "生命值%"}, {"key": 12102, "name": "攻击力%"},
            {"key": 13102, "name": "防御力%"}, {"key": 20103, "name": "暴击率"},
            {"key": 21103, "name": "暴击伤害"}, {"key": 23103, "name": "穿透率"},
            {"key": 31203, "name": "异常精通"},
        ],
        "5": [
            {"key": 11102, "name": "生命值%"}, {"key": 12102, "name": "攻击力%"},
            {"key": 13102, "name": "防御力%"}, {"key": 20103, "name": "暴击率"},
            {"key": 21103, "name": "暴击伤害"}, {"key": 23103, "name": "穿透率"},
            {"key": 31203, "name": "异常精通"}, {"key": 30502, "name": "能量自动回复"},
        ],
        "6": [
            {"key": 11102, "name": "生命值%"}, {"key": 12102, "name": "攻击力%"},
            {"key": 13102, "name": "防御力%"}, {"key": 20103, "name": "暴击率"},
            {"key": 21103, "name": "暴击伤害"}, {"key": 23103, "name": "穿透率"},
            {"key": 23203, "name": "穿透值"}, {"key": 31203, "name": "异常精通"},
            {"key": 31402, "name": "异常掌控"}, {"key": 31503, "name": "物理伤害加成"},
            {"key": 31603, "name": "火属性伤害加成"}, {"key": 31703, "name": "冰属性伤害加成"},
            {"key": 31803, "name": "电属性伤害加成"}, {"key": 31903, "name": "以太属性伤害加成"},
        ],
    });

    let sub_stat_options = json!([
        {"key": 11102, "name": "生命值%"}, {"key": 11103, "name": "生命值"},
        {"key": 12102, "name": "攻击力%"}, {"key": 12103, "name": "攻击力"},
        {"key": 13102, "name": "防御力%"}, {"key": 13103, "name": "防御力"},
        {"key": 20103, "name": "暴击率"}, {"key": 21103, "name": "暴击伤害"},
        {"key": 23103, "name": "穿透率"}, {"key": 23203, "name": "穿透值"},
        {"key": 31203, "name": "异常精通"},
    ]);

    json!({
        "avatars": avatars,
        "weapons": weapons,
        "profession_names": profession_names,
        "suit_groups": suit_groups,
        "main_stat_options": main_stat_options,
        "sub_stat_options": sub_stat_options,
        "stat_names": stat_names,
        "fixed_main_slots": [1,2,3],
    })
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

// ─── Players ────────────────────────────────────────────

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
            if v < 0 { return json!({"ok": false, "error": "exp must be >= 0"}); }
        }
        if let Some(v) = data.get("avatar_id").and_then(|v| v.as_i64()) {
            if v < 0 { return json!({"ok": false, "error": "avatar_id must be >= 0"}); }
        }
        dm.update_basic_info(uid, &data);
        json!({"ok": true})
    })
}

// ─── Avatars ────────────────────────────────────────────

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
pub fn update_avatar(state: State<AppState>, uid: i64, avatar_id: i64, data: BTreeMap<String, ZonValue>) -> Value {
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
        dm.update_avatar(uid, avatar_id, &data);
        json!({"ok": true})
    })
}

// ─── Weapons ────────────────────────────────────────────

#[tauri::command]
pub fn get_weapons(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        let tl = &state.template_loader;
        let weapons: Vec<Value> = dm.list_weapons(uid).iter().filter_map(|wid| {
            let w = dm.get_weapon(uid, *wid)?;
            let item_id = zon_int(&w, "id", 0);
            Some(json!({
                "uid": *wid,
                "id": item_id,
                "name": tl.weapon_name(item_id),
                "en_name": tl.weapon_en_name(item_id),
                "profession": tl.weapon_profession(item_id),
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
                json!({
                    "uid": weapon_uid,
                    "id": item_id,
                    "name": tl.weapon_name(item_id),
                    "en_name": tl.weapon_en_name(item_id),
                    "profession": tl.weapon_profession(item_id),
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
pub fn update_weapon(state: State<AppState>, uid: i64, weapon_uid: i64, data: BTreeMap<String, ZonValue>) -> Value {
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
        dm.update_weapon(uid, weapon_uid, &data);
        json!({"ok": true})
    })
}

// ─── Equips ─────────────────────────────────────────────

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
        clean_equip_data(&mut data);
        dm.update_equip(uid, equip_uid, &data);
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
        clean_equip_data(&mut data);
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

// ─── Hadal Zone ─────────────────────────────────────────

#[tauri::command]
pub fn get_hadal_zone(state: State<AppState>, uid: i64) -> Value {
    with_manager(&state, |dm| {
        match dm.get_hadal_zone(uid) {
            Some(hz) => json!({
                "entrances": hz.get("entrances").map(|v| serde_json::to_value(v).unwrap_or_default()).unwrap_or_default(),
            }),
            None => json!(null),
        }
    })
}

#[tauri::command]
pub fn update_hadal_zone(state: State<AppState>, uid: i64, data: BTreeMap<String, ZonValue>) -> Value {
    with_manager(&state, |dm| {
        dm.update_hadal_zone(uid, &data);
        json!({"ok": true})
    })
}

// ─── Quick Launch ───────────────────────────────────────

#[tauri::command]
pub fn get_launch_config(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    json!(config.get("launch").cloned().unwrap_or(json!({})))
}

#[tauri::command]
pub fn set_launch_path(state: State<AppState>, key: String, path: String) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let mut config_map = config.as_object().cloned().unwrap_or_default();
    let mut launch = config_map.get("launch").cloned().unwrap_or(json!({}));
    if let Some(launch_obj) = launch.as_object_mut() {
        launch_obj.insert(key, json!(path));
    }
    config_map.insert("launch".to_string(), launch);
    let tmp = format!("{}.tmp", state.config_path);
    if let Ok(mut f) = std::fs::File::create(&tmp) {
        if serde_json::to_writer_pretty(&mut f, &config_map).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return json!({"ok": false, "error": "Failed to write config"});
        }
        if std::fs::rename(&tmp, &state.config_path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return json!({"ok": false, "error": "Failed to save config"});
        }
    }
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_program(path: String) -> Value {
    match std::process::Command::new(&path).spawn() {
        Ok(_) => json!({"ok": true}),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

#[tauri::command]
pub fn launch_program_admin(path: String) -> Value {
    // Validate path contains no shell metacharacters
    if path.contains(|c: char| c == '\'' || c == '"' || c == ';' || c == '|' || c == '&' || c == '`') {
        return json!({"ok": false, "error": "Invalid characters in path"});
    }
    match std::process::Command::new("powershell")
        .args(&["-Command", &format!("Start-Process -FilePath '{}' -Verb RunAs", path)])
        .spawn()
    {
        Ok(_) => json!({"ok": true}),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

#[tauri::command]
pub fn launch_yoshunko(state: State<AppState>) -> Value {
    // Extract WSL distro from state_dir path
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let state_dir = config.get("state_dir").and_then(|v| v.as_str()).unwrap_or("");

    // Try to extract distro name from path like \\wsl.localhost\UbuntuZZZ\...
    let distro = if state_dir.contains("wsl.localhost") {
        state_dir.split('\\').filter(|s| !s.is_empty()).nth(2).unwrap_or("Ubuntu")
    } else {
        "Ubuntu"
    };

    let cmd = format!(
        "cd /root/yoshunko && (zig build run-dpsv &) && sleep 2 && zig build run-gamesv"
    );

    match std::process::Command::new("wsl")
        .args(&["-u", "root", "-d", distro, "-e", "bash", "-c", &cmd])
        .spawn()
    {
        Ok(_) => json!({"ok": true, "distro": distro}),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

// ─── Zon helpers ────────────────────────────────────────

fn zon_str(obj: &BTreeMap<String, ZonValue>, key: &str, default: &str) -> String {
    obj.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn zon_int(obj: &BTreeMap<String, ZonValue>, key: &str, default: i64) -> i64 {
    obj.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

fn zon_bool(obj: &BTreeMap<String, ZonValue>, key: &str, default: bool) -> bool {
    obj.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

fn extract_skills(av: &BTreeMap<String, ZonValue>) -> Value {
    let skills: Vec<Value> = match av.get("skill_type_level") {
        Some(ZonValue::Array(arr)) => arr.iter().filter_map(|s| match s {
            ZonValue::Object(obj) => Some(json!({
                "type": zon_str(obj, "type", "common_attack"),
                "level": zon_int(obj, "level", 1),
            })),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    };
    Value::Array(skills)
}

fn slot_name(slot: i64) -> String {
    match slot {
        1 => "I", 2 => "II", 3 => "III", 4 => "IV", 5 => "V", 6 => "VI",
        _ => "?",
    }.to_string()
}

fn extract_talent_switches(av: &BTreeMap<String, ZonValue>) -> Value {
    match av.get("talent_switch_list") {
        Some(ZonValue::Array(arr)) => {
            let switches: Vec<Value> = arr.iter().map(|v| json!(v.as_bool().unwrap_or(false))).collect();
            Value::Array(switches)
        }
        _ => json!([false, false, false, false, false, false]),
    }
}

fn extract_dressed_equip(av: &BTreeMap<String, ZonValue>) -> Value {
    match av.get("dressed_equip") {
        Some(ZonValue::Array(arr)) => {
            let equips: Vec<Value> = arr.iter().map(|v| match v {
                ZonValue::Null => json!(null),
                ZonValue::Int(n) => json!(n),
                _ => json!(null),
            }).collect();
            Value::Array(equips)
        }
        _ => json!([null, null, null, null, null, null]),
    }
}

fn extract_show_weapon_type(av: &BTreeMap<String, ZonValue>) -> Value {
    match av.get("show_weapon_type") {
        Some(ZonValue::Enum(e)) => json!(e.0),
        Some(ZonValue::String(s)) => json!(s),
        _ => json!("active"),
    }
}

fn extract_equip_properties(e: &BTreeMap<String, ZonValue>, key: &str) -> Value {
    match e.get(key) {
        Some(ZonValue::Array(arr)) => {
            let props: Vec<Value> = arr.iter().filter_map(|v| match v {
                ZonValue::Object(obj) => Some(json!({
                    "key": zon_int(obj, "key", 0),
                    "key_name": "",
                    "base_value": zon_int(obj, "base_value", 0),
                    "add_value": zon_int(obj, "add_value", 0),
                })),
                ZonValue::Null => None,
                _ => None,
            }).collect();
            Value::Array(props)
        }
        _ => json!([]),
    }
}

/// Strip frontend-only fields (key_name) and null entries from equip data
/// before writing to ZON, matching Python's _dict_to_equip_data behavior.
fn clean_equip_data(data: &mut BTreeMap<String, ZonValue>) {
    // Clean properties: remove key_name from each entry
    if let Some(ZonValue::Array(props)) = data.get_mut("properties") {
        for p in props.iter_mut() {
            if let ZonValue::Object(obj) = p {
                obj.remove("key_name");
            }
        }
    }
    // Clean sub_properties: remove key_name from each entry, filter out nulls
    if let Some(ZonValue::Array(subs)) = data.get_mut("sub_properties") {
        for s in subs.iter_mut() {
            if let ZonValue::Object(obj) = s {
                obj.remove("key_name");
            }
        }
        // Remove null entries (unfilled sub-property slots)
        subs.retain(|s| !matches!(s, ZonValue::Null));
    }
}
