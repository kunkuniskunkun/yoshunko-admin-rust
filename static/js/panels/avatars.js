// ═══════════════════════════════════════════════════════
// Avatars Panel — 角色仓库
// ═══════════════════════════════════════════════════════

async function renderAvatars() {
  // Use cache if available
  if (state._avatarCache && !state._cacheDirty) {
    var avatars = state._avatarCache;
  } else {
    var data = await API.getAvatars(state.uid);
    state._avatarCache = data.avatars;
    state._cacheDirty = false;
    var avatars = data.avatars;
  }

  // Filter out alt characters (2071, 2121)
  avatars = avatars.filter(function(av) { return av.avatar_id !== 2071 && av.avatar_id !== 2121; });

  // -- Search filter --
  var q = (state.avatarSearch || '').toLowerCase();
  if (q) {
    avatars = avatars.filter(function(av) {
      var t = state.avatarMap ? state.avatarMap.get(av.avatar_id) : null;
      if (!t && state.templates && state.templates.avatars) {
        t = state.templates.avatars.find(function(a) { return a.id === av.avatar_id; });
      }
      var campName = (t && t.camp_name ? t.camp_name : '').toLowerCase();
      var campIdStr = t ? String(t.camp_id || '') : '';
      var profName = (av.profession || '').toLowerCase();
      // Pinyin matching
      var py = (typeof AVATAR_PINYIN !== 'undefined' && AVATAR_PINYIN[av.avatar_id]) ? AVATAR_PINYIN[av.avatar_id] : null;
      var pinyinMatch = py && (py.full.includes(q) || py.initials.includes(q));
      return String(av.avatar_id).includes(q)
        || (av.name || '').toLowerCase().includes(q)
        || campName.includes(q)
        || campIdStr.includes(q)
        || profName.includes(q)
        || pinyinMatch;
    });
  }

  // -- Group by camp or profession --
  var groupBy = state.avatarGroupBy || 'camp';
  var groups = {};
  for (var i = 0; i < avatars.length; i++) {
    var av = avatars[i];
    var t = state.avatarMap ? state.avatarMap.get(av.avatar_id) : null;
    if (!t && state.templates && state.templates.avatars) {
      t = state.templates.avatars.find(function(a) { return a.id === av.avatar_id; });
    }
    var groupId, groupName;
    if (groupBy === 'profession') {
      groupId = av.profession || '未知';
      groupName = groupId;
    } else {
      groupId = (t && t.camp_id != null) ? t.camp_id : 0;
      groupName = (t && t.camp_name) ? t.camp_name : '营' + groupId;
    }
    if (!groups[groupId]) {
      groups[groupId] = { group_name: groupName, avatars: [] };
    }
    groups[groupId].avatars.push(av);
  }

  // Sort by avatar_id within each group
  Object.keys(groups).forEach(function(key) {
    groups[key].avatars.sort(function(a, b) { return a.avatar_id - b.avatar_id; });
  });

  // Sort groups
  var sortedGroups;
  if (groupBy === 'profession') {
    var profOrder = ['强攻', '击破', '异常', '支援', '防护', '命破'];
    sortedGroups = Object.keys(groups).sort(function(a, b) {
      var ia = profOrder.indexOf(a); if (ia < 0) ia = 999;
      var ib = profOrder.indexOf(b); if (ib < 0) ib = 999;
      return ia - ib;
    });
  } else {
    sortedGroups = Object.keys(groups).map(Number).sort(function(a, b) { return a - b; });
  }

  // ── Editor Mode ──
  if (state.avatarView === 'editor' && state.selectedAvatarId) {
    $('#main').innerHTML = await avatarEditorHTML(state.selectedAvatarId);
    return;
  }

  // ── Gallery Mode ──
  state.avatarView = 'gallery';

  var html = '<div class="page-header"><h2>角色仓库</h2><span class="subtitle">管理等级、影画、技能、潜能激发</span></div>';

  // Search bar with group toggle
  var groupLabel = groupBy === 'profession' ? '按职业' : '按阵营';
  html += '<div class="search-bar-row"><div class="search-wrap"><input type="text" id="avatar-search" placeholder="搜索角色 ID、名称、阵营或职业..." aria-label="搜索角色" value="' + escHtml(state.avatarSearch || '') + '" data-action="search-avatars"><span class="search-count" id="avatar-search-count"></span>' + (state.avatarSearch ? '<button class="search-clear" data-action="clear-avatar-search" aria-label="清除搜索">×</button>' : '') + '</div><button class="search-group-toggle" data-action="toggle-avatar-group" title="切换分组：' + groupLabel + '">' + groupLabel + '</button></div>';

  if (avatars.length === 0) {
    html += '<div class="empty-state"><div class="empty-state__icon"></div><p>没有找到匹配的角色</p></div>';
    $('#main').innerHTML = html;
  applyStaggeredAnimation('.avatar-gallery__card');
    return;
  }

  // Gallery grid grouped by camp or profession
  html += '<div class="avatar-gallery">';
  for (var ci = 0; ci < sortedGroups.length; ci++) {
    var groupId = sortedGroups[ci];
    var grp = groups[groupId];
    html += '<div class="avatar-gallery__camp-section">';
    html += '<div class="avatar-gallery__camp-header">' + escHtml(grp.group_name) + ' <span class="text-xs text-muted">(' + grp.avatars.length + ')</span></div>';
    html += '<div class="avatar-gallery__grid">';
    for (var ai = 0; ai < grp.avatars.length; ai++) {
      var av = grp.avatars[ai];
      var rarity = av.rarity || '?';
      var rarityCls = rarity === 'S' ? 'rarity-s' : 'rarity-a';
      var cardCls = rarity === 'S' ? 'avatar-gallery__card--s' : 'avatar-gallery__card--a';
      var favStarHtml = av.is_favorite ? '<div class="avatar-card__fav">★</div>' : '';
      var t2 = state.avatarMap ? state.avatarMap.get(av.avatar_id) : null;
      if (!t2 && state.templates && state.templates.avatars) {
        t2 = state.templates.avatars.find(function(a) { return a.id === av.avatar_id; });
      }
      var campName = (t2 && t2.camp_name) ? t2.camp_name : '';
      html += '<div class="game-card avatar-gallery__card ' + cardCls + '" tabindex="0" role="button" data-action="select-avatar" data-id="' + av.avatar_id + '">' +
        favStarHtml +
        '<div class="card-header">' +
          '<span><span class="game-card__rarity ' + rarityCls + '">' + rarity + '</span><span class="card-title">' + escHtml(av.name) + '</span></span>' +
          '<span class="game-card__level">Lv.' + av.level + '</span>' +
        '</div>' +
        '<div class="card-meta">' +
          '<span>' + renderRankDots(av.unlocked_talent_num || 0) + '</span>' +
          '<span class="text-sm text-accent ml-1\.5">' + (av.unlocked_talent_num || 0) + '影</span>' +
        '</div>' +
        '<div class="avatar-card__footer">' +
          '<span class="avatar-card__camp-tag">' + escHtml(campName) + '</span>' +
          (av.profession ? '<span class="avatar-card__camp-tag ml-1">' + escHtml(av.profession) + '</span>' : '') +
          '<span class="text-xs text-muted">ID:' + av.avatar_id + '</span>' +
        '</div>' +
      '</div>';
    }
    html += '</div></div>';
  }
  html += '</div>';

  $('#main').innerHTML = html;
  applyStaggeredAnimation('.avatar-gallery__card');
}

