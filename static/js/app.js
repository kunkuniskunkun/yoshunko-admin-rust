// ═══════════════════════════════════════════════════════
// App Core — Navigation, Render Dispatch, Event Delegation
// ═══════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════
// Event Delegation (Task 4.3)
// Catches all [data-action] clicks via document-level listener
// ═══════════════════════════════════════════════════════
document.addEventListener('click', function(e) {
  var el = e.target.closest('[data-action]');
  if (!el) return;
  var action = el.dataset.action;

  // Window controls (frameless)
  if (action === 'win-min') { try { pywebview.api.window_minimize(); } catch(e) {} return; }
  if (action === 'win-max') { try { pywebview.api.window_toggle_max(); } catch(e) {} return; }
  if (action === 'win-close') { try { pywebview.api.window_close(); } catch(e) {} return; }

  // Gallery / Navigation actions
  if (action === 'back-to-gallery') { backToGallery(); return; }
  if (action === 'back-to-weapon-gallery') { backToWeaponGallery(); return; }
  if (action === 'back-to-equip-gallery') { backToEquipGallery(); return; }

  // Equip creation flow
  if (action === 'show-create-equip') { showCreateEquip(); return; }
  if (action === 'close-create-equip') { closeCreateEquip(); return; }
  if (action === 'select-create-suit') { selectCreateSuit(el.dataset.suit); return; }
  if (action === 'select-create-slot') {
    selectCreateSlot(parseInt(el.dataset.eqid), parseInt(el.dataset.slot), el.dataset.suit, el.dataset.slotname);
    return;
  }

  // Save actions
  if (action === 'save-hadal-zone') { saveHadalZone(); return; }
  if (action === 'save-player-info') { savePlayerInfo(); return; }
  if (action === 'setup-connect') { onSetupConnect(); return; }
  if (action === 'close-toast') { var t = el.closest('.toast'); if (t) removeToast(t); return; }
  if (action === 'confirm-cancel') { var ov = el.closest('.confirm-overlay'); if (ov) ov.remove(); return; }
  if (action === 'confirm-ok') { var ov = el.closest('.confirm-overlay'); if (ov) { var cb = ov._onConfirm; ov.remove(); if (cb) cb(); } return; }
  if (action === 'change-state-dir') { showConfirm('更改数据目录需要重新设置。确定继续？', function() { renderSetup(); }); return; }
  if (action === 'auto-detect-paths') { autoDetectSettings(); return; }
  if (action === 'open-settings') { state.panel = 'settings'; $$('.nav-item').forEach(function(n) { n.classList.remove('active'); }); state.selectedAvatarId = null; state.selectedWeaponUid = null; state.selectedEquipUid = null; state.equipView = 'gallery'; state.avatarView = 'gallery'; state.weaponView = 'gallery'; _dirty = false; renderPanel().then(function() { var m = $('#main'); m.classList.remove('editor-slide-in'); void m.offsetHeight; m.classList.add('editor-slide-in'); }); return; }
  if (action === 'toggle-theme') { toggleTheme(el); return; }
  if (action === 'set-theme-light') { revealTheme(el, 'light'); return; }
  if (action === 'set-theme-dark') { revealTheme(el, 'dark'); return; }
  if (action === 'toggle-animations') { var ck = el.checked; var v = ck ? 'on' : 'off'; try { localStorage.setItem('yos-animations', v); } catch(e) {} document.documentElement.setAttribute('data-animations', v); var hint = el.parentElement.nextElementSibling; if (hint) hint.textContent = ck ? '已开启' : '已关闭'; return; }
  if (action === 'clear-caches') { state._avatarCache = null; state._weaponCache = null; state._equipCache = null; state._cacheDirty = true; toast('缓存已清除'); return; }
  if (action === 'reset-config') { showConfirm('重置配置将返回初始化页面，继续？', function() { renderSetup(); }); return; }
  if (action === 'go-shortcuts') { state.panel = 'shortcuts'; $$('.nav-item').forEach(function(n) { n.classList.remove('active'); }); renderShortcuts(); return; }
  if (action === 'back-to-suit-select') {
    document.getElementById('create-step1').style.display = 'block';
    document.getElementById('create-step2').style.display = 'none';
    document.getElementById('step-label').textContent = '选择套装';
    return;
  }
  if (action === 'back-to-slot-select') { selectCreateSuit(el.dataset.suit); return; }
  if (action === 'export-player') { exportPlayerData(); return; }
  if (action === 'import-player') { importPlayerData(); return; }

  // Quick launch actions
  if (action === 'launch-yoshunko') { launchYoshunko(); return; }
  if (action === 'launch-program') { launchByPath(el.dataset.key); return; }
  if (action === 'launch-all') { launchAll(); return; }
  if (action === 'save-launch-path') { saveLaunchPath(el.dataset.key); return; }
  if (action === 'edit-launch-path') { editLaunchPath(el.dataset.key); return; }
  if (action === 'paste-launch-path') {
    navigator.clipboard.readText().then(function(text) {
      var inputEl = document.getElementById('path-input-' + el.dataset.key);
      if (inputEl && text) inputEl.value = text.trim();
    }).catch(function() {});
    return;
  }

  // Modal overlay close (only if backdrop was clicked, not children)
  if (action === 'modal-close') {
    if (e.target === el) closeCreateEquip();
    return;
  }

  // Setup candidate path click
  if (action === 'candidate-path') {
    var inputEl = document.getElementById('setup-state-dir');
    if (inputEl) inputEl.value = el.dataset.path;
    return;
  }

  // Paste from clipboard to input
  if (action === 'paste-to-input') {
    navigator.clipboard.readText().then(function(text) {
      var inputEl = document.getElementById('setup-state-dir');
      if (inputEl && text) inputEl.value = text.trim();
    }).catch(function() { toast('无法读取剪贴板', 'error'); });
    return;
  }

  // ID-based actions
  var rawId = parseInt(el.dataset.id);
  var id = !isNaN(rawId) ? rawId : null;

  if (action === 'select-avatar') { pressCard(el); selectAvatar(id); return; }
  if (action === 'save-avatar') { saveAvatar(id); return; }
  if (action === 'select-weapon') { pressCard(el); selectWeapon(id); return; }
  if (action === 'save-weapon') { saveWeapon(id); return; }
  if (action === 'select-equip') { pressCard(el); selectEquip(id); return; }
  if (action === 'save-equip') { saveEquip(id); return; }
  if (action === 'delete-equip') { deleteEquip(id); return; }
  if (action === 'create-equip') { createEquip(id); return; }

  // Clamp input on change
  if (action === 'clamp-input') {
    var v = parseInt(el.value) || 0;
    var mn = parseInt(el.dataset.min) || 0;
    var mx = parseInt(el.dataset.max) || 999;
    if (v < mn) el.value = mn;
    if (v > mx) el.value = mx;
    return;
  }

  // Search clear buttons
  if (action === 'clear-avatar-search') { onAvatarSearch(''); document.getElementById('avatar-search').value = ''; return; }
  if (action === 'clear-weapon-search') { onWeaponSearch(''); document.getElementById('weapon-search').value = ''; return; }
  if (action === 'clear-equip-search') { onEquipSearch(''); document.getElementById('equip-search').value = ''; return; }

  // Avatar group toggle
  if (action === 'toggle-avatar-group') {
    state.avatarGroupBy = state.avatarGroupBy === 'camp' ? 'profession' : 'camp';
    renderAvatars();
    return;
  }

  // Unified stepper (Task 4.4)
  if (action === 'step') {
    stepBySelector(
      el.dataset.selector,
      parseInt(el.dataset.delta),
      parseInt(el.dataset.min),
      parseInt(el.dataset.max)
    );
    return;
  }
});

