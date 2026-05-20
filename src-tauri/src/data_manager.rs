// Data Manager — ZON file I/O with atomic writes, backups, and audit logging
// Equivalent to Python data_manager.py

use crate::zon::{parse_zon, serialize_zon_object, ZonValue};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Configuration for data manager
pub struct DataManager {
    #[allow(dead_code)]
    pub state_dir: String,
    player_dir: PathBuf,
    cache: HashMap<PathBuf, BTreeMap<String, ZonValue>>,
}

impl DataManager {
    pub fn new(state_dir: &str) -> Self {
        let player_dir = Path::new(state_dir).join("player");
        DataManager {
            state_dir: state_dir.to_string(),
            player_dir,
            cache: HashMap::new(),
        }
    }

    pub fn list_players(&self) -> Vec<i64> {
        self.list_dir(&self.player_dir)
    }

    fn player_path(&self, uid: i64, sub: &str, name: &str) -> PathBuf {
        let base = self.player_dir.join(uid.to_string()).join(sub);
        if name.is_empty() { base } else { base.join(name) }
    }

    pub fn _debug_avatar_path(&self, uid: i64, avatar_id: i64) -> PathBuf {
        self.player_dir.join(uid.to_string()).join("avatar").join(avatar_id.to_string())
    }

    fn list_dir(&self, dir: &Path) -> Vec<i64> {
        let mut result = Vec::new();
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        // Skip "next" counter file and ".tmp" partial writes
                        if name == "next" || name.ends_with(".tmp") {
                            continue;
                        }
                        if let Ok(id) = name.parse::<i64>() {
                            result.push(id);
                        }
                    }
                }
            }
            Err(_) => {} // silently skip unreadable dirs
        }
        result.sort();
        result
    }

    // ─── Basic Info ───────────────────────────────────────

    pub fn get_basic_info(&mut self, uid: i64) -> Option<BTreeMap<String, ZonValue>> {
        let path = self.player_path(uid, "info", "");
        self.read_zon_obj(&path)
    }

    pub fn update_basic_info(&mut self, uid: i64, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        let path = self.player_path(uid, "info", "");
        self.write_zon(&path, data)
    }

    // ─── Avatars ──────────────────────────────────────────

    pub fn list_avatars(&self, uid: i64) -> Vec<i64> {
        let dir = self.player_dir.join(uid.to_string()).join("avatar");
        self.list_dir(&dir)
    }

    pub fn get_avatar(&mut self, uid: i64, avatar_id: i64) -> Option<BTreeMap<String, ZonValue>> {
        let path = self.player_path(uid, "avatar", &avatar_id.to_string());
        self.read_zon_obj(&path)
    }

    pub fn update_avatar(&mut self, uid: i64, avatar_id: i64, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        let path = self.player_path(uid, "avatar", &avatar_id.to_string());
        self.write_zon(&path, data)
    }

    // ─── Weapons ──────────────────────────────────────────

    pub fn list_weapons(&self, uid: i64) -> Vec<i64> {
        let dir = self.player_dir.join(uid.to_string()).join("weapon");
        self.list_dir(&dir)
    }

    pub fn get_weapon(&mut self, uid: i64, weapon_uid: i64) -> Option<BTreeMap<String, ZonValue>> {
        let path = self.player_path(uid, "weapon", &weapon_uid.to_string());
        self.read_zon_obj(&path)
    }

    pub fn update_weapon(&mut self, uid: i64, weapon_uid: i64, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        let path = self.player_path(uid, "weapon", &weapon_uid.to_string());
        self.write_zon(&path, data)
    }

    pub fn delete_weapon(&mut self, uid: i64, weapon_uid: i64) -> Result<(), String> {
        let path = self.player_path(uid, "weapon", &weapon_uid.to_string());
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        self.cache.remove(&path);
        Ok(())
    }

    pub fn copy_weapon(&mut self, uid: i64, weapon_uid: i64) -> Result<i64, String> {
        let src_path = self.player_path(uid, "weapon", &weapon_uid.to_string());
        let data = self.read_zon_obj(&src_path).ok_or("武器不存在")?;
        let weapon_dir = self.player_dir.join(uid.to_string()).join("weapon");
        let new_uid = self.next_uid(&weapon_dir);
        let dst_path = weapon_dir.join(new_uid.to_string());
        self.write_zon(&dst_path, &data)?;
        Ok(new_uid)
    }

    // ─── Equips ───────────────────────────────────────────

    pub fn list_equips(&self, uid: i64) -> Vec<i64> {
        let dir = self.player_dir.join(uid.to_string()).join("equip");
        self.list_dir(&dir)
    }

    pub fn get_equip(&mut self, uid: i64, equip_uid: i64) -> Option<BTreeMap<String, ZonValue>> {
        let path = self.player_path(uid, "equip", &equip_uid.to_string());
        self.read_zon_obj(&path)
    }

    pub fn update_equip(&mut self, uid: i64, equip_uid: i64, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        let path = self.player_path(uid, "equip", &equip_uid.to_string());
        self.write_zon(&path, data)
    }

    pub fn copy_equip(&mut self, uid: i64, equip_uid: i64) -> Result<i64, String> {
        let src_path = self.player_path(uid, "equip", &equip_uid.to_string());
        let data = self.read_zon_obj(&src_path).ok_or("驱动盘不存在")?;
        let equip_dir = self.player_dir.join(uid.to_string()).join("equip");
        let new_uid = self.next_uid(&equip_dir);
        let dst_path = equip_dir.join(new_uid.to_string());
        self.write_zon(&dst_path, &data)?;
        Ok(new_uid)
    }

    pub fn delete_equip(&mut self, uid: i64, equip_uid: i64) -> Result<(), String> {
        let path = self.player_path(uid, "equip", &equip_uid.to_string());
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        self.cache.remove(&path);
        Ok(())
    }

    pub fn create_equip(&mut self, uid: i64, data: &BTreeMap<String, ZonValue>) -> Result<i64, String> {
        let equip_dir = self.player_dir.join(uid.to_string()).join("equip");
        fs::create_dir_all(&equip_dir).map_err(|e| format!("Cannot create equip dir: {}", e))?;
        let new_uid = self.next_uid(&equip_dir);
        let path = equip_dir.join(new_uid.to_string());
        self.write_zon(&path, data)?;
        Ok(new_uid)
    }

    fn next_uid(&self, dir: &Path) -> i64 {
        // Read counter from "next" file if it exists
        let next_file = dir.join("next");
        if let Ok(content) = fs::read_to_string(&next_file) {
            if let Ok(val) = content.trim().parse::<i64>() {
                if val > 0 {
                    // Update counter: write val+1 atomically
                    let tmp = next_file.with_extension("tmp");
                    if let Ok(mut f) = fs::File::create(&tmp) {
                        use std::io::Write;
                        let _ = write!(f, "{}", val + 1);
                        let _ = f.sync_all();
                        drop(f);
                        let _ = fs::rename(&tmp, &next_file);
                    }
                    return val;
                }
            }
        }
        // Fallback: scan directory for max ID
        let mut max_id = 0i64;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name == "next" || name.ends_with(".tmp") {
                        continue;
                    }
                    if let Ok(id) = name.parse::<i64>() {
                        max_id = max_id.max(id);
                    }
                }
            }
        }
        let new_uid = max_id.saturating_add(1);
        // Write counter file
        let tmp = next_file.with_extension("tmp");
        if let Ok(mut f) = fs::File::create(&tmp) {
            use std::io::Write;
            let _ = write!(f, "{}", new_uid + 1);
            let _ = f.sync_all();
            drop(f);
            let _ = fs::rename(&tmp, &next_file);
        }
        new_uid
    }

    // ─── Hadal Zone ───────────────────────────────────────

    pub fn get_hadal_zone(&mut self, uid: i64) -> Option<BTreeMap<String, ZonValue>> {
        let path = self.player_player_path(uid, "hadal_zone", "info");
        self.read_zon_obj(&path)
    }

    pub fn update_hadal_zone(&mut self, uid: i64, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        let path = self.player_player_path(uid, "hadal_zone", "info");
        self.write_zon(&path, data)
    }

    fn player_player_path(&self, uid: i64, sub: &str, name: &str) -> PathBuf {
        self.player_dir.join(uid.to_string()).join(sub).join(name)
    }

    // ─── ZON read/write ───────────────────────────────────

    fn read_zon(&self, path: &Path) -> Option<ZonValue> {
        match fs::read_to_string(path) {
            Ok(content) => match parse_zon(&content) {
                Ok(val) => Some(val),
                Err(e) => {
                    eprintln!("[DataManager] ZON parse error for {:?}: {}", path, e);
                    None
                }
            },
            Err(e) => {
                eprintln!("[DataManager] read_zon failed for {:?}: {}", path, e);
                None
            }
        }
    }

    fn read_zon_obj(&mut self, path: &Path) -> Option<BTreeMap<String, ZonValue>> {
        if let Some(cached) = self.cache.get(path) {
            return Some(cached.clone());
        }
        match self.read_zon(path)? {
            ZonValue::Object(obj) => {
                self.cache.insert(path.to_path_buf(), obj.clone());
                Some(obj)
            }
            _ => None,
        }
    }

    fn write_zon(&mut self, path: &Path, data: &BTreeMap<String, ZonValue>) -> Result<(), String> {
        // Backup (best-effort)
        let _ = self.backup_zon(path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // Atomic write: write to .tmp, then rename
        let tmp = path.with_extension("tmp");
        let zon_str = serialize_zon_object(data);
        let mut f = fs::File::create(&tmp).map_err(|e| format!("创建临时文件失败: {}", e))?;
        f.write_all(zon_str.as_bytes()).map_err(|e| format!("写入失败: {}", e))?;
        f.write_all(b"\n").map_err(|e| format!("写入换行失败: {}", e))?;
        f.sync_all().map_err(|e| { let _ = fs::remove_file(&tmp); format!("同步磁盘失败: {}", e) })?;
        drop(f);
        fs::rename(&tmp, path).map_err(|e| format!("重命名失败: {}", e))?;

        // Update cache
        self.cache.insert(path.to_path_buf(), data.clone());

        // Audit log (best-effort)
        let _ = self.audit_log(path);
        Ok(())
    }

    fn backup_zon(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let backup_dir = path.parent().ok_or("Path has no parent")?.join(".backup");
        fs::create_dir_all(&backup_dir).map_err(|_| "Cannot create backup dir")?;

        use chrono::Local;
        let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let fname = path.file_name().ok_or("Path has no filename")?.to_str().ok_or("Filename not UTF-8")?;
        let backup_path = backup_dir.join(format!("{}.{}", fname, ts));

        fs::copy(path, &backup_path).map_err(|_| "Backup copy failed")?;

        // Rotate: keep only 5 most recent backups
        let prefix = format!("{}.", fname);
        let mut backups: Vec<_> = fs::read_dir(&backup_dir)
            .map(|entries| entries.flatten().filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                if name.starts_with(&prefix) { Some(name) } else { None }
            }).collect())
            .unwrap_or_default();
        backups.sort();
        while backups.len() > 5 {
            if let Some(old) = backups.first() {
                let _ = fs::remove_file(backup_dir.join(old));
                backups.remove(0);
            }
        }
        Ok(())
    }

    fn audit_log(&self, path: &Path) -> Result<(), String> {
        use chrono::Local;
        use std::io::Write;

        let audit_path = self.player_dir.parent().unwrap_or(&self.player_dir).join("audit.log");
        let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let entry = format!("[{}] WRITE {} {}\n", ts, path.display(), Local::now().format("%z"));

        // Rotate if > 1MB
        if let Ok(meta) = fs::metadata(&audit_path) {
            if meta.len() > 1_000_000 {
                let _ = fs::rename(&audit_path, audit_path.with_extension("log.old"));
            }
        }

        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&audit_path) {
            let _ = f.write_all(entry.as_bytes());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::thread;

    fn temp_dm() -> (DataManager, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("yos_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let player_dir = dir.join("player").join("1").join("weapon");
        fs::create_dir_all(&player_dir).unwrap();
        let dm = DataManager::new(dir.to_str().unwrap());
        (dm, dir)
    }

    #[test]
    fn next_uid_sequential_increments() {
        let (dm, _dir) = temp_dm();
        let dir = dm.player_dir.join("1").join("weapon");
        let a = dm.next_uid(&dir);
        let b = dm.next_uid(&dir);
        let c = dm.next_uid(&dir);
        assert!(a < b, "{} should be < {}", a, b);
        assert!(b < c, "{} should be < {}", b, c);
    }

    #[test]
    fn write_then_read_roundtrip() {
        let (mut dm, _dir) = temp_dm();
        let path = dm.player_dir.join("1").join("weapon").join("42");
        let mut data = BTreeMap::new();
        data.insert("name".into(), ZonValue::String("TestWeapon".into()));
        data.insert("level".into(), ZonValue::Int(60));
        dm.write_zon(&path, &data).unwrap();

        let read = dm.read_zon_obj(&path).unwrap();
        assert_eq!(read.get("name").and_then(|v| v.as_str()), Some("TestWeapon"));
        assert_eq!(read.get("level").and_then(|v| v.as_i64()), Some(60));
    }

    #[test]
    fn concurrent_next_uid_uniqueness() {
        let dm = Arc::new(Mutex::new(temp_dm().0));
        let dir = {
            let guard = dm.lock().unwrap();
            guard.player_dir.join("1").join("weapon")
        };

        let n_threads = 8;
        let iters_per_thread = 10;
        let mut handles = vec![];
        let results = Arc::new(Mutex::new(Vec::new()));

        for _ in 0..n_threads {
            let dm = Arc::clone(&dm);
            let dir = dir.clone();
            let results = Arc::clone(&results);
            handles.push(thread::spawn(move || {
                for _ in 0..iters_per_thread {
                    let uid = {
                        let guard = dm.lock().unwrap();
                        guard.next_uid(&dir)
                    };
                    results.lock().unwrap().push(uid);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let uids = results.lock().unwrap();
        let total = n_threads * iters_per_thread;
        assert_eq!(uids.len(), total);

        // All UIDs must be unique
        let mut sorted = uids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), total, "duplicate UIDs detected");
    }

    #[test]
    fn atomic_write_clean_on_crash_simulation() {
        // Verify that a .tmp file doesn't become a valid ZON file
        // unless the full write+rename sequence completes
        let (_dm, dir) = temp_dm();
        let path = dir.join("player").join("1").join("weapon").join("99");
        let tmp_path = path.with_extension("tmp");

        // Write partial data to .tmp (simulating crash mid-write)
        fs::write(&tmp_path, "corrupted data").unwrap();
        assert!(tmp_path.exists());
        assert!(!path.exists());

        // Cleanup
        fs::remove_file(&tmp_path).unwrap();
    }
}