var onAvatarSearch = createSearchHandler('avatarSearch', 'avatarView', renderAvatars, 'selectedAvatarId');
function selectAvatar(id) {
  state._scrollPos.avatars = $('#main').scrollTop;
  state.selectedAvatarId = id;
  state.avatarView = 'editor';
  renderAvatars().then(function() {
    var m = $('#main');
    m.classList.remove('editor-slide-in');
    void m.offsetHeight;
    m.classList.add('editor-slide-in');
  });
}

function backToGallery() {
  state.avatarView = 'gallery';
  state.selectedAvatarId = null;
  renderAvatars();
  var m = $('#main');
  m.style.scrollBehavior = 'auto';
  m.scrollTop = state._scrollPos.avatars || 0;
  m.style.scrollBehavior = '';
}

async function avatarEditorHTML(aid) {
  var av = await API.getAvatar(state.uid, aid);
  state.skillTypes[aid] = av.skill_type_level.map(function(s) { return s.type; });
  state.skillData[aid] = av.skill_type_level;

  var skillNames = ['普攻', '强化特殊技', '闪避', '连携技', '终结技', null, '支援技'];

  // Compute expected rank from level
  var level = av.level;
  var expectedRank = level === 0 ? 0 : Math.min(6, Math.max(1, Math.floor((level - 1) / 10) + 1));

  var h = '<div class="editor-page">';
  h += '<div class="editor-page__top">';
  h += '<span class="back-link" data-action="back-to-gallery">← 角色仓库</span>';
  h += '<div class="editor-page__header"><h2>' + escHtml(av.name) + '</h2>' + (av.en_name ? '<span class="sub en-name">' + escHtml(av.en_name) + '</span>' : '') + '<span class="sub">ID: ' + aid + '</span></div>';
  h += '</div>';
  h += '<div class="editor-page__body">';

  // Basic stats
  h += '<div class="section-title">基础属性</div>';
  h += '<div class="form-row">';
  h += '<div class="form-field"><label class="form-label">等级</label>' + stepperInput('av-level', av.level, 0, 60) + '</div>';
  h += '<div class="form-field"><label class="form-label">影画 (命座)</label>' + stepperInput('av-talent', av.unlocked_talent_num, 0, 6) + '</div>';
  h += '</div>';

  // Show auto-computed rank (read-only)
  h += '<div class="text-sm text-muted mt-2">等级 ' + level + ' → 自动突破等级 ' + expectedRank + ' (Lv上限 ' + (expectedRank * 10) + ')</div>';

  // Skills
  h += '<div class="section-title">技能等级</div>';
  h += '<div class="skill-grid">';
  for (var i = 0; i < Math.min(av.skill_type_level.length, 7); i++) {
    var sk = av.skill_type_level[i];
    if (sk.type === 'core_skill') continue;
    if (sk.type === 'unique_skill') continue; // 终结技与连携技重复，跳过
    var maxLv = 12;
    h += '<div class="skill-card">' +
      '<div class="skill-name">' + (skillNames[i] || sk.type) + '</div>' +
      '<input type="number" class="av-skill-lvl" data-idx="' + i + '" data-type="' + sk.type + '" value="' + sk.level + '" min="1" max="' + maxLv + '">' +
      '<div class="text-xs text-muted">max ' + maxLv + '</div>' +
    '</div>';
  }
  // 核心被动卡片（替代终结技位置）
  h += '<div class="skill-card">' +
    '<div class="skill-name">核心被动</div>' +
    '<input type="number" id="av-passive" value="' + av.passive_skill_level + '" min="0" max="6">' +
    '<div class="text-xs text-muted">max 6</div>' +
  '</div>';
  h += '</div>';

  // Potential Awakening (潜能激发) — awake_id format: avatar_id*100 + (level-1), 0=未解锁
  var awakeLevel = av.awake_id === 0 ? 0 : (av.awake_id % 100) + 1;
  h += '<div class="section-title">潜能激发</div>';
  h += '<div class="form-row">';
  h += '<div class="form-field"><label class="form-label">潜能激发等级 (0-6)</label>' + stepperInput('av-awake', awakeLevel, 0, 6) + '</div>';
  h += '<div class="form-field"><label class="form-label">当前武器 UID</label><input class="form-input form-input--short" type="number" id="av-weapon" value="' + av.cur_weapon_uid + '" min="0"></div>';
  h += '</div>';
  h += '<div class="form-row">';
  h += '<div class="form-field"><label class="form-label">皮肤 ID (0=默认)</label>' + stepperInput('av-skin', av.avatar_skin_id || 0, 0, 999999) + '</div>';
  h += '<div class="form-field"></div>';
  h += '</div>';

  h += '</div>';
  h += '<button class="btn btn-primary editor-fab" data-action="save-avatar" data-id="' + aid + '">保存更改</button>';
  h += '</div>';

  return h;
}

