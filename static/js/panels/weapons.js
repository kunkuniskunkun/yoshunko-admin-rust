// ═══════════════════════════════════════════════════════
// Weapons Panel — Card Gallery grouped by profession
// ═══════════════════════════════════════════════════════

async function renderWeapons() {
  // Use cache if available
  if (state._weaponCache && !state._cacheDirty) {
    var weapons = state._weaponCache;
  } else {
    var data = await API.getWeapons(state.uid);
    state._weaponCache = data.weapons;
    state._cacheDirty = false;
    var weapons = data.weapons;
  }

  weapons.sort(function(a, b) { return b.uid - a.uid; });

  // Filter out B-rank weapons (12000-12999)
  weapons = weapons.filter(function(w) { return w.id < 12000 || w.id > 12999; });

  // Search filter
  var q = (state.weaponSearch || '').toLowerCase();
  if (q) {
    weapons = weapons.filter(function(w) {
      // Pinyin matching
      var py = (typeof WEAPON_PINYIN !== 'undefined' && WEAPON_PINYIN[w.id]) ? WEAPON_PINYIN[w.id] : null;
      var pinyinMatch = py && (py.full.includes(q) || py.initials.includes(q));
      var profName = (w.profession || '').toLowerCase();
      return String(w.id).includes(q) || String(w.uid).includes(q)
        || (w.name || '').toLowerCase().includes(q)
        || profName.includes(q)
        || pinyinMatch;
    });
  }

  // ── Editor View ──
  if (state.weaponView === 'editor' && state.selectedWeaponUid) {
    $('#main').innerHTML = await weaponEditorHTML(state.selectedWeaponUid);
    return;
  }

  // ── Gallery View ──
  state.weaponView = 'gallery';

  // Group by profession (from weapon data directly)
  var groups = {};
  var profOrder = ['强攻', '击破', '异常', '支援', '防护', '命破'];
  var ungroupedIdx = 0;
  for (var i = 0; i < weapons.length; i++) {
    var w = weapons[i];
    var prof = w.profession || '';
    if (!prof) {
      prof = '_其他_' + (ungroupedIdx++);
      if (!groups[prof]) groups[prof] = { label: '未知职业', weapons: [] };
    } else {
      if (!groups[prof]) groups[prof] = { label: prof, weapons: [] };
    }
    groups[prof].weapons.push(w);
  }

  var html = '<div class="page-header"><h2>音擎仓库</h2><span class="subtitle">管理音擎等级、星级突破与精炼等级</span></div>';

  // Search bar
  html += '<div class="search-wrap"><input type="text" id="weapon-search" placeholder="搜索音擎 ID、名称、职业..." aria-label="搜索音擎" value="' + escHtml(state.weaponSearch || '') + '" data-action="search-weapons"><span class="search-count" id="weapon-search-count"></span>' + (state.weaponSearch ? '<button class="search-clear" data-action="clear-weapon-search" aria-label="清除搜索">×</button>' : '') + '</div>';

  if (weapons.length === 0) {
    html += '<div class="empty-state"><div class="empty-state__icon"></div><p>没有找到匹配的音擎</p></div>';
    $('#main').innerHTML = html;
  applyStaggeredAnimation('.avatar-gallery__card');
    return;
  }

  html += '<div class="avatar-gallery">';
  for (var gi = 0; gi < profOrder.length; gi++) {
    var prof = profOrder[gi];
    var group = groups[prof];
    if (!group) continue;
    html += '<div class="avatar-gallery__camp-section">';
    html += '<div class="avatar-gallery__camp-header">' + group.label + ' <span class="text-xs text-muted">(' + group.weapons.length + ')</span></div>';
    html += '<div class="avatar-gallery__grid">';
    for (var wi = 0; wi < group.weapons.length; wi++) {
      var w = group.weapons[wi];
      var tier;
      if (w.id >= 14000) tier = 'S';
      else if (w.id >= 13000) tier = 'A';
      else tier = 'B';
      var rarityCls = tier === 'S' ? 'rarity-s' : (tier === 'A' ? 'rarity-a' : '');
      var cardCls = tier === 'S' ? 'avatar-gallery__card--s' : (tier === 'A' ? 'avatar-gallery__card--a' : '');
      var maxRefine = w.max_refine || 5;
      html += '<div class="game-card avatar-gallery__card ' + cardCls + '" tabindex="0" role="button" data-action="select-weapon" data-id="' + w.uid + '">' +
        '<div class="card-header">' +
          '<span>' + (rarityCls ? '<span class="game-card__rarity ' + rarityCls + '">' + tier + '</span>' : '') + '<span class="card-title">' + escHtml(w.name) + '</span></span>' +
          '<span class="game-card__level">Lv.' + w.level + '</span>' +
        '</div>' +
        '<div class="card-meta">' +
          '<span>' + renderStars(w.refine_level, maxRefine) + '</span>' +
          '<span class="refine-pill">R' + w.refine_level + '</span>' +
        '</div>' +
        '<div class="avatar-card__footer">' +
          '<span class="avatar-card__camp-tag">' + escHtml(group.label) + '</span>' +
          '<span class="text-xs text-muted">#' + w.uid + '</span>' +
        '</div>' +
      '</div>';
    }
    html += '</div></div>';
  }
  // Handle unknown profession groups
  for (var key in groups) {
    if (profOrder.indexOf(key) >= 0) continue;
    var group = groups[key];
    html += '<div class="avatar-gallery__camp-section">';
    html += '<div class="avatar-gallery__camp-header">' + group.label + ' <span class="text-xs text-muted">(' + group.weapons.length + ')</span></div>';
    html += '<div class="avatar-gallery__grid">';
    for (var wi = 0; wi < group.weapons.length; wi++) {
      var w = group.weapons[wi];
      var maxRef = w.max_refine || 5;
      html += '<div class="game-card avatar-gallery__card" tabindex="0" role="button" data-action="select-weapon" data-id="' + w.uid + '">' +
        '<div class="card-header"><span class="card-title">' + escHtml(w.name) + '</span><span class="game-card__level">Lv.' + w.level + '</span></div>' +
        '<div class="card-meta"><span>' + renderStars(w.refine_level, maxRef) + '</span></div>' +
        '<div class="avatar-card__footer"><span class="text-xs text-muted">#' + w.uid + '</span></div>' +
      '</div>';
    }
    html += '</div></div>';
  }
  html += '</div>';

  $('#main').innerHTML = html;
  applyStaggeredAnimation('.avatar-gallery__card');
}

