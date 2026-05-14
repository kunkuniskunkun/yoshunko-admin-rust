// Template Loader — loads game template JSON data
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TemplateLoader {
    pub avatar_names: HashMap<i64, String>,
    pub avatar_rarity: HashMap<i64, String>,
    pub avatar_camps: HashMap<i64, i64>,
    pub avatar_camp_names: HashMap<i64, String>,
    pub avatar_en: HashMap<i64, String>,
    pub avatar_professions: HashMap<i64, String>,
    pub weapon_names: HashMap<i64, String>,
    pub weapon_en: HashMap<i64, String>,
    pub weapon_professions: HashMap<i64, String>,
    pub weapon_star_limit: HashMap<i64, i64>,
    pub weapon_refine_limit: HashMap<i64, i64>,
    pub equip_suit_types: HashMap<i64, i64>,
    pub equip_types: HashMap<i64, i64>,
    pub suit_names: HashMap<i64, String>,
    pub suit_chinese: HashMap<i64, String>,
    pub suit_en: HashMap<i64, String>,
    pub stat_names: HashMap<i64, String>,
    data_dir: PathBuf,
}

fn load_json_kv(filename: &PathBuf, target: &mut HashMap<i64, String>) {
    if let Ok(content) = fs::read_to_string(filename) {
        if let Ok(map) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = map.as_object() {
                for (k, v) in obj {
                    if let Ok(id) = k.parse::<i64>() {
                        match v {
                            serde_json::Value::String(s) => { target.insert(id, s.clone()); }
                            serde_json::Value::Number(n) => { target.insert(id, n.to_string()); }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn load_json_kv_i64(filename: &PathBuf, target: &mut HashMap<i64, i64>) {
    if let Ok(content) = fs::read_to_string(filename) {
        if let Ok(map) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = map.as_object() {
                for (k, v) in obj {
                    if let (Ok(id), Some(n)) = (k.parse::<i64>(), v.as_i64()) {
                        target.insert(id, n);
                    }
                }
            }
        }
    }
}

impl TemplateLoader {
    pub fn new(data_dir: PathBuf) -> Self {
        let mut avatar_names = HashMap::new();
        let mut avatar_rarity = HashMap::new();
        let mut avatar_camps = HashMap::new();
        let mut avatar_camp_names = HashMap::new();
        let mut avatar_en = HashMap::new();
        let mut avatar_professions = HashMap::new();
        let mut weapon_names = HashMap::new();
        let mut weapon_en = HashMap::new();
        let mut weapon_professions = HashMap::new();
        let mut suit_chinese = HashMap::new();
        let mut suit_en = HashMap::new();
        let mut stat_names = HashMap::new();

        load_json_kv(&data_dir.join("avatar_names.json"), &mut avatar_names);
        load_json_kv(&data_dir.join("avatar_rarity.json"), &mut avatar_rarity);
        load_json_kv_i64(&data_dir.join("avatar_camps_override.json"), &mut avatar_camps);
        load_json_kv(&data_dir.join("camps.json"), &mut avatar_camp_names);
        load_json_kv(&data_dir.join("en/avatar_en.json"), &mut avatar_en);
        load_json_kv(&data_dir.join("avatar_professions.json"), &mut avatar_professions);
        load_json_kv(&data_dir.join("weapon_names_zh.json"), &mut weapon_names);
        load_json_kv(&data_dir.join("en/weapon_en.json"), &mut weapon_en);
        load_json_kv(&data_dir.join("weapon_professions.json"), &mut weapon_professions);
        load_json_kv(&data_dir.join("equip_suit_names_zh.json"), &mut suit_chinese);
        load_json_kv(&data_dir.join("en/suit_en.json"), &mut suit_en);
        load_json_kv(&data_dir.join("equip_stat_names.json"), &mut stat_names);

        let mut tl = TemplateLoader {
            avatar_names, avatar_rarity, avatar_camps, avatar_camp_names,
            avatar_en, avatar_professions,
            weapon_names, weapon_en, weapon_professions,
            weapon_star_limit: HashMap::new(),
            weapon_refine_limit: HashMap::new(),
            equip_suit_types: HashMap::new(),
            equip_types: HashMap::new(),
            suit_names: HashMap::new(),
            suit_chinese, suit_en, stat_names,
            data_dir,
        };
        tl.load_templates();
        tl
    }

    fn load_templates(&mut self) {
        let path = self.data_dir.join("templates/AvatarBaseTemplateTb.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(items) = val.get("data").and_then(|d| d.as_array()) {
                    for item in items {
                        if let (Some(id), Some(name)) = (item.get("id").and_then(|v| v.as_i64()), item.get("code_name").and_then(|v| v.as_str())) {
                            self.avatar_names.entry(id).or_insert_with(|| name.to_string());
                            if let Some(camp) = item.get("camp").and_then(|v| v.as_i64()) {
                                self.avatar_camps.entry(id).or_insert(camp);
                            }
                        }
                    }
                }
            }
        }

        let path = self.data_dir.join("templates/WeaponTemplateTb.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(items) = val.get("data").and_then(|d| d.as_array()) {
                    for item in items {
                        if let Some(item_id) = item.get("item_id").and_then(|v| v.as_i64()) {
                            if let Some(name) = item.get("weapon_name").and_then(|v| v.as_str()) {
                                self.weapon_names.entry(item_id).or_insert_with(|| name.to_string());
                            }
                            if let Some(star) = item.get("star_limit").and_then(|v| v.as_i64()) {
                                self.weapon_star_limit.insert(item_id, star);
                            }
                            if let Some(refine) = item.get("refine_limit").and_then(|v| v.as_i64()) {
                                self.weapon_refine_limit.insert(item_id, refine);
                            }
                        }
                    }
                }
            }
        }

        let path = self.data_dir.join("templates/EquipmentTemplateTb.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(items) = val.get("data").and_then(|d| d.as_array()) {
                    for item in items {
                        if let Some(item_id) = item.get("item_id").and_then(|v| v.as_i64()) {
                            if let Some(st) = item.get("suit_type").and_then(|v| v.as_i64()) {
                                self.equip_suit_types.insert(item_id, st);
                            }
                            if let Some(et) = item.get("equipment_type").and_then(|v| v.as_i64()) {
                                self.equip_types.insert(item_id, et);
                            }
                        }
                    }
                }
            }
        }

        let path = self.data_dir.join("templates/EquipmentSuitTemplateTb.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(items) = val.get("data").and_then(|d| d.as_array()) {
                    for item in items {
                        if let Some(id) = item.get("id").and_then(|v| v.as_i64()) {
                            // Use Chinese name from suit_chinese, fall back to JSON name field
                            let cn = self.suit_chinese.get(&id).cloned()
                                .or_else(|| item.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                                .unwrap_or_else(|| format!("Suit_{}", id));
                            self.suit_names.insert(id, cn);
                        }
                    }
                }
            }
        }
    }

    pub fn avatar_name(&self, id: i64) -> String {
        self.avatar_names.get(&id).cloned().unwrap_or_else(|| format!("Avatar_{}", id))
    }
    pub fn avatar_en_name(&self, id: i64) -> String {
        self.avatar_en.get(&id).cloned().unwrap_or_default()
    }
    pub fn avatar_rarity(&self, id: i64) -> String {
        self.avatar_rarity.get(&id).cloned().unwrap_or_else(|| "?".into())
    }
    pub fn avatar_camp(&self, id: i64) -> i64 {
        self.avatar_camps.get(&id).copied().unwrap_or(0)
    }
    pub fn avatar_profession(&self, id: i64) -> String {
        self.avatar_professions.get(&id).cloned().unwrap_or_default()
    }
    pub fn weapon_name(&self, id: i64) -> String {
        self.weapon_names.get(&id).cloned().unwrap_or_else(|| format!("Weapon_{}", id))
    }
    pub fn weapon_en_name(&self, id: i64) -> String {
        self.weapon_en.get(&id).cloned().unwrap_or_default()
    }
    pub fn weapon_profession(&self, id: i64) -> String {
        self.weapon_professions.get(&id).cloned().unwrap_or_default()
    }
    pub fn weapon_max_star(&self, id: i64) -> i64 {
        self.weapon_star_limit.get(&id).copied().unwrap_or(4) + 1
    }
    pub fn weapon_max_refine(&self, id: i64) -> i64 {
        self.weapon_refine_limit.get(&id).copied().unwrap_or(5)
    }
    pub fn suit_name(&self, equip_id: i64) -> String {
        let st = self.equip_suit_types.get(&equip_id).copied().unwrap_or(0);
        if st == 0 {
            return format!("装备_{}", equip_id);
        }
        self.suit_chinese.get(&st).cloned().unwrap_or_else(|| format!("Suit_{}", st))
    }
    pub fn suit_en_name(&self, equip_id: i64) -> String {
        let st = self.equip_suit_types.get(&equip_id).copied().unwrap_or(0);
        self.suit_en.get(&st).cloned().unwrap_or_default()
    }
    pub fn equip_slot(&self, equip_id: i64) -> i64 {
        self.equip_types.get(&equip_id).copied().unwrap_or(0)
    }
}