// IME composition guard for search
var _imeComposing = false;
document.addEventListener('compositionstart', function() { _imeComposing = true; });
document.addEventListener('compositionend', function(e) {
  _imeComposing = false;
  // Fire search with final composed value
  var el = e.target.closest('[data-action^="search-"]');
  if (!el) return;
  var action = el.dataset.action;
  if (action === 'search-avatars') onAvatarSearch(el.value);
  else if (action === 'search-weapons') onWeaponSearch(el.value);
  else if (action === 'search-equips') onEquipSearch(el.value);
});

// Input event delegation for search and clamp
document.addEventListener('input', function(e) {
  if (_imeComposing) return; // Don't search during IME composition
  var el = e.target.closest('[data-action]');
  if (!el) return;
  var action = el.dataset.action;
  if (action === 'search-avatars') onAvatarSearch(el.value);
  else if (action === 'search-weapons') onWeaponSearch(el.value);
  else if (action === 'search-equips') onEquipSearch(el.value);
  else if (action === 'clamp-input') {
    var v = parseInt(el.value) || 0;
    var mn = parseInt(el.dataset.min) || 0;
    var mx = parseInt(el.dataset.max) || 999;
    if (v < mn) el.value = mn;
    if (v > mx) el.value = mx;
  }
});

// Keyboard activation for cards
document.addEventListener('keydown', function(e) {
  if (e.key === 'Enter' || e.key === ' ') {
    var card = e.target.closest('[role="button"]');
    if (card && card.dataset.action) {
      e.preventDefault();
      card.click();
      return;
    }
  }

  if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') return;

  // ESC: close modal or return to gallery
  if (e.key === 'Escape') {
    var confirmEl = document.querySelector('.confirm-overlay');
    if (confirmEl) { confirmEl.remove(); return; }
    var modal = document.querySelector('.modal-overlay');
    if (modal) { closeCreateEquip(); return; }
    if (state.panel === 'avatars' && state.avatarView === 'editor') { backToGallery(); return; }
    if (state.panel === 'weapons' && state.weaponView === 'editor') { backToWeaponGallery(); return; }
    if (state.panel === 'equips' && state.equipView === 'editor') { backToEquipGallery(); return; }
  }

  // Tab: basic focus trap for create modal
  if (e.key === 'Tab') {
    var modal = document.querySelector('.modal-overlay');
    if (!modal) return;
    var focusable = modal.querySelectorAll('button, [tabindex], input, select');
    if (focusable.length === 0) return;
    var first = focusable[0];
    var last = focusable[focusable.length - 1];
    if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
  }

  // 1-6: switch panels
  if (e.key >= '1' && e.key <= '7') {
    var tabs = $$('.nav-item');
    var map = {1: 'avatars', 2: 'weapons', 3: 'equips', 4: 'hadal_zone', 5: 'player_info', 6: 'settings', 7: 'quick_launch'};
    var target = map[parseInt(e.key)];
    if (target) { state.panel = target; $$('.nav-item').forEach(function(n) { n.classList.remove('active'); }); var items = $$('.nav-item'); for (var i = 0; i < items.length; i++) { if (items[i].dataset.panel === target) { items[i].classList.add('active'); break; } } state.selectedAvatarId = null; state.selectedWeaponUid = null; state.selectedEquipUid = null; state.equipView = 'gallery'; state.avatarView = 'gallery'; state.weaponView = 'gallery'; renderPanel(); }
  }

  // Ctrl+F: focus search
  if (e.ctrlKey && e.key === 'f') {
    e.preventDefault();
    var map = { avatars: 'avatar-search', weapons: 'weapon-search', equips: 'equip-search' };
    var searchId = map[state.panel];
    if (searchId) { var input = document.getElementById(searchId); if (input) { input.focus(); input.select(); } }
    else { var s = document.querySelector('.search-wrap input'); if (s) s.focus(); }
  }

  // Ctrl+S: save
  if (e.ctrlKey && e.key === 's') {
    e.preventDefault();
    var saveBtn = document.querySelector('[data-action="save-avatar"][data-id], [data-action="save-weapon"][data-id], [data-action="save-equip"][data-id]');
    if (saveBtn) saveBtn.click();
  }

  // Ctrl+Z: undo
  if (e.ctrlKey && !e.shiftKey && e.key === 'z') {
    e.preventDefault();
    var snap = popUndo();
    if (snap && snap.restore) snap.restore();
  }
});

