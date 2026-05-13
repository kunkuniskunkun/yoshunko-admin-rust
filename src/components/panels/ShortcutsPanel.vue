<script setup lang="ts">
const sections = [
  {
    title: '面板导航',
    items: [
      ['1', '角色管理'], ['2', '音擎仓库'], ['3', '驱动盘仓库'],
      ['4', '式舆防卫战'], ['5', '玩家信息'], ['6', '系统设置'], ['7', '快速启动'],
    ],
  },
  {
    title: '编辑操作',
    items: [
      ['Ctrl+S', '保存当前编辑'],
      ['Ctrl+Z', '撤销上一步'],
      ['↑ ↓', '调整数字输入框数值'],
      ['Enter', '激活聚焦的卡片'],
      ['Space', '激活聚焦的卡片'],
    ],
  },
  {
    title: '导航与搜索',
    items: [
      ['Ctrl+F', '聚焦当前面板搜索框'],
      ['ESC', '关闭弹窗/确认框'],
      ['ESC', '从编辑器返回画廊'],
      ['Tab', '在弹窗内循环切换焦点'],
      ['Shift+Tab', '在弹窗内反向切换焦点'],
    ],
  },
]
</script>

<template>
  <div>
    <div class="page-header">
      <h2>键盘快捷键</h2>
      <span class="subtitle text-muted">所有可用快捷键一览</span>
    </div>
    <div class="settings-panel">
      <div v-for="sec in sections" :key="sec.title" class="panel-box">
        <div class="panel-box__header">{{ sec.title }}</div>
        <div class="panel-box__body">
          <div class="about-info">
            <div v-for="item in sec.items" :key="item[0] + item[1]" class="about-row">
              <span class="about-label"><kbd>{{ item[0] }}</kbd></span>
              <span class="about-value">{{ item[1] }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
