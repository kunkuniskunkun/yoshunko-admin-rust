// ═══════════════════════════════════════════════════════
// Global State + Template Helpers
// ═══════════════════════════════════════════════════════
var state = {
  uid: null,
  panel: 'avatars',
  templates: null,
  selectedAvatarId: null,
  selectedWeaponUid: null,
  selectedEquipUid: null,
  equipView: 'gallery',
  skillTypes: {},
  skillData: {},
  avatarSearch: '',
  avatarView: 'gallery',
  avatarGroupBy: 'camp', // 'camp' or 'profession'
  weaponSearch: '',
  weaponView: 'gallery',
  equipSearch: '',
  avatarMap: null,
  weaponMap: null,
  // Data cache — avoid re-fetching on panel switch
  _avatarCache: null,
  _weaponCache: null,
  _equipCache: null,
  _cacheDirty: false,
  _scrollPos: {}
};

// ═══════════════════════════════════════════════════════
// Build template lookup Maps (Task 4.6 — O(1) lookups)
// ═══════════════════════════════════════════════════════
function buildTemplateMaps() {
  if (!state.templates) return;
  state.avatarMap = new Map();
  if (state.templates.avatars) {
    state.templates.avatars.forEach(function(a) {
      state.avatarMap.set(a.id, a);
    });
  }
  state.weaponMap = new Map();
  if (state.templates.weapons) {
    state.templates.weapons.forEach(function(w) {
      state.weaponMap.set(w.id, w);
    });
  }
}

// ═══════════════════════════════════════════════════════
// Template lookup helpers (using Map for O(1) access)
// ═══════════════════════════════════════════════════════
function avatarName(id) {
  if (state.avatarMap) {
    var t = state.avatarMap.get(id);
    if (t) return '' + t.name;
  }
  // Fallback to array search
  var t = state.templates && state.templates.avatars ? state.templates.avatars.find(function(a) { return a.id === id; }) : null;
  return t ? '' + t.name : 'Avatar ' + id;
}

function weaponName(id) {
  if (state.weaponMap) {
    var t = state.weaponMap.get(id);
    if (t) return t.name;
  }
  // Fallback to array search
  var t = state.templates && state.templates.weapons ? state.templates.weapons.find(function(w) { return w.id === id; }) : null;
  return t ? t.name : 'Weapon ' + id;
}

var suitColors = ['wood', 'fire', 'ice', 'elec', 'phys', 'ether'];
function suitColorClass(suitType) {
  return suitColors[suitType % suitColors.length] || '';
}