// ═══════════════════════════════════════════════════════
// Navigation
// ═══════════════════════════════════════════════════════
$('#nav').addEventListener('click', function(e) {
  var item = e.target.closest('.nav-item');
  if (!item) return;
  $$('.nav-item').forEach(function(n) { n.classList.remove('active'); });
  item.classList.add('active');
  item.setAttribute('aria-selected', 'true');
  $$('.nav-item').forEach(function(n) { if (n !== item) n.setAttribute('aria-selected', 'false'); });
  state.panel = item.dataset.panel;
  state.selectedAvatarId = null;
  state.selectedWeaponUid = null;
  state.selectedEquipUid = null;
  state.equipView = 'gallery';
  state.avatarView = 'gallery';
  state.weaponView = 'gallery';
  _dirty = false;
  renderPanel();
});

async function doSwitchPlayer(newUid) {
  state.uid = newUid;
  state.selectedAvatarId = null;
  state.selectedWeaponUid = null;
  state.selectedEquipUid = null;
  state.equipView = 'gallery';
  state.avatarView = 'gallery';
  state.weaponView = 'gallery';
  state.skillTypes = {};
  state.skillData = {};
  state.avatarSearch = '';
  state.weaponSearch = '';
  state.equipSearch = '';
  state._avatarCache = null;
  state._weaponCache = null;
  state._equipCache = null;
  state._cacheDirty = false;
  if (state.uid) {
    await loadCounts();
    await renderPanel();
  } else {
    $('#main').innerHTML = '<div class="empty-state"><div class="empty-state__icon"></div><p>选择一个玩家开始管理游戏数据</p></div>';
  }
}

