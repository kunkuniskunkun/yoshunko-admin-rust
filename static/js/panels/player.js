// ═══════════════════════════════════════════════════════
// Player Info Panel + Setup Page
// ═══════════════════════════════════════════════════════

async function renderPlayerInfo() {
  var info = await API.getPlayer(state.uid);
  if (!info) { $('#main').innerHTML = '<div class="empty-state"><p>未找到玩家信息</p></div>'; return; }

  var html = '<div class="page-header"><h2>玩家信息</h2><span class="subtitle">修改基础账号信息</span></div>';

  html += '<div class="panel-box panel-narrow"><div class="panel-box__body">';
  html += '<div class="form-row">';
  html += '<div class="form-field"><label class="form-label">昵称</label><input class="form-input" type="text" id="pi-nickname" value="' + escHtml(info.nickname) + '"></div>';
  html += '<div class="form-field"><label class="form-label">等级</label><input class="form-input" type="number" id="pi-level" value="' + info.level + '" min="1" max="60"></div>';
  html += '</div>';
  html += '<div class="form-field"><label class="form-label">经验</label><input class="form-input" type="number" id="pi-exp" value="' + info.exp + '"></div>';
  html += '<div class="form-row-3">';
  html += '<div class="form-field"><label class="form-label">展示角色 ID</label><input class="form-input" type="number" id="pi-avatar" value="' + info.avatar_id + '"></div>';
  html += '<div class="form-field"><label class="form-label">操控角色 ID</label><input class="form-input" type="number" id="pi-control" value="' + info.control_avatar_id + '"></div>';
  html += '<div class="form-field"><label class="form-label">伪装角色 ID</label><input class="form-input" type="number" id="pi-guise" value="' + info.control_guise_avatar_id + '"></div>';
  html += '</div>';
  html += '<div class="btn-group"><button class="btn btn-primary" data-action="save-player-info">保存更改</button><button class="btn btn-ghost" data-action="export-player">导出</button><button class="btn btn-ghost" data-action="import-player">导入</button></div>';
  html += '</div></div>';

  $('#main').innerHTML = html;
}

async function exportPlayerData() {
  try {
    var r = await API.exportData(state.uid);
    if (!r.ok) { toast('导出失败', 'error'); return; }
    var blob = new Blob([JSON.stringify(r.data, null, 2)], {type: 'application/json'});
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'yoshunko_player_' + state.uid + '_' + new Date().toISOString().slice(0,10) + '.json';
    a.click();
    URL.revokeObjectURL(a.href);
    toast('数据已导出');
  } catch(e) { toast('导出失败: ' + e.message, 'error'); }
}

async function importPlayerData() {
  var input = document.createElement('input');
  input.type = 'file'; input.accept = '.json';
  input.onchange = async function() {
    var file = input.files[0];
    if (!file) return;
    try {
      var text = await file.text();
      var json = JSON.parse(text);
      if (!json.uid) { toast('无效的导出文件', 'error'); return; }
      showConfirm('导入将覆盖当前玩家 ' + state.uid + ' 的数据，继续？', async function() {
        var r = await API.importData(state.uid, json);
        if (r.ok) { toast('已导入 ' + r.imported + ' 条数据'); loadCounts(); renderPlayerInfo(); }
        else { toast('导入失败', 'error'); }
      });
    } catch(e) { toast('导入失败: ' + e.message, 'error'); }
  };
  input.click();
}
async function savePlayerInfo() {
  await safeSave('玩家信息', async function() {
    await API.updatePlayer(state.uid, {
      nickname: $('#pi-nickname').value,
      level: parseInt($('#pi-level').value),
      exp: parseInt($('#pi-exp').value),
      avatar_id: parseInt($('#pi-avatar').value),
      control_avatar_id: parseInt($('#pi-control').value),
      control_guise_avatar_id: parseInt($('#pi-guise').value)
    });
  });
}

// ═══════════════════════════════════════════════════════
// Setup Page
// ═══════════════════════════════════════════════════════
async function renderSetup() {
  var html = '<div class="setup-page"><div class="setup-card">';
  html += '<h1>Yoshunko Admin</h1>';
  html += '<p class="setup-version" id="setup-version">V0.506</p>';
  html += '<p class="setup-desc">首次使用，请配置 yoshunko state 目录路径</p>';
  html += '<div class="setup-input-wrap">';
  html += '<label>State 目录</label>';
  html += '<input type="text" id="setup-state-dir" placeholder="例如: D:\\3.0.1\\state" />';
  html += '<div id="setup-candidates"></div></div>';
  html += '<div id="setup-error" class="setup-error"></div>';
  html += '<button class="setup-btn" data-action="setup-connect">连接</button>';
  html += '</div></div>';
  document.getElementById('main').innerHTML = html;

  // Auto-detect
  var r = await API.autoDetectPaths();
  if (r.candidates && r.candidates.length > 0) {
    var candHtml = '<p class="text-sm text-muted mb-1 mt-2">检测到的路径（点击填入）：</p>';
    r.candidates.forEach(function(p) {
      candHtml += '<div class="candidate-path" data-action="candidate-path" data-path="' + escHtml(p) + '">' + escHtml(p) + '</div>';
    });
    document.getElementById('setup-candidates').innerHTML = candHtml;
    document.getElementById('setup-state-dir').value = r.candidates[0];
  }
}

async function onSetupConnect() {
  var path = document.getElementById('setup-state-dir').value.trim();
  // Strip surrounding quotes
  path = path.replace(/^["']+|["']+$/g, '');
  if (!path) return;
  // Pre-validate path format
  if (!/[a-zA-Z]:[\\/]|^\/|^\\\\/.test(path)) {
    var el = document.getElementById('setup-error');
    el.textContent = '路径格式不正确，请输入有效的绝对路径（如 D:\\3.0.1\\state 或 //wsl/...）';
    el.classList.add('visible');
    return;
  }
  var btn = document.querySelector('.setup-btn');
  btn.disabled = true; btn.textContent = '连接中...';
  var result = await API.setStateDir(path);
  btn.disabled = false; btn.textContent = '连接';
  if (result.ok) {
    $('#main').innerHTML = '<div class="loading-wrap"><div class="spinner"></div><span>正在启动...</span></div>';
    await new Promise(function(resolve) { setTimeout(resolve, 100); });
    initApp();
  } else {
    var el = document.getElementById('setup-error');
    el.textContent = result.error;
    el.classList.add('visible');
  }
}
