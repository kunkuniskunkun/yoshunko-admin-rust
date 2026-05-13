// ═══════════════════════════════════════════════════════
// Equips Panel — 驱动盘仓库
// ═══════════════════════════════════════════════════════

async function renderEquips() {
  // Use cache if available
  if (state._equipCache && !state._cacheDirty) {
    var equips = state._equipCache;
  } else {
    var data = await API.getEquips(state.uid);
    state._equipCache = data.equips;
    state._cacheDirty = false;
    var equips = data.equips;
  }

  equips.sort(function(a, b) { return b.uid - a.uid; });

  // Search filter
  var q = (state.equipSearch || '').toLowerCase();
  if (q) {
    equips = equips.filter(function(eq) {
      // Pinyin matching for suit name
      var py = (typeof SUIT_PINYIN !== 'undefined') ? SUIT_PINYIN[String(eq.suit_type || '')] : null;
      var pinyinMatch = py && (py.full.includes(q) || py.initials.includes(q));
      return String(eq.uid).includes(q)
        || (eq.suit_name || '').toLowerCase().includes(q)
        || (eq.slot_name || '').toLowerCase().includes(q)
        || String(eq.id).includes(q)
        || pinyinMatch;
    });
  }

  // ── Editor View ──
  if (state.equipView === 'editor' && state.selectedEquipUid) {
    $('#main').innerHTML = await equipEditorHTML(state.selectedEquipUid);
    initEquipEditorDOM();
    return;
  }

  // ── Gallery View ──
  var html = '<div class="equip-gallery">';
  html += '<div class="page-header flex-between"><div><h2>驱动盘仓库</h2><span class="subtitle">管理驱动盘数据，包括主属性与副属性</span></div>';
  html += '<button class="btn btn-success" data-action="show-create-equip">+ 创建驱动盘</button></div>';

  // Search bar
  html += '<div class="search-wrap"><input type="text" id="equip-search" placeholder="搜索驱动盘 UID、套装、槽位..." aria-label="搜索驱动盘" value="' + escHtml(state.equipSearch || '') + '" data-action="search-equips"><span class="search-count" id="equip-search-count"></span>' + (state.equipSearch ? '<button class="search-clear" data-action="clear-equip-search" aria-label="清除搜索">×</button>' : '') + '</div>';

  var suits = {};
  for (var i = 0; i < equips.length; i++) {
    var eq = equips[i];
    var suit = eq.suit_name || 'Unknown';
    if (!suits[suit]) suits[suit] = [];
    suits[suit].push(eq);
  }

  var suitKeys = Object.keys(suits);
  for (var si = 0; si < suitKeys.length; si++) {
    var suit = suitKeys[si];
    var items = suits[suit];
    var suitType = 0;
    if (state.templates && state.templates.equips) {
      var tpl = state.templates.equips.find(function(e) { return items[0] && items[0].id === e.id; });
      if (tpl) suitType = tpl.suit_type || 0;
    }
    html += '<div class="equip-suit-section">';
    html += '<div class="equip-suit-title"><span class="suit-chip ' + suitColorClass(suitType) + '">' + escHtml(suit) + '</span> <span class="text-muted">· ' + items.length + ' 件</span></div>';
    html += '<div class="equip-grid">';
    for (var ei = 0; ei < items.length; ei++) {
      var eq = items[ei];
      var mainProp = eq.properties ? eq.properties[0] : null;
      var subCount = eq.sub_properties ? eq.sub_properties.filter(function(p) { return p !== null; }).length : 0;
      var statNames = (state.templates && state.templates.stat_names) || {};
      var mainStatName = mainProp ? (statNames[mainProp.key] || '属性' + mainProp.key) : '—';
      var totalAdd = (eq.sub_properties || []).reduce(function(s, p) { return s + (p && p.add_value ? p.add_value : 0); }, 0);
      var filled = (eq.sub_properties || []).filter(function(p) { return p !== null; }).length;
      var totalExtra = totalAdd - filled;
      var enhanceValid = totalExtra >= 4 && totalExtra <= 5;
      var posNum = eq.id % 10;
      var suitCls = 'suit-' + (suitType % 6);
      html += '<div class="game-card equip-card ' + suitCls + '" tabindex="0" role="button" data-action="select-equip" data-id="' + eq.uid + '">' +
        '<div class="card-header">' +
          '<span><span class="slot-tag">' + posNum + '号位</span><span class="card-title">#' + eq.uid + '</span></span>' +
          '<span class="game-card__level">Lv.' + eq.level + '</span>' +
        '</div>' +
        '<div class="card-meta">' +
          '<span>' + renderStars(eq.star, 5) + '</span>' +
          '<span class="enhance-badge ' + (enhanceValid ? 'enhance-badge--ok' : 'enhance-badge--warn') + '">+' + totalExtra + '</span>' +
        '</div>' +
        '<div class="stat-preview">' +
          '<span class="stat-pill stat-pill--main">' + mainStatName + ' ' + (mainProp ? mainProp.base_value || 0 : 0) + '</span>' +
          renderSubDots(subCount, 4) +
        '</div>' +
      '</div>';
    }
    html += '</div></div>';
  }

  html += '</div>';
  $('#main').innerHTML = html;
}