$('#player-select').addEventListener('change', function() {
  var rawVal = parseInt(this.value);
  var newUid = !isNaN(rawVal) ? rawVal : null;
  if (_dirty && state.uid && newUid !== state.uid) {
    showConfirm('当前有未保存的更改，切换玩家将丢失。继续？', function() {
      _dirty = false;
      doSwitchPlayer(newUid);
      $('#player-select').value = newUid;
    });
    this.value = state.uid;
    return;
  }
  doSwitchPlayer(newUid);
});

// ═══════════════════════════════════════════════════════
// Render Dispatcher
// ═══════════════════════════════════════════════════════
async function renderPanel() {
  if (!state.uid) return;
  // Remove floating launch-all button if exists
  var fab = document.querySelector('.launch-all-fab');
  if (fab) fab.remove();
  showSkeleton($('#main'));
  try {
    switch(state.panel) {
      case 'avatars': await renderAvatars(); break;
      case 'weapons': await renderWeapons(); break;
      case 'equips': await renderEquips(); break;
      case 'hadal_zone': await renderHadalZone(); break;
      case 'player_info': await renderPlayerInfo(); break;
      case 'quick_launch': await renderQuickLaunch(); break;
      case 'settings': await renderSettings(); break;
      case 'shortcuts': renderShortcuts(); break;
    }
    var m = $('#main');
    m.classList.remove('editor-slide-in');
    void m.offsetHeight;
    m.classList.add('editor-slide-in');
  } catch(e) {
    $('#main').innerHTML = '<div class="empty-state"><p class="text-danger">加载失败: ' + escHtml(e.message) + '</p></div>';
  }
}

// ═══════════════════════════════════════════════════════
// Load badge counts for sidebar
// ═══════════════════════════════════════════════════════
function markCacheDirty() {
  state._cacheDirty = true;
}

async function loadCounts() {
  if (!state.uid) return;
  try {
    var results = await Promise.all([
      API.getAvatars(state.uid),
      API.getWeapons(state.uid),
      API.getEquips(state.uid)
    ]);
    // Also seed the cache
    state._avatarCache = results[0].avatars;
    state._weaponCache = results[1].weapons;
    state._equipCache = results[2].equips;
    state._cacheDirty = false;
    $('#avatar-count').textContent = results[0].avatars.filter(function(a) { return a.avatar_id !== 2071 && a.avatar_id !== 2121; }).length;
    $('#weapon-count').textContent = results[1].weapons.filter(function(w) { return w.id < 12000 || w.id > 12999; }).length;
    $('#equip-count').textContent = results[2].equips.length;
  } catch(e) { console.error('loadCounts failed:', e); }
}

// ═══════════════════════════════════════════════════════
// Version display
// ═══════════════════════════════════════════════════════
async function renderVersion() {
  try {
    var resp = await API.getVersion();
    var v = resp.version;
    var elFooter = document.getElementById('app-version');
    if (elFooter) elFooter.textContent = v;
    var elSetup = document.getElementById('setup-version');
    if (elSetup) elSetup.textContent = v;
    var elTitle = document.getElementById('title-version');
    if (elTitle) elTitle.textContent = v;
  } catch(e) {
    // fallback: keep hardcoded version
  }
}

