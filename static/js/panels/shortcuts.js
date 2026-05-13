// ═══════════════════════════════════════════════════════
// Keyboard Shortcuts Reference
// ═══════════════════════════════════════════════════════

function renderShortcuts() {
  var html = '<div class="page-header"><h2>键盘快捷键</h2><span class="subtitle">所有可用快捷键一览</span></div>';
  html += '<div class="settings-panel">';

  var sections = [
    {
      title: '面板导航',
      items: [
        ['1', '角色管理'], ['2', '音擎仓库'], ['3', '驱动盘仓库'],
        ['4', '式舆防卫战'], ['5', '玩家信息'], ['6', '系统设置'], ['7', '快速启动']
      ]
    },
    {
      title: '编辑操作',
      items: [
        ['Ctrl+S', '保存当前编辑'],
        ['Ctrl+Z', '撤销上一步'],
        ['↑ ↓', '调整数字输入框数值'],
        ['Enter', '激活聚焦的卡片'],
        ['Space', '激活聚焦的卡片']
      ]
    },
    {
      title: '导航与搜索',
      items: [
        ['Ctrl+F', '聚焦当前面板搜索框'],
        ['ESC', '关闭弹窗/确认框'],
        ['ESC', '从编辑器返回画廊'],
        ['Tab', '在弹窗内循环切换焦点'],
        ['Shift+Tab', '在弹窗内反向切换焦点']
      ]
    }
  ];

  sections.forEach(function(sec) {
    html += '<div class="panel-box"><div class="panel-box__header">' + sec.title + '</div><div class="panel-box__body">';
    html += '<div class="about-info">';
    sec.items.forEach(function(item) {
      html += '<div class="about-row"><span class="about-label"><kbd>' + item[0] + '</kbd></span><span class="about-value">' + item[1] + '</span></div>';
    });
    html += '</div></div></div>';
  });

  html += '</div>';
  $('#main').innerHTML = html;
}
