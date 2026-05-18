// Tauri command handlers — JS Bridge API
// Returns data directly (no wrapper), matching Python api.py behavior

use crate::data_manager::DataManager;
use crate::template_loader::TemplateLoader;
use crate::zon::ZonValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
use tauri::State;


#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct AppState {
    pub data_manager: Mutex<Option<DataManager>>,
    pub template_loader: TemplateLoader,
    pub config_path: String,
    pub cached_templates: std::sync::OnceLock<Value>,
    pub log_manager: crate::log_manager::LogManager,
    pub running_processes: std::sync::Arc<Mutex<std::collections::HashMap<String, u32>>>,
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
        Err(format!("{} 必须在 {} 到 {} 之间，当前值: {}", name, min, max, value))
    } else {
        Ok(())
    }
}

fn format_version() -> String {
    let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let version = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| {
            let v: serde_json::Value = serde_json::from_str(&s).ok()?;
            v.get("version")?.as_str().map(|s| s.to_string())
        })
        .unwrap_or_else(|| "0.0.0".to_string());
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0];
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        format!("V{}.{:03}", major, minor)
    } else {
        format!("V{}", version)
    }
}

// Helper: call f with mutable DataManager reference, return error JSON on failure
fn with_manager<F>(state: &AppState, f: F) -> Value
where
    F: FnOnce(&mut DataManager) -> Value,
{
    match state.data_manager.lock() {
        Ok(mut guard) => match guard.as_mut() {
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
        "version": format_version(),
        "config_exists": std::fs::metadata(&state.config_path).is_ok(),
        "launch_config": launch_config
    })
}

#[tauri::command]
pub fn get_version() -> Value {
    json!({"version": format_version()})
}

