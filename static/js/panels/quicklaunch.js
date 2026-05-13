// ═══════════════════════════════════════════════════════
// Quick Launch Panel — 快速启动
// ═══════════════════════════════════════════════════════

var LAUNCH_ITEMS = [
  { key: 'yoshunko', name: 'Yoshunko 服务端', desc: '游戏服务端 · 自动从 WSL 启动', auto: true },
  { key: 'hoyosdk', name: 'HoyoSDK', desc: 'SDK 服务 · hoyo-sdk.exe', auto: false },
  { key: 'kcpshim', name: 'KCPSHIM', desc: '桥接服务 · kcpshim.exe', auto: false },
  { key: 'yidhari', name: 'Yidhari 客户端', desc: '游戏客户端 · yidhari.exe（管理员身份）', auto: false }
];

async function renderQuickLaunch() {
  var cfg = {};
  try {
    var r = await API.getLaunchConfig();
    if (r && r.config) cfg = r.config;
  } catch(e) {}

  var allConfigured = cfg.hoyosdk && cfg.kcpshim && cfg.yidhari;

  var h = '<div class="page-header"><h2>快速启动</h2><span class="subtitle">一键启动游戏相关服务</span></div>';

  // Status bar showing configured count
  var configured = (cfg.hoyosdk ? 1 : 0) + (cfg.kcpshim ? 1 : 0) + (cfg.yidhari ? 1 : 0);
  if (configured < 3) {
    h += '<div class="launch-status-bar"><span class="launch-status-icon">' + configured + '/3</span> 三件套路径已配置，全部配置后可使用一键启动</div>';
  } else {
    h += '<div class="launch-status-bar launch-status-bar--ready">全部路径已配置，可以使用一键启动</div>';
  }

  h += '<div class="launch-grid">';
  for (var i = 0; i < LAUNCH_ITEMS.length; i++) {
    var item = LAUNCH_ITEMS[i];
    var saved = cfg[item.key];
    var hasPath = !!saved;
    h += '<div class="launch-card' + (item.auto || hasPath ? ' launch-card--ready' : '') + '" style="animation-delay:' + (i * 0.08) + 's">';
    // Status dot
    h += '<div class="launch-card__dot ' + (item.auto || hasPath ? 'launch-card__dot--on' : '') + '"></div>';
    h += '<div class="launch-card__body">';
    h += '<div class="launch-card__header">';
    h += '<span class="launch-card__name">' + escHtml(item.name) + '</span>';
    if (item.auto) {
      h += '<span class="launch-card__badge launch-card__badge--auto">自动</span>';
    } else if (hasPath) {
      h += '<span class="launch-card__badge launch-card__badge--ok">已配置</span>';
    }
    h += '</div>';
    h += '<div class="launch-card__desc">' + escHtml(item.desc) + '</div>';
    if (hasPath) {
      h += '<div class="launch-card__path">' + escHtml(saved) + '</div>';
    }
    h += '</div>';
    h += '<div class="launch-card__actions">';
    if (item.auto) {
      h += '<button class="btn btn-success launch-btn" data-action="launch-yoshunko">启动</button>';
    } else if (hasPath) {
      h += '<button class="btn btn-primary launch-btn" data-action="launch-program" data-key="' + item.key + '">启动</button>';
      h += '<button class="btn btn-ghost launch-config-btn" data-action="edit-launch-path" data-key="' + item.key + '">修改</button>';
    } else {
      h += '<div class="launch-card__input-wrap">' +
        '<div class="launch-input-row">' +
        '<input type="text" class="form-input launch-path-input" id="path-input-' + item.key + '" placeholder="例如: D:\\3.0.1\\tools\\' + item.key + '.exe">' +
        '<button class="btn btn-ghost launch-paste-btn" data-action="paste-launch-path" data-key="' + item.key + '" title="从剪贴板粘贴">粘贴</button>' +
        '</div>' +
        '<button class="btn btn-primary btn-sm launch-input-btn" data-action="save-launch-path" data-key="' + item.key + '">确定</button>' +
      '</div>';
    }
    h += '</div></div>';
  }
  h += '</div>';
  $('#main').innerHTML = h;

  applyStaggeredAnimation('.launch-card');

  // Floating "Launch All" button
  if (allConfigured) {
    var fab = document.createElement('button');
    fab.className = 'btn btn-primary editor-fab launch-all-fab';
    fab.innerHTML = '&#9654; 一键启动';
    fab.setAttribute('data-action', 'launch-all');
    document.body.appendChild(fab);
  }
}

async function saveLaunchPath(key) {
  var input = document.getElementById('path-input-' + key);
  if (!input) return;
  var path = input.value.trim().replace(/^["']+|["']+$/g, ''); // strip quotes
  if (!path) { toast('请输入路径'); return; }
  // Validate file exists on backend
  var result = await API.setLaunchPath(key, path);
  if (result && result.ok) {
    toast('路径已保存');
    renderQuickLaunch();
  } else {
    toast('保存失败: ' + (result ? result.error : '未知错误'), 'error');
  }
}

async function editLaunchPath(key) {
  var current = '';
  try {
    var r = await API.getLaunchConfig();
    if (r && r.config && r.config[key]) current = r.config[key];
  } catch(e) {}

  var card = document.querySelector('[data-key="' + key + '"]');
  if (!card) return;
  card = card.closest('.launch-card');
  if (!card) return;

  var pathEl = card.querySelector('.launch-card__path');
  var actionsEl = card.querySelector('.launch-card__actions');
  if (!actionsEl) return;

  // Hide current buttons, show input
  actionsEl.innerHTML = '<div class="launch-card__input-wrap">' +
    '<input type="text" class="form-input launch-path-input" id="path-input-' + key + '" value="' + escHtml(current) + '" placeholder="例如: D:\\3.0.1\\' + key + '.exe">' +
    '<button class="btn btn-primary btn-sm launch-input-btn" data-action="save-launch-path" data-key="' + key + '">确定</button>' +
  '</div>';
}

function launchByPath(key) {
  try {
    if (key === 'yidhari') {
      API.launchProgramAdmin(key); toast('Yidhari 正在以管理员身份启动...');
    } else {
      API.launchProgram(key); toast('已启动');
    }
  } catch(e) { toast('启动失败', 'error'); }
}

function launchYoshunko() {
  try { API.launchYoshunko(); toast('Yoshunko 正在启动...'); } catch(e) { toast('启动失败', 'error'); }
}

async function launchAll() {
  try {
    await API.launchYoshunko();
    var cfg = {};
    try { var r = await API.getLaunchConfig(); if (r && r.config) cfg = r.config; } catch(e) {}
    var keys = ['hoyosdk', 'kcpshim', 'yidhari'];
    for (var i = 0; i < keys.length; i++) {
      if (cfg[keys[i]]) {
        if (keys[i] === 'yidhari') {
          await API.launchProgramAdmin(keys[i]);
        } else {
          await API.launchProgram(keys[i]);
        }
        await new Promise(function(r) { setTimeout(r, 300); });
      }
    }
    toast('全部启动完成');
  } catch(e) {
    toast('启动失败: ' + escHtml(e.message || String(e)), 'error');
  }
}
