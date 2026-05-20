// Shared utilities — validation, version formatting, ZON extraction, config I/O

use crate::data_manager::DataManager;
use crate::zon::ZonValue;
use crate::AppState;
use serde_json::{json, Value};
use std::collections::BTreeMap;

// ─── Validation constants ──────────────────────────────────

pub const MIN_LEVEL: i64 = 1;
pub const MAX_LEVEL: i64 = 60;
pub const MIN_STAR: i64 = 1;
pub const MAX_STAR: i64 = 5;
pub const MIN_REFINE: i64 = 1;
pub const MAX_REFINE: i64 = 5;
pub const MIN_RANK: i64 = 0;
pub const MAX_RANK: i64 = 6;
pub const MIN_PASSIVE: i64 = 0;
pub const MAX_PASSIVE: i64 = 6;
pub const MIN_EQUIP_LEVEL: i64 = 0;
pub const MAX_EQUIP_LEVEL: i64 = 15;
pub const MAX_EQUIP_STAR: i64 = 5;

pub fn check_range(value: i64, min: i64, max: i64, name: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{} 必须在 {} 到 {} 之间，当前值: {}", name, min, max, value))
    } else {
        Ok(())
    }
}

pub fn format_version() -> String {
    // Try exe directory first (release builds bundle config there),
    // fall back to CARGO_MANIFEST_DIR (dev mode)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let config_path = exe_dir
        .as_ref()
        .map(|d| d.join("tauri.conf.json"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json")
        });
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

pub fn with_manager<F>(state: &AppState, f: F) -> Value
where
    F: FnOnce(&mut DataManager) -> Value,
{
    let mut guard = state.data_manager.lock().unwrap_or_else(|poisoned| {
        eprintln!("[with_manager] Mutex was poisoned, recovering...");
        poisoned.into_inner()
    });
    match guard.as_mut() {
        Some(dm) => f(dm),
        None => json!({"ok": false, "error": "状态目录未配置"}),
    }
}

pub fn atomic_write_config(config_path: &str, config: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let tmp = format!("{}.tmp", config_path);
    let mut f = std::fs::File::create(&tmp).map_err(|e| format!("创建临时文件失败: {}", e))?;
    serde_json::to_writer_pretty(&mut f, config).map_err(|e| format!("写入配置失败: {}", e))?;
    f.sync_all().map_err(|e| format!("同步磁盘失败: {}", e))?;
    std::fs::rename(&tmp, config_path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(())
}

// ─── Zon helpers ───────────────────────────────────────────

pub fn zon_str(obj: &BTreeMap<String, ZonValue>, key: &str, default: &str) -> String {
    obj.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

pub fn zon_int(obj: &BTreeMap<String, ZonValue>, key: &str, default: i64) -> i64 {
    obj.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

pub fn zon_bool(obj: &BTreeMap<String, ZonValue>, key: &str, default: bool) -> bool {
    obj.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

pub fn extract_skills(av: &BTreeMap<String, ZonValue>) -> Value {
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

pub fn slot_name(slot: i64) -> String {
    match slot {
        1 => "I", 2 => "II", 3 => "III", 4 => "IV", 5 => "V", 6 => "VI",
        _ => "?",
    }.to_string()
}

pub fn extract_talent_switches(av: &BTreeMap<String, ZonValue>) -> Value {
    match av.get("talent_switch_list") {
        Some(ZonValue::Array(arr)) => {
            let switches: Vec<Value> = arr.iter().map(|v| json!(v.as_bool().unwrap_or(false))).collect();
            Value::Array(switches)
        }
        _ => json!([false, false, false, false, false, false]),
    }
}

pub fn extract_dressed_equip(av: &BTreeMap<String, ZonValue>) -> Value {
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

pub fn extract_show_weapon_type(av: &BTreeMap<String, ZonValue>) -> Value {
    match av.get("show_weapon_type") {
        Some(ZonValue::Enum(e)) => json!(e.0),
        Some(ZonValue::String(s)) => json!(s),
        _ => json!("active"),
    }
}

pub fn extract_equip_properties(e: &BTreeMap<String, ZonValue>, key: &str) -> Value {
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
pub fn clean_equip_data(data: &mut BTreeMap<String, ZonValue>) {
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

/// Validate avatar data structure before writing.
/// Returns Ok(()) or Err with a descriptive message.
pub fn validate_avatar_data(data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
    if let Some(ZonValue::Array(skills)) = data.get("skill_type_level") {
        for (i, skill) in skills.iter().enumerate() {
            match skill {
                ZonValue::Object(obj) => {
                    if !obj.contains_key("type") {
                        return Err(format!("skill_type_level[{}] 缺少 type 字段", i));
                    }
                    if !obj.contains_key("level") {
                        return Err(format!("skill_type_level[{}] 缺少 level 字段", i));
                    }
                }
                other => return Err(format!("skill_type_level[{}] 应为对象，实际为 {:?}", i, other)),
            }
        }
    }
    Ok(())
}

/// Validate equip data structure before writing.
/// Returns Ok(()) or Err with a descriptive message.
pub fn validate_equip_data(data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
    for key in &["properties", "sub_properties"] {
        if let Some(ZonValue::Array(props)) = data.get(*key) {
            for (i, prop) in props.iter().enumerate() {
                match prop {
                    ZonValue::Null => {} // allowed before cleaning
                    ZonValue::Object(obj) => {
                        if !obj.contains_key("key") {
                            return Err(format!("{}[{}] 缺少 key 字段", key, i));
                        }
                    }
                    other => return Err(format!("{}[{}] 应为对象，实际为 {:?}", key, i, other)),
                }
            }
        }
    }
    Ok(())
}