/// 原子写入配置文件：tmp + write + sync + rename
fn atomic_write_config(config_path: &str, config: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let tmp = format!("{}.tmp", config_path);
    let mut f = std::fs::File::create(&tmp).map_err(|e| format!("创建临时文件失败: {}", e))?;
    serde_json::to_writer_pretty(&mut f, config).map_err(|e| format!("写入配置失败: {}", e))?;
    f.sync_all().map_err(|e| format!("同步磁盘失败: {}", e))?;
    std::fs::rename(&tmp, config_path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(())
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
    // Read existing config to preserve launch fields
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
    if let Some(cached) = state.cached_templates.get() {
        return cached.clone();
    }
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
        let rarity = if *id >= 14000 { "S" } else if *id >= 13000 { "A" } else { "B" };
        json!({
            "id": id,
            "name": name,
            "rarity": rarity,
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

    // Suit groups (S-rank only, 6 positions)
    // ID formula: suit_prefix * 100 + 40 + position (S-rank)
    let mut suit_groups = serde_json::Map::new();
    for (suit_type, name) in &tl.suit_names {
        let en_name = tl.suit_en.get(suit_type).cloned().unwrap_or_default();
        let suit_prefix = suit_type / 100;
        let mut slots = Vec::new();
        for pos in 1..=6 {
            let item_id = suit_prefix * 100 + 40 + pos;  // S-rank
            slots.push(json!({
                "id": item_id,
                "slot": pos,
                "slot_name": slot_name(pos),
            }));
        }
        suit_groups.insert(suit_type.to_string(), json!({
            "suit_type": suit_type,
            "suit_name": name,
            "suit_en_name": en_name,
            "slots": slots,
        }));
    }

    // Main stat options (slots 1-3 have fixed stats, 4-6 have selectable)
    // Reference: Python version MAIN_STAT_OPTIONS
    let main_stat_options = json!({
        "1": [{"key": 11103, "name": "生命值", "base_value": 550}],
        "2": [{"key": 12103, "name": "攻击力", "base_value": 79}],
        "3": [{"key": 13103, "name": "防御力", "base_value": 46}],
        "4": [
            {"key": 11102, "name": "生命值%", "base_value": 750},
            {"key": 12102, "name": "攻击力%", "base_value": 750},
            {"key": 13102, "name": "防御力%", "base_value": 1200},
            {"key": 31203, "name": "异常精通", "base_value": 23},
            {"key": 21103, "name": "暴击伤害", "base_value": 1200},
            {"key": 20103, "name": "暴击率", "base_value": 600},
        ],
        "5": [
            {"key": 11102, "name": "生命值%", "base_value": 750},
            {"key": 12102, "name": "攻击力%", "base_value": 750},
            {"key": 13102, "name": "防御力%", "base_value": 1200},
            {"key": 23103, "name": "穿透率", "base_value": 600},
            {"key": 31503, "name": "物理伤害加成", "base_value": 750},
            {"key": 31603, "name": "火属性伤害加成", "base_value": 750},
            {"key": 31703, "name": "冰属性伤害加成", "base_value": 750},
            {"key": 31803, "name": "电属性伤害加成", "base_value": 750},
            {"key": 31903, "name": "以太属性伤害加成", "base_value": 750},
        ],
        "6": [
            {"key": 11102, "name": "生命值%", "base_value": 750},
            {"key": 12102, "name": "攻击力%", "base_value": 750},
            {"key": 13102, "name": "防御力%", "base_value": 1200},
            {"key": 31402, "name": "异常掌控", "base_value": 750},
            {"key": 12202, "name": "冲击力", "base_value": 450},
            {"key": 30502, "name": "能量自动回复", "base_value": 1500},
        ],
    });

    // Sub stat options with base_value
    // Reference: Python version SUB_STAT_OPTIONS
    let sub_stat_options = json!([
        {"key": 11103, "name": "生命值", "base_value": 112},
        {"key": 11102, "name": "生命值%", "base_value": 300},
        {"key": 12103, "name": "攻击力", "base_value": 19},
        {"key": 12102, "name": "攻击力%", "base_value": 300},
        {"key": 13103, "name": "防御力", "base_value": 15},
        {"key": 13102, "name": "防御力%", "base_value": 480},
        {"key": 23203, "name": "穿透值", "base_value": 9},
        {"key": 31203, "name": "异常精通", "base_value": 9},
        {"key": 21103, "name": "暴击伤害", "base_value": 480},
        {"key": 20103, "name": "暴击率", "base_value": 240},
    ]);

    let result = json!({
        "avatars": avatars,
        "weapons": weapons,
        "profession_names": profession_names,
        "suit_groups": suit_groups,
        "main_stat_options": main_stat_options,
        "sub_stat_options": sub_stat_options,
        "stat_names": stat_names,
        "fixed_main_slots": [1,2,3],
    });
    let _ = state.cached_templates.set(result.clone());
    result
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
            if v < 0 { return json!({"ok": false, "error": "经验值不能为负数"}); }
        }
        if let Some(v) = data.get("avatar_id").and_then(|v| v.as_i64()) {
            if v < 0 { return json!({"ok": false, "error": "角色 ID 不能为负数"}); }
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

// ─── Weapons ────────────────────────────────────────────

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

// ─── Hadal Zone ─────────────────────────────────────────

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

// ─── Quick Launch ───────────────────────────────────────

#[tauri::command]
pub fn get_launch_config(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    json!({"config": config.get("launch").cloned().unwrap_or(json!({}))})
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
    if let Err(e) = atomic_write_config(&state.config_path, &config_map) {
        return json!({"ok": false, "error": e});
    }
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_program(state: State<AppState>, key: String) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let path = config.get("launch")
        .and_then(|l| l.get(&key))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if path.is_empty() {
        return json!({"ok": false, "error": format!("路径未配置: {}", key)});
    }
    let p = std::path::Path::new(path);
    if !p.exists() {
        return json!({"ok": false, "error": format!("文件不存在: {}", path)});
    }
    let lm = &state.log_manager;
    let (log_file, _log_name) = lm.create_log(&key);
    let cwd = p.parent().map(|d| d.to_path_buf());
    let mut cmd = std::process::Command::new(path);
    if let Some(dir) = cwd { cmd.current_dir(dir); }
    if let Ok(log_file2) = log_file.try_clone() {
        use std::process::Stdio;
        cmd.stdout(Stdio::from(log_file));
        cmd.stderr(Stdio::from(log_file2));
    }
    #[cfg(windows)]
    { cmd.creation_flags(CREATE_NO_WINDOW); }
    match cmd.spawn() {
        Ok(child) => {
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert(key, child.id());
            }
            json!({"ok": true})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

#[tauri::command]
pub fn launch_program_admin(state: State<AppState>, path: String, key: String) -> Value {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return json!({"ok": false, "error": format!("文件不存在: {}", path)});
    }
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let file_w: Vec<u16> = OsStr::new(&path).encode_wide().chain(Some(0)).collect();
        let verb_w: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
        let cwd = p.parent().map(|d| {
            let w: Vec<u16> = OsStr::new(d).encode_wide().chain(Some(0)).collect();
            w
        });
        let cwd_ptr = cwd.as_ref().map(|w| w.as_ptr()).unwrap_or(std::ptr::null());

        let result = unsafe {
            ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), file_w.as_ptr(), std::ptr::null(), cwd_ptr, SW_SHOWNORMAL)
        };
        // ShellExecuteW returns a value > 32 on success
        let result_val = result as isize;
        if result_val > 32 {
            // Mark as running immediately (admin process may not be visible to us)
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert(key.clone(), 1);
            }
            json!({"ok": true})
        } else {
            json!({"ok": false, "error": format!("ShellExecuteW failed: code {}", result_val)})
        }
    }
    #[cfg(not(windows))]
    {
        match std::process::Command::new("pkexec").arg(&path).spawn() {
            Ok(_) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        }
    }
}

#[tauri::command]
pub fn launch_yoshunko(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let state_dir = config.get("state_dir").and_then(|v| v.as_str()).unwrap_or("");

    if state_dir.is_empty() {
        return json!({"ok": false, "error": "未配置状态目录"});
    }

    // Extract distro and WSL path from \\wsl.localhost\<Distro>\<path>\state
    let normalized = state_dir.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    let distro = parts.iter().position(|&s| s == "wsl.localhost")
        .and_then(|i| parts.get(i + 1))
        .copied()
        .unwrap_or("Ubuntu");
    // Build WSL path: join parts after distro and strip trailing /state
    let wsl_path = if let Some(wsl_idx) = parts.iter().position(|&s| s == "wsl.localhost") {
        if wsl_idx + 2 < parts.len() {
            let joined = format!("/{}", parts[(wsl_idx + 2)..].join("/"));
            joined.strip_suffix("/state").unwrap_or(&joined).to_string()
        } else {
            "/root/yoshunko".to_string()
        }
    } else {
        "/root/yoshunko".to_string()
    };

    let cmd = format!(
        "cd {} && (zig build run-dpsv &) && sleep 2 && zig build run-gamesv", wsl_path
    );

    let lm = &state.log_manager;
    let (log_file, _log_name) = lm.create_log("yoshunko");

    let mut cmd_proc = std::process::Command::new("wsl");
    cmd_proc.args(&["-u", "root", "-d", distro, "-e", "bash", "-c", &cmd]);
    if let Ok(log_file2) = log_file.try_clone() {
        use std::process::Stdio;
        cmd_proc.stdout(Stdio::from(log_file));
        cmd_proc.stderr(Stdio::from(log_file2));
    }
    #[cfg(windows)]
    { cmd_proc.creation_flags(CREATE_NO_WINDOW); }
    match cmd_proc.spawn() {
        Ok(child) => {
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert("yoshunko".to_string(), child.id());
            }
            json!({"ok": true, "distro": distro})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

/// Check if a process with the given name is running.
#[cfg(windows)]
fn is_process_running(name: &str) -> bool {
    std::process::Command::new("tasklist")
        .args(&["/FI", &format!("IMAGENAME eq {}", name), "/NH"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.to_lowercase().contains(&name.to_lowercase())
        })
        .unwrap_or(false)
}

// ─── Process Management ─────────────────────────────────

#[tauri::command]
pub fn get_running_processes(state: State<AppState>) -> Value {
    let mut procs = state.running_processes.lock().unwrap_or_else(|e| e.into_inner());
    // Verify processes are still alive (skip client which uses sentinel PID)
    procs.retain(|key, pid| {
        if *pid == 1 {
            // Client sentinel: assume running until explicitly stopped
            key == "client"
        } else {
            std::process::Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                .unwrap_or(false)
        }
    });
    let map: serde_json::Map<String, Value> = procs.iter()
        .map(|(k, &pid)| (k.clone(), json!(pid)))
        .collect();
    json!({"processes": map})
}

#[tauri::command]
pub fn stop_process(state: State<AppState>, key: String) -> Value {
    let pid = {
        let procs = state.running_processes.lock().unwrap_or_else(|e| e.into_inner());
        procs.get(&key).copied()
    };
    #[cfg(windows)]
    {
        if key == "client" || key == "gale" || key == "velina" {
            // Game runs as admin — use ShellExecuteW(runas) to kill it
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

            let file_w: Vec<u16> = OsStr::new("taskkill").encode_wide().chain(Some(0)).collect();
            let verb_w: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
            let args_w: Vec<u16> = OsStr::new("/IM ZenlessZoneZeroBeta.exe /F /T").encode_wide().chain(Some(0)).collect();
            unsafe {
                ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), file_w.as_ptr(), args_w.as_ptr(), std::ptr::null(), SW_HIDE);
            }
        } else if let Some(pid) = pid {
            let _ = std::process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F", "/T"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(pid) = pid {
            let _ = std::process::Command::new("kill").arg(pid.to_string()).spawn();
        }
    }
    if let Ok(mut procs) = state.running_processes.lock() {
        procs.remove(&key);
    }
    json!({"ok": true})
}

// ─── Log ───────────────────────────────────────────────

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