var onEquipSearch = createSearchHandler('equipSearch', 'equipView', renderEquips, 'selectedEquipUid');

function selectEquip(uid) {
  state._scrollPos.equips = $('#main').scrollTop;
  state.selectedEquipUid = uid;
  state.equipView = 'editor';
  renderEquips().then(function() {
    var m = $('#main');
    m.classList.remove('editor-slide-in');
    void m.offsetHeight;
    m.classList.add('editor-slide-in');
  });
}

function backToEquipGallery() {
  state.equipView = 'gallery';
  state.selectedEquipUid = null;
  renderEquips();
  var m = $('#main');
  m.style.scrollBehavior = 'auto';
  m.scrollTop = state._scrollPos.equips || 0;
  m.style.scrollBehavior = '';
}

// ═══════════════════════════════════════════════════════
// Equip Editor HTML (returns HTML string only, no setTimeout)
// Task 4.5: Binding happens in initEquipEditorDOM() called after innerHTML
// ═══════════════════════════════════════════════════════
async function equipEditorHTML(euid) {
  var eq = await API.getEquip(state.uid, euid);
  var mainOpts = (state.templates && state.templates.main_stat_options) || {};
  var subOpts = (state.templates && state.templates.sub_stat_options) || [];
  // Determine slot from equip ID: last digit = position
  var eqId = eq.id || 0;
  var eqSlot = eqId % 10;
  var slotOpts = mainOpts[String(eqSlot)] || [];
  var isFixed = (eqSlot >= 1 && eqSlot <= 3);

  var h = '<div class="editor-page">';
  h += '<div class="editor-page__top">';
  h += '<span class="back-link" data-action="back-to-equip-gallery">← 驱动盘仓库</span>';
  h += '<div class="editor-page__header"><h2>' + escHtml(eq.suit_name) + '</h2><span class="sub">' + eqSlot + '号位 · #' + euid + '</span></div>';
  h += '</div>';
  h += '<div class="editor-page__body">';

  h += '<div class="section-title">基本信息</div>';
  h += '<div class="form-row">';
  h += '<div class="form-field"><label class="form-label">等级</label>' + stepperInput('eq-level', eq.level, 0, 15) + '</div>';
  h += '<div class="form-field"><label class="form-label">星级</label>' + stepperInput('eq-star', eq.star, 1, 5) + '</div>';
  h += '</div>';

  h += '<div class="section-title">主属性</div>';
  var mp = eq.properties ? eq.properties[0] : null;
  h += '<div class="prop-row">';
  h += '<select id="eq-prop-key" class="form-select form-select--flex2"' + (isFixed ? ' disabled' : '') + '>';
  var foundMain = false;
  for (var oi = 0; oi < slotOpts.length; oi++) {
    var opt = slotOpts[oi];
    var sel = mp && mp.key === opt.key ? ' selected' : '';
    if (mp && mp.key === opt.key) foundMain = true;
    h += '<option value="' + opt.key + '" data-base="' + opt.base_value + '"' + sel + '>' + opt.name + '</option>';
  }
  if (mp && mp.key && !foundMain) {
    h += '<option value="' + mp.key + '" data-base="' + (mp.base_value || 0) + '" selected>属性' + mp.key + '</option>';
  }
  h += '</select>';
  h += '<input class="form-input prop-value-readonly" id="eq-prop-base" value="' + (mp ? mp.base_value || 0 : 0) + '" readonly>';
  h += '<span class="text-muted prop-add-label">+0</span>';
  h += '</div>';
  if (isFixed) {
    h += '<div class="text-sm text-muted hint-text">1-3号位主属性固定，不可更改</div>';
  }

  h += '<div class="section-title">副词条 · 4 条 <span class="text-sm text-muted">(追加强化 0-4)</span></div>';
  h += '<div class="prop-header"><span>#</span><span>属性</span><span>基础值</span><span>强化</span></div>';
  for (var i = 0; i < 4; i++) {
    var sp = eq.sub_properties ? eq.sub_properties[i] : null;
    var extraVal = sp ? Math.max(0, (sp.add_value || 1) - 1) : 0;
    h += '<div class="prop-row">';
    h += '<span class="prop-index">' + (i + 1) + '</span>';
    h += '<select class="form-select eq-sub-key form-select--flex2" data-idx="' + i + '">';
    h += '<option value="0">— 无 —</option>';
    var foundSub = false;
    for (var si = 0; si < subOpts.length; si++) {
      var sopt = subOpts[si];
      var ssel = sp && sp.key === sopt.key ? ' selected' : '';
      if (sp && sp.key === sopt.key) foundSub = true;
      h += '<option value="' + sopt.key + '" data-base="' + sopt.base_value + '"' + ssel + '>' + sopt.name + '</option>';
    }
    if (sp && sp.key && !foundSub) {
      h += '<option value="' + sp.key + '" data-base="' + (sp.base_value || 0) + '" selected>属性' + sp.key + '</option>';
    }
    h += '</select>';
    h += '<input class="form-input eq-sub-base prop-value-readonly" data-idx="' + i + '" value="' + (sp ? sp.base_value || 0 : 0) + '" readonly>';
    h += '<button type="button" class="eq-step-btn" data-action="step" data-selector=".eq-sub-add[data-idx=\'' + i + '\']" data-delta="-1" data-min="0" data-max="4">−</button>';
    h += '<input class="form-input eq-sub-add prop-value-stepper" data-idx="' + i + '" value="' + extraVal + '" min="0" max="4">';
    h += '<button type="button" class="eq-step-btn" data-action="step" data-selector=".eq-sub-add[data-idx=\'' + i + '\']" data-delta="1" data-min="0" data-max="4">+</button>';
    h += '</div>';
  }

  h += '<div class="enhance-sum" id="edit-enhance-sum">追加强化总和: <strong>...</strong> / 5</div>';
  h += '</div>';
  h += '<div class="editor-fab-group">';
  h += '<button class="btn btn-danger" data-action="delete-equip" data-id="' + euid + '">删除</button>';
  h += '<button class="btn btn-primary" data-action="save-equip" data-id="' + euid + '">保存更改</button>';
  h += '</div>';
  h += '</div>';

  // Store metadata for post-render DOM binding (Task 4.5)
  state._equipMeta = { subOpts: subOpts, isFixed: isFixed, slotOpts: slotOpts };

  return h;
}

