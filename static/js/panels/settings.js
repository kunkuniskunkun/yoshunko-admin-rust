// ═══════════════════════════════════════════════════════
// Settings Panel
// ═══════════════════════════════════════════════════════

async function renderSettings() {
  var cfg = await API.getConfig();
  var isDark = document.documentElement.getAttribute('data-theme') === 'dark';

  var html = '<div class="page-header"><h2>系统设置</h2><span class="subtitle">应用配置与偏好</span></div>';
  html += '<div class="settings-panel"><div class="panel-box__body">';

  // ── Data Management ──
  html += '<div class="section-title">数据管理</div>';
  html += '<div class="form-field"><label class="form-label">State 目录</label>';
  html += '<div class="input-group"><input class="form-input form-input--readonly" type="text" id="set-state-dir" value="' + escHtml(cfg.state_dir || '未配置') + '" readonly>';
  html += '<button class="btn btn-ghost" data-action="change-state-dir">更改</button></div>';
  html += '<p class="form-hint">游戏存档数据所在目录</p></div>';

  html += '<div class="btn-group" style="margin-top:8px;margin-bottom:16px">';
  html += '<button class="btn btn-ghost" data-action="auto-detect-paths">自动检测路径</button>';
  html += '<button class="btn btn-ghost" data-action="clear-caches">清除缓存</button>';
  html += '<button class="btn btn-ghost" data-action="reset-config">重置配置</button>';
  html += '</div>';

  // ── Appearance ──
  html += '<div class="section-title">界面偏好</div>';
  html += '<div class="form-row"><div class="form-field">';
  html += '<label class="form-label">主题模式</label>';
  html += '<div class="setting-toggle-group">';
  html += '<button class="btn ' + (isDark ? 'btn-ghost' : 'btn-primary') + '" data-action="set-theme-light">浅色</button>';
  html += '<button class="btn ' + (isDark ? 'btn-primary' : 'btn-ghost') + '" data-action="set-theme-dark">深色</button>';
  html += '</div></div>';

  html += '</div></div>';

  // ── Shortcuts ──
  html += '<div class="section-title">键盘快捷键</div>';
  html += '<p class="form-hint">Ctrl+S 保存 · Ctrl+F 搜索 · Ctrl+Z 撤销 · ESC 关闭 · 1-6 切换面板 · ↑↓ 调整数值</p>';
  html += '<button class="btn btn-ghost" style="margin-top:4px;margin-bottom:16px" onclick="state.panel=\'shortcuts\';renderShortcuts()">查看全部快捷键 →</button>';

  // ── About ──
  html += '<div class="section-title">关于</div>';
  html += '<div class="about-info">';
  html += '<div class="about-row"><span class="about-label">应用</span><span class="about-value">Yoshunko Admin</span></div>';
  html += '<div class="about-row"><span class="about-label">版本</span><span class="about-value" id="set-version">---</span></div>';
  html += '<div class="about-row"><span class="about-label">Python</span><span class="about-value">' + (cfg.python_version || '3.13') + '</span></div>';
  html += '<div class="about-row"><span class="about-label">平台</span><span class="about-value">Windows</span></div>';
  html += '<div class="about-row"><span class="about-label">数据状态</span><span class="about-value">' + (cfg.configured ? '已配置' : '未配置') + '</span></div>';
  html += '</div>';

  html += '</div></div>';
  $('#main').innerHTML = html;

  try { var v = await API.getVersion(); var el = document.getElementById('set-version'); if (el) el.textContent = v.version; } catch(e) {}
}
