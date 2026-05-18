use super::*;

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