// ═══════════════════════════════════════════════════════
// Post-render DOM binding (Task 4.5 — called after innerHTML)
// ═══════════════════════════════════════════════════════
function initEquipEditorDOM() {
  var meta = state._equipMeta;
  if (!meta) return;
  bindEditEnhanceListeners();
  if (!meta.isFixed) bindEditMainStatChange(meta.slotOpts);
  bindEditSubStatChanges(meta.subOpts);
  updateEditEnhanceSum();
  state._equipMeta = null;
}

// ═══════════════════════════════════════════════════════
// Save / Delete Equip
// ═══════════════════════════════════════════════════════
async function saveEquip(euid) {
  var mainKey = parseInt((document.getElementById('eq-prop-key') && document.getElementById('eq-prop-key').value) || '0');
  var mainBase = parseInt((document.getElementById('eq-prop-base') && document.getElementById('eq-prop-base').value) || '0');

  var subProps = [];
  var totalExtra = 0;
  for (var i = 0; i < 4; i++) {
    var keyEl = document.querySelector('.eq-sub-key[data-idx="' + i + '"]');
    var addEl = document.querySelector('.eq-sub-add[data-idx="' + i + '"]');
    var baseEl = document.querySelector('.eq-sub-base[data-idx="' + i + '"]');
    var key = parseInt((keyEl && keyEl.value) || '0');
    if (key > 0) {
      var extra = parseInt((addEl && addEl.value) || '0');
      totalExtra += extra;
      subProps.push({
        key: key,
        base_value: parseInt((baseEl && baseEl.value) || '0'),
        add_value: extra + 1
      });
    } else {
      subProps.push(null);
    }
  }

  if (totalExtra < 4 || totalExtra > 5) {
    toast('副词条追加强化总和必须为4-5，当前为' + totalExtra, 'error');
    return;
  }

  // Check duplicate sub stats
  var subKeys = subProps.filter(function(p) { return p !== null; }).map(function(p) { return p.key; });
  if (new Set(subKeys).size !== subKeys.length) {
    toast('副词条种类不能重复', 'error');
    return;
  }

  await safeSave('驱动盘数据', async function() {
    var result = await API.updateEquip(state.uid, euid, {
      level: parseInt(document.getElementById('eq-level').value),
      star: parseInt(document.getElementById('eq-star').value),
      properties: [{ key: mainKey, base_value: mainBase, add_value: 0 }],
      sub_properties: subProps
    });
    if (result && result.ok === false) throw new Error(result.error || '保存失败');
    markCacheDirty();
    state.equipView = 'gallery';
    state.selectedEquipUid = null;
    renderEquips();
  });
}