// ═══════════════════════════════════════════════════════
// Init Application
// ═══════════════════════════════════════════════════════
async function initApp() {
  try {
    var cfg = await API.getConfig();
    if (cfg && cfg.configured && cfg.config_exists) {
      var data = await API.getPlayerList();
      var sel = $('#player-select');
      sel.innerHTML = '<option value="">-- 选择玩家 --</option>';
      for (var i = 0; i < data.players.length; i++) {
        var pid = data.players[i];
        sel.innerHTML += '<option value="' + pid + '">玩家 UID: ' + escHtml(String(pid)) + '</option>';
      }
      if (data.players.length > 0) {
        sel.value = data.players[0];
        state.uid = data.players[0];
      }
      state.templates = await API.getTemplates();
      buildTemplateMaps();
      if (state.uid) { await loadCounts(); await renderPanel(); }
      renderVersion();
      return;
    }
  } catch(e) { console.error('initApp error:', e); }
  // Not configured — enhance existing setup form with auto-detect
  setupEnhance();
  renderVersion();
}

// ═══════════════════════════════════════════════════════
// Theme toggle — overlay opacity fade
// ═══════════════════════════════════════════════════════
function revealTheme(btnEl, targetTheme) {
  var html = document.documentElement;
  var current = html.getAttribute('data-theme');
  if (current === targetTheme) return;

  // Capture old theme's background color
  var oldBg = getComputedStyle(document.body).backgroundColor;
  // Switch theme instantly
  html.setAttribute('data-theme', targetTheme);
  try { localStorage.setItem('yos-theme', targetTheme); } catch (e) {}
  // Overlay with old color fades out, revealing new theme underneath
  var overlay = document.createElement('div');
  overlay.className = 'theme-fade-overlay';
  overlay.style.background = oldBg;
  document.body.appendChild(overlay);
  overlay.addEventListener('animationend', function() {
    overlay.remove();
  });
  if (state.panel === 'settings') renderSettings();
}

function toggleTheme(btnEl) {
  var current = document.documentElement.getAttribute('data-theme');
  var next = current === 'dark' ? 'light' : 'dark';
  revealTheme(btnEl, next);
}

// ═══════════════════════════════════════════════════════

function pressCard(el) {
  el.style.transition = 'transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1)';
  el.style.transform = 'scale(0.92)';
  setTimeout(function () { el.style.transform = 'scale(1)'; }, 120);
}

function showSkeleton(container) {
  container.innerHTML = '<div class="skeleton-wrap"><div class="skeleton skeleton--title"></div><div class="skeleton-grid">' +
    Array(6).fill('<div class="skeleton skeleton--card"></div>').join('') + '</div></div>';
}

function applyStaggeredAnimation(selector) {
  var cards = document.querySelectorAll(selector);
  cards.forEach(function(card, i) {
    card.style.opacity = '0';
    card.style.transform = 'translateY(16px)';
    card.style.transition = 'opacity 0.3s cubic-bezier(0.34,1.56,0.64,1), transform 0.3s cubic-bezier(0.34,1.56,0.64,1)';
    card.style.transitionDelay = (i % 6) * 35 + 'ms';
    requestAnimationFrame(function() {
      card.style.opacity = '1';
      card.style.transform = 'translateY(0)';
    });
  });
}

async function autoDetectSettings() {
  try {
    var r = await API.autoDetectPaths();
    if (r.candidates && r.candidates.length > 0) {
      var el = document.getElementById('set-state-dir');
      if (el) el.value = r.candidates[0];
      toast('检测到 ' + r.candidates.length + ' 个路径');
    } else { toast('未检测到可用路径', 'warning'); }
  } catch(e) { toast('检测失败', 'error'); }
}

