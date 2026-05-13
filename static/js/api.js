// ═══════════════════════════════════════════════════════
// API Bridge — pywebview.api wrapper
// ═══════════════════════════════════════════════════════
var API = {
  _checkReady: function() {
    if (!window.pywebview || !window.pywebview.api) {
      console.warn('pywebview bridge not available');
      return false;
    }
    return true;
  },

  // Player
  getPlayerList: async function() { return await pywebview.api.get_player_list(); },
  getPlayer: async function(uid) {
    var r = await pywebview.api.get_player_basic(uid);
    return r;
  },
  updatePlayer: async function(uid, data) { return await pywebview.api.update_player_basic(uid, data); },

  // Templates
  getTemplates: async function() { return await pywebview.api.get_templates(); },

  // Avatars
  getAvatars: async function(uid) { return await pywebview.api.get_avatars(uid); },
  getAvatar: async function(uid, aid) {
    var r = await pywebview.api.get_avatar(uid, aid);
    if (!r) return null;
    return { avatar_id: r.avatar.avatar_id, name: r.avatar.name, en_name: r.avatar.en_name || '', rarity: r.avatar.rarity, level: r.avatar.level, unlocked_talent_num: r.avatar.unlocked_talent_num, passive_skill_level: r.avatar.passive_skill_level, skill_type_level: r.avatar.skill_type_level, awake_id: r.avatar.awake_id, cur_weapon_uid: r.avatar.cur_weapon_uid, is_favorite: r.avatar.is_favorite, forms: r.forms };
  },
  updateAvatar: async function(uid, aid, data) { return await pywebview.api.update_avatar(uid, aid, data); },

  // Weapons
  getWeapons: async function(uid) { return await pywebview.api.get_weapons(uid); },
  getWeapon: async function(uid, wid) {
    var w = await pywebview.api.get_weapon(uid, wid);
    return w;
  },
  updateWeapon: async function(uid, wid, data) { return await pywebview.api.update_weapon(uid, wid, data); },

  // Equips
  getEquips: async function(uid) { return await pywebview.api.get_equips(uid); },
  getEquip: async function(uid, eid) {
    var e = await pywebview.api.get_equip(uid, eid);
    return e;
  },
  createEquip: async function(uid, data) { return await pywebview.api.create_equip(uid, data); },
  deleteEquip: async function(uid, eid) { return await pywebview.api.delete_equip(uid, eid); },
  updateEquip: async function(uid, eid, data) { return await pywebview.api.update_equip(uid, eid, data); },

  // Hadal Zone
  getHadalZone: async function(uid) { return await pywebview.api.get_hadal_zone(uid); },
  updateHadalZone: async function(uid, data) { return await pywebview.api.update_hadal_zone(uid, data); },

  // Export/Import
  exportData: async function(uid) { return await pywebview.api.export_player_data(uid); },
  importData: async function(uid, data) { return await pywebview.api.import_player_data(uid, data); },

  // Config
  getConfig: async function() { return await pywebview.api.get_config(); },
  getVersion: async function() { return await pywebview.api.get_version(); },
  setStateDir: async function(path) { return await pywebview.api.set_state_dir(path); },
  autoDetectPaths: async function() { return await pywebview.api.auto_detect_paths(); },

  // Quick launch
  getLaunchConfig: async function() { return await pywebview.api.get_launch_config(); },
  setLaunchPath: async function(key, path) { return await pywebview.api.set_launch_path(key, path); },
  launchProgram: async function(key) { return await pywebview.api.launch_program(key); },
  launchProgramAdmin: async function(key) { return await pywebview.api.launch_program_admin(key); },
  launchYoshunko: async function() { return await pywebview.api.launch_yoshunko(); }
};