async function saveAvatar(aid) {
  await safeSave('角色数据', async function() {
    var skillLevels = [];
    $$('.av-skill-lvl').forEach(function(inp) {
      skillLevels.push({ type: inp.dataset.type, level: parseInt(inp.value) });
    });
    // unique_skill follows cooperate_skill level
    var coopSkill = skillLevels.find(function(s) { return s.type === 'cooperate_skill'; });
    var types = state.skillTypes[aid] || [];
    var uniqueIdx = types.indexOf('unique_skill');
    if (uniqueIdx >= 0 && coopSkill) {
      skillLevels.splice(uniqueIdx, 0, { type: 'unique_skill', level: coopSkill.level });
    }
    // core_skill = 1 + passive_skill_level
    var coreIdx = types.indexOf('core_skill');
    var passiveLevel = parseInt($('#av-passive').value) || 0;
    skillLevels.splice(coreIdx >= 0 ? coreIdx : 5, 0, { type: 'core_skill', level: 1 + passiveLevel });

    // Convert awake level back to awake_id format: avatar_id*100 + (level-1), 0=未解锁
    var awakeLvl = parseInt($('#av-awake').value) || 0;
    var awakeId = awakeLvl === 0 ? 0 : aid * 100 + (awakeLvl - 1);

    var body = {
      level: parseInt($('#av-level').value),
      passive_skill_level: parseInt($('#av-passive').value),
      unlocked_talent_num: parseInt($('#av-talent').value),
      skill_type_level: skillLevels,
      awake_id: awakeId,
      avatar_skin_id: parseInt($('#av-skin').value) || 0,
      cur_weapon_uid: parseInt($('#av-weapon').value) || 0
    };
    var result = await API.updateAvatar(state.uid, aid, body);
    if (result && result.ok === false) throw new Error(result.error || '保存失败');
    markCacheDirty();
    state.avatarView = 'gallery';
    state.selectedAvatarId = null;
    renderAvatars();
  });
}
