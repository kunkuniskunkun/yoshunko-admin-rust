// ═══════════════════════════════════════════════════════
// Tauri v2 Compatibility Layer
// Creates window.pywebview with IPC-backed methods.
// ═══════════════════════════════════════════════════════

(function() {
  var _invoke = null;

  function getInvoke() {
    if (_invoke) return _invoke;
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
      _invoke = window.__TAURI__.core.invoke.bind(window.__TAURI__.core);
    } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
      _invoke = window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
    }
    return _invoke;
  }

  function invoke(cmd, args) {
    var fn = getInvoke();
    if (!fn) return Promise.reject(new Error('Tauri IPC not available'));
    return fn(cmd, args || {});
  }

  function buildApi() {
    var api = {};
    api.get_config = function() { return invoke('get_config'); };
    api.get_version = function() { return invoke('get_version'); };
    api.set_state_dir = function(path) { return invoke('set_state_dir', { path: path }); };
    api.auto_detect_paths = function() { return invoke('auto_detect_paths'); };
    api.get_player_list = function() { return invoke('get_player_list'); };
    api.get_player_basic = function(uid) { return invoke('get_player_basic', { uid: uid }); };
    api.update_player_basic = function(uid, data) { return invoke('update_player_basic', { uid: uid, data: data }); };
    api.get_templates = function() { return invoke('get_templates'); };
    api.get_avatars = function(uid) { return invoke('get_avatars', { uid: uid }); };
    api.get_avatar = function(uid, aid) { return invoke('get_avatar', { uid: uid, avatarId: aid }); };
    api.update_avatar = function(uid, aid, data) { return invoke('update_avatar', { uid: uid, avatarId: aid, data: data }); };
    api.get_weapons = function(uid) { return invoke('get_weapons', { uid: uid }); };
    api.get_weapon = function(uid, wid) { return invoke('get_weapon', { uid: uid, weaponUid: wid }); };
    api.update_weapon = function(uid, wid, data) { return invoke('update_weapon', { uid: uid, weaponUid: wid, data: data }); };
    api.get_equips = function(uid) { return invoke('get_equips', { uid: uid }); };
    api.get_equip = function(uid, eid) { return invoke('get_equip', { uid: uid, equipUid: eid }); };
    api.create_equip = function(uid, data) { return invoke('create_equip', { uid: uid, data: data }); };
    api.delete_equip = function(uid, eid) { return invoke('delete_equip', { uid: uid, equipUid: eid }); };
    api.update_equip = function(uid, eid, data) { return invoke('update_equip', { uid: uid, equipUid: eid, data: data }); };
    api.get_hadal_zone = function(uid) { return invoke('get_hadal_zone', { uid: uid }); };
    api.update_hadal_zone = function(uid, data) { return invoke('update_hadal_zone', { uid: uid, data: data }); };
    api.export_player_data = function(uid) { return Promise.resolve({ ok: false, error: 'Not yet implemented' }); };
    api.import_player_data = function(uid, data) { return Promise.resolve({ ok: false, error: 'Not yet implemented' }); };
    api.window_minimize = function() { invoke('window_minimize'); };
    api.window_toggle_max = function() { invoke('window_toggle_max'); };
    api.window_close = function() { invoke('window_close'); };
    api.window_move = function() {};
    return api;
  }

  function trySetup() {
    if (!getInvoke()) return false;
    window.pywebview = { api: buildApi() };
    window.dispatchEvent(new Event('pywebviewready'));
    return true;
  }

  if (!trySetup()) {
    var iv = setInterval(function() {
      if (trySetup()) clearInterval(iv);
    }, 50);
    setTimeout(function() { clearInterval(iv); trySetup(); }, 5000);
  }
})();