async function deleteEquip(euid) {
  showConfirm('确定删除驱动盘 #' + euid + '？此操作不可逆。', async function() {
    await safeSave('驱动盘', async function() {
      await API.deleteEquip(state.uid, euid);
      markCacheDirty();
      state.selectedEquipUid = null;
      state.equipView = 'gallery';
      renderEquips();
      loadCounts();
    });
  });
}

// ═══════════════════════════════════════════════════════
// Create Equip — Two-Step Flow
// ═══════════════════════════════════════════════════════
async function showCreateEquip() {
  var suitGroups = (state.templates && state.templates.suit_groups) || {};
  var suitNames = Object.keys(suitGroups).sort();

  var html = '<div class="modal-overlay" id="create-modal" role="dialog" aria-modal="true" data-action="modal-close">' +
    '<div class="modal modal--wide">' +
      '<div class="create-steps"><span class="create-step active">1. 选择套装</span><span class="create-step-arrow">→</span><span class="create-step">2. 选择位置</span><span class="create-step-arrow">→</span><span class="create-step">3. 配置属性</span></div>' +
      '<h2>创建驱动盘 — <span id="step-label" class="text-accent">选择套装</span></h2>' +
      '<div id="create-step1">' +
        '<div class="suit-grid-scroll">';

  for (var ni = 0; ni < suitNames.length; ni++) {
    var suitName = suitNames[ni];
    var slots = suitGroups[suitName] || [];
    html += '<div class="suit-card" data-action="select-create-suit" data-suit="' + suitName.replace(/"/g, '&quot;') + '">' +
      '<div class="suit-card__name">' + escHtml(suitName) + '</div>' +
      '<div class="suit-card__slots">' + slots.map(function(s) { return s.slot_name; }).join(' · ') + '</div>' +
    '</div>';
  }

  html += '</div></div>' +
    '<div id="create-step2" class="hidden"></div>' +
    '<div class="btn-group mt-4">' +
      '<button class="btn btn-ghost" data-action="close-create-equip">取消</button>' +
    '</div>' +
  '</div></div>';

  var container = document.createElement('div');
  container.id = 'modal-container';
  container.innerHTML = html;
  document.body.appendChild(container);
}

function selectCreateSuit(suitName) {
  var suitGroups = (state.templates && state.templates.suit_groups) || {};
  var slots = suitGroups[suitName] || [];

  document.getElementById('create-step1').style.display = 'none';
  document.getElementById('step-label').textContent = '选择位置 — ' + suitName;
  // Update step indicator
  var steps = document.querySelectorAll('.create-step');
  if (steps.length >= 2) { steps[0].classList.remove('active'); steps[1].classList.add('active'); }

  var html = '<button class="btn btn-ghost mb-3" data-action="back-to-suit-select">← 返回选择套装</button>';
  html += '<div class="slot-grid">';
  for (var si = 0; si < slots.length; si++) {
    var slot = slots[si];
    html += '<div class="slot-card" data-action="select-create-slot" data-eqid="' + slot.id + '" data-slot="' + slot.slot + '" data-suit="' + suitName.replace(/"/g, '&quot;') + '" data-slotname="' + slot.slot_name + '">' +
      '<div class="slot-card__num">' + slot.slot_name + '</div>' +
    '</div>';
  }
  html += '</div>';
  var step2 = document.getElementById('create-step2');
  step2.innerHTML = html;
  step2.style.display = 'block';
  // Re-trigger modal animation
  var modal = document.querySelector('.modal');
  if (modal) { modal.style.animation = 'none'; void modal.offsetHeight; modal.style.animation = ''; }
}

function selectCreateSlot(equipId, slot, suitName, slotName) {
  var mainOpts = (state.templates && state.templates.main_stat_options) || {};
  var subOpts = (state.templates && state.templates.sub_stat_options) || [];
  var slotOpts = mainOpts[String(slot)] || [];
  var isFixed = (slot >= 1 && slot <= 3);

  // Update step indicator
  var steps = document.querySelectorAll('.create-step');
  if (steps.length >= 3) { steps[1].classList.remove('active'); steps[2].classList.add('active'); }

  var html = '<button class="btn btn-ghost mb-3" data-action="back-to-slot-select" data-suit="' + escHtml(suitName) + '">← 返回选择位置</button>';
  html += '<div class="section-title">' + suitName + ' · ' + slotName + '号位 · 主属性</div>';
  html += '<div class="prop-row"><select id="new-eq-main-key" class="form-select form-select--flex2"' + (isFixed ? ' disabled' : '') + '>';
  for (var oi = 0; oi < slotOpts.length; oi++) {
    var opt = slotOpts[oi];
    html += '<option value="' + opt.key + '" data-base="' + opt.base_value + '">' + opt.name + '</option>';
  }
  html += '</select>';
  html += '<input class="form-input prop-value-readonly" id="new-eq-main-base" value="' + (slotOpts[0] ? slotOpts[0].base_value || 0 : 0) + '" readonly>';
  html += '<span class="text-muted prop-add-label">+0</span>';
  html += '</div>';
  if (isFixed) {
    html += '<div class="text-sm text-muted hint-text">1-3号位主属性固定，不可更改</div>';
  }

  html += '<div class="section-title">副词条 · 4 条 <span class="text-sm text-muted">(追加强化 0-4)</span></div>';
  html += '<div class="prop-header"><span>#</span><span>属性</span><span>基础值</span><span>强化</span></div>';
  for (var i = 0; i < 4; i++) {
    html += '<div class="prop-row">';
    html += '<span class="prop-index">' + (i + 1) + '</span>';
    html += '<select class="form-select new-eq-sub-key form-select--flex2" data-idx="' + i + '">';
    html += '<option value="0">— 无 —</option>';
    for (var si = 0; si < subOpts.length; si++) {
      var sopt = subOpts[si];
      html += '<option value="' + sopt.key + '" data-base="' + sopt.base_value + '">' + sopt.name + '</option>';
    }
    html += '</select>';
    html += '<input class="form-input new-eq-sub-base prop-value-readonly" data-idx="' + i + '" value="0" readonly>';
    html += '<button type="button" class="eq-step-btn" data-action="step" data-selector=".new-eq-sub-add[data-idx=\'' + i + '\']" data-delta="-1" data-min="0" data-max="4">−</button>';
    html += '<input class="form-input new-eq-sub-add prop-value-stepper" data-idx="' + i + '" value="1" min="0" max="4">';
    html += '<button type="button" class="eq-step-btn" data-action="step" data-selector=".new-eq-sub-add[data-idx=\'' + i + '\']" data-delta="1" data-min="0" data-max="4">+</button>';
    html += '</div>';
  }

  html += '<div class="enhance-sum enhance-sum--valid" id="create-enhance-sum">追加强化总和: <strong>4</strong> / 5</div>';
  html += '<div class="btn-group"><button class="btn btn-success btn-lg" data-action="create-equip" data-id="' + equipId + '">创建驱动盘</button></div>';
  var step2 = document.getElementById('create-step2');
  step2.innerHTML = html;
  // Re-trigger modal animation
  var modal = document.querySelector('.modal');
  if (modal) { modal.style.animation = 'none'; void modal.offsetHeight; modal.style.animation = ''; }
  bindCreateEnhanceListeners();
  if (!isFixed) bindCreateMainStatChange(slotOpts);
}

function closeCreateEquip() {
  var el = document.getElementById('modal-container');
  if (el) el.remove();
}

// ═══════════════════════════════════════════════════════
// Edit Equip Bind Helpers
// ═══════════════════════════════════════════════════════
function bindEditEnhanceListeners() {
  for (let i = 0; i < 4; i++) {
    var add = document.querySelector('.eq-sub-add[data-idx="' + i + '"]');
    if (add) add.addEventListener('input', updateEditEnhanceSum);
  }
}

function bindEditMainStatChange(slotOpts) {
  var sel = document.getElementById('eq-prop-key');
  if (!sel) return;
  sel.addEventListener('change', function() {
    var base = (sel.selectedOptions[0] && sel.selectedOptions[0].dataset && sel.selectedOptions[0].dataset.base) || '0';
    document.getElementById('eq-prop-base').value = base;
    updateEditEnhanceSum();
  });
}

function bindEditSubStatChanges(subOpts) {
  for (let i = 0; i < 4; i++) {
    var sel = document.querySelector('.eq-sub-key[data-idx="' + i + '"]');
    if (sel) {
      sel.addEventListener('change', function() {
        var idx = parseInt(this.dataset.idx);
        var base = (this.selectedOptions[0] && this.selectedOptions[0].dataset && this.selectedOptions[0].dataset.base) || '0';
        var baseEl = document.querySelector('.eq-sub-base[data-idx="' + idx + '"]');
        if (baseEl) baseEl.value = base;
        updateEditEnhanceSum();
      });
    }
  }
}

function updateEditEnhanceSum() {
  var sum = 0;
  for (var i = 0; i < 4; i++) {
    var keyEl = document.querySelector('.eq-sub-key[data-idx="' + i + '"]');
    if (keyEl && parseInt(keyEl.value || '0') === 0) continue;
    var el = document.querySelector('.eq-sub-add[data-idx="' + i + '"]');
    sum += parseInt((el && el.value) || '0');
  }
  var sumEl = document.getElementById('edit-enhance-sum');
  if (sumEl) {
    sumEl.innerHTML = '强化值总和: <strong>' + sum + '</strong> / 5';
    sumEl.className = 'enhance-sum' + (sum < 4 || sum > 5 ? ' enhance-sum--invalid' : ' enhance-sum--valid');
  }
}

// ═══════════════════════════════════════════════════════
// Create Equip Bind Helpers
// ═══════════════════════════════════════════════════════
function bindCreateMainStatChange(slotOpts) {
  var sel = document.getElementById('new-eq-main-key');
  if (!sel) return;
  sel.addEventListener('change', function() {
    var base = (sel.selectedOptions[0] && sel.selectedOptions[0].dataset && sel.selectedOptions[0].dataset.base) || '0';
    document.getElementById('new-eq-main-base').value = base;
    updateCreateEnhanceSum();
  });
}

function bindCreateEnhanceListeners() {
  for (let i = 0; i < 4; i++) {
    var sel = document.querySelector('.new-eq-sub-key[data-idx="' + i + '"]');
    var add = document.querySelector('.new-eq-sub-add[data-idx="' + i + '"]');
    if (sel) {
      sel.addEventListener('change', function() {
        var idx = parseInt(this.dataset.idx);
        var base = (this.selectedOptions[0] && this.selectedOptions[0].dataset && this.selectedOptions[0].dataset.base) || '0';
        var baseEl = document.querySelector('.new-eq-sub-base[data-idx="' + idx + '"]');
        if (baseEl) baseEl.value = base;
        updateCreateEnhanceSum();
      });
    }
    if (add) add.addEventListener('input', updateCreateEnhanceSum);
  }
}

function updateCreateEnhanceSum() {
  var sum = 0;
  for (var i = 0; i < 4; i++) {
    var keyEl = document.querySelector('.new-eq-sub-key[data-idx="' + i + '"]');
    if (keyEl && parseInt(keyEl.value || '0') === 0) continue;
    var el = document.querySelector('.new-eq-sub-add[data-idx="' + i + '"]');
    sum += parseInt((el && el.value) || '0');
  }
  var sumEl = document.getElementById('create-enhance-sum');
  if (sumEl) {
    sumEl.innerHTML = '强化值总和: <strong>' + sum + '</strong> / 5';
    sumEl.className = 'enhance-sum' + (sum < 4 || sum > 5 ? ' enhance-sum--invalid' : ' enhance-sum--valid');
  }
}

// ═══════════════════════════════════════════════════════
// Create Equip — Final Submit
// ═══════════════════════════════════════════════════════
async function createEquip(equipId) {
  var mainKeyEl = document.getElementById('new-eq-main-key');
  var mainKey = parseInt(mainKeyEl ? mainKeyEl.value || '0' : '0');
  var mainBaseEl = document.getElementById('new-eq-main-base');
  var mainBase = parseInt(mainBaseEl ? mainBaseEl.value || '0' : '0');

  var subProps = [];
  var totalExtra = 0;
  for (var i = 0; i < 4; i++) {
    var keyEl = document.querySelector('.new-eq-sub-key[data-idx="' + i + '"]');
    var addEl = document.querySelector('.new-eq-sub-add[data-idx="' + i + '"]');
    var baseEl = document.querySelector('.new-eq-sub-base[data-idx="' + i + '"]');
    var key = parseInt((keyEl && keyEl.value) || '0');
    if (key > 0) {
      var extra = parseInt((addEl && addEl.value) || '0');
      totalExtra += extra;
      subProps.push({
        key: key,
        base_value: parseInt((baseEl && baseEl.value) || '0'),
        add_value: extra + 1
      });
    } else {
      subProps.push(null);
    }
  }

  if (totalExtra < 4 || totalExtra > 5) {
    toast('副词条追加强化总和必须为4-5，当前为' + totalExtra, 'error');
    return;
  }

  var subKeys = subProps.filter(function(p) { return p !== null; }).map(function(p) { return p.key; });
  if (new Set(subKeys).size !== subKeys.length) {
    toast('副词条种类不能重复', 'error');
    return;
  }

  var body = {
    id: equipId, level: 15, star: 5,
    properties: [{ key: mainKey, base_value: mainBase, add_value: 0 }],
    sub_properties: subProps
  };
  await safeSave('驱动盘', async function() {
    var result = await API.createEquip(state.uid, body);
    closeCreateEquip();
    toast('驱动盘 #' + result.uid + ' 已创建');
    markCacheDirty();
    loadCounts();
    renderEquips();
  });
}