var onWeaponSearch = createSearchHandler('weaponSearch', 'weaponView', renderWeapons, 'selectedWeaponUid');

function selectWeapon(uid) {
  state._scrollPos.weapons = $('#main').scrollTop;
  state.selectedWeaponUid = uid;
  state.weaponView = 'editor';
  renderWeapons().then(function() {
    var m = $('#main');
    m.classList.remove('editor-slide-in');
    void m.offsetHeight;
    m.classList.add('editor-slide-in');
  });
}

function backToWeaponGallery() {
  state.weaponView = 'gallery';
  state.selectedWeaponUid = null;
  renderWeapons();
  var m = $('#main');
  m.style.scrollBehavior = 'auto';
  m.scrollTop = state._scrollPos.weapons || 0;
  m.style.scrollBehavior = '';
}

async function weaponEditorHTML(wuid) {
  var w = await API.getWeapon(state.uid, wuid);
  var prof = w.profession || '未知';
  var maxRefine = w.max_refine || 5;

  var h = '<div class="editor-page">';
  h += '<div class="editor-page__top">';
  h += '<span class="back-link" data-action="back-to-weapon-gallery">← 音擎仓库</span>';
  h += '<div class="editor-page__header">';
  h += '<h2>' + escHtml(w.name) + '</h2>';
  if (w.en_name) h += '<span class="sub en-name">' + escHtml(w.en_name) + '</span>';
  h += '<div class="editor-header-meta">';
  h += '<span class="slot-tag">' + escHtml(prof) + '</span>';
  h += '<span class="star-rating star-rating--lg">';
  for (var i = 1; i <= maxRefine; i++) {
    h += '<span class="star ' + (i <= w.refine_level ? 'on' : 'off') + '">★</span>';
  }
  h += '</span>';
  h += '<span class="sub">#' + wuid + '</span>';
  h += '</div></div></div>';

  h += '<div class="editor-page__body">';

  // Level
  h += '<div class="section-title">基础属性</div>';
  h += '<div class="form-row">';
  h += '<div class="form-field"><label class="form-label">等级</label>' + stepperInput('w-level', w.level, 1, 60) + '</div>';
  h += '<div class="form-field"><label class="form-label">精炼等级</label>' + stepperInput('w-refine', w.refine_level, 1, maxRefine) + '<span class="text-xs text-muted">max: ' + maxRefine + '</span></div>';
  h += '</div>';

  h += '</div>';
  h += '<button class="btn btn-primary editor-fab" data-action="save-weapon" data-id="' + wuid + '">保存更改</button>';
  h += '</div>';
  return h;
}

async function saveWeapon(wuid) {
  await safeSave('音擎数据', async function() {
    var result = await API.updateWeapon(state.uid, wuid, {
      level: parseInt($('#w-level').value),
      refine_level: parseInt($('#w-refine').value)
    });
    if (result && result.ok === false) throw new Error(result.error || '保存失败');
    markCacheDirty();
    state.weaponView = 'gallery';
    state.selectedWeaponUid = null;
    renderWeapons();
  });
}