async function setupEnhance() {
  try {
    var v = await API.getVersion();
    var el = document.getElementById('setup-version');
    if (el) el.textContent = v.version;
  } catch(e) {}
  try {
    var r = await API.autoDetectPaths();
    if (r.candidates && r.candidates.length > 0) {
      var candHtml = '';
      r.candidates.forEach(function(p) {
        candHtml += '<div class=\"candidate-path\" data-action=\"candidate-path\" data-path=\"' + escHtml(p) + '\">' + escHtml(p) + '</div>';
      });
      var el = document.getElementById('setup-candidates');
      if (el) el.innerHTML = '<p class=\"text-sm text-muted mb-1 mt-2\">检测到的路径（点击填入）：</p>' + candHtml;
      var inp = document.getElementById('setup-state-dir');
      if (inp) inp.value = r.candidates[0];
    }
  } catch(e) {}
}

// ═══════════════════════════════════════════════════════
// Theme init (no localStorage dependency)
// ═══════════════════════════════════════════════════════
(function() {
  try { var t = localStorage.getItem('yos-theme'); if (t) document.documentElement.setAttribute('data-theme', t); } catch(e) {}
})();

// ═══════════════════════════════════════════════════════
// Title bar drag (JS fallback for frameless window)
// ═══════════════════════════════════════════════════════
(function() {
  var titleBar = document.querySelector('.title-bar');
  if (!titleBar) return;
  var dragging = false, startX = 0, startY = 0;
  titleBar.addEventListener('mousedown', function(e) {
    if (e.target.closest('.tb-btn') || e.target.closest('[data-action]')) return;
    dragging = true;
    startX = e.screenX;
    startY = e.screenY;
    e.preventDefault();
  });
  document.addEventListener('mousemove', function(e) {
    if (!dragging) return;
    var dx = e.screenX - startX;
    var dy = e.screenY - startY;
    if (dx === 0 && dy === 0) return;
    startX = e.screenX;
    startY = e.screenY;
    try { window.pywebview.api.window_move(dx, dy); } catch(ex) {}
  });
  document.addEventListener('mouseup', function() { dragging = false; });
})();

// ═══════════════════════════════════════════════════════
// Undo Stack
// ═══════════════════════════════════════════════════════
var _undoStack = [];
var MAX_UNDO = 20;
function pushUndo(snapshot) {
  _undoStack.push(snapshot);
  if (_undoStack.length > MAX_UNDO) _undoStack.shift();
}
function popUndo() { return _undoStack.pop(); }

// ═══════════════════════════════════════════════════════
// Unsaved changes warning
// ═══════════════════════════════════════════════════════
var _dirty = false;
function markDirty() { _dirty = true; updateDirtyNav(); }
function markClean() { _dirty = false; updateDirtyNav(); }
function updateDirtyNav() {
  $$('.nav-item').forEach(function(n) {
    if (n.dataset.panel === state.panel) n.classList.toggle('dirty', _dirty);
  });
}
window.addEventListener('beforeunload', function(e) {
  if (_dirty) { e.preventDefault(); e.returnValue = ''; }
});

// Mark dirty on input changes in editor views
document.addEventListener('input', function(e) {
  if (e.target.closest('.panel-box__body') || e.target.closest('.weapon-editor') || e.target.closest('.avatar-editor') || e.target.closest('.equip-editor')) {
    _dirty = true;
  }
});

// ═══════════════════════════════════════════════════════
// Bootstrap — poll bridge, auto-detect first, then init
// ═══════════════════════════════════════════════════════
(function bootstrap() {
  function ready() { return window.pywebview && window.pywebview.api; }

  // Auto-detect runs FIRST, independently, as soon as bridge is ready
  function tryAutoDetect() {
    if (!ready()) return false;
    if (!document.getElementById('setup-state-dir')) return false;
    setupEnhance();
    return true;
  }

  // Aggressive auto-detect: every 200ms
  var _ad = setInterval(function() {
    if (tryAutoDetect() || ++_adTries > 150) clearInterval(_ad);
  }, 200);
  var _adTries = 0;

  // Init runs after auto-detect attempt
  function tryInit() {
    if (!ready()) return false;
    clearInterval(_ad);
    tryAutoDetect();
    initApp();
    return true;
  }
  if (tryInit()) return;
  window.addEventListener('pywebviewready', tryInit);
  var _n = 0;
  var _t = setInterval(function() {
    if (tryInit() || ++_n >= 50) clearInterval(_t);
  }, 200);
})();
