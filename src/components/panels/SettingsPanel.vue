<script setup lang="ts">
import { ref, onMounted, onActivated, watch, nextTick } from 'vue'
import { panel, avatarCache, weaponCache, equipCache, cacheDirty } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast, showConfirm } from '@/lib/utils'
import { currentTheme, setTheme } from '@/composables/useTheme'
import type { Config } from '@/lib/types'

// ─── Logs ────────────────────────────────────────────
const logTabs = [
  { key: 'hoyosdk', name: 'HoyoSDK' },
  { key: 'yoshunko', name: 'Yoshunko' },
  { key: 'kcpshim', name: 'KCPSHIM' },
  { key: 'client', name: 'Client' },
]
const activeLogTab = ref('hoyosdk')
const logContent = ref('')
const logOffset = ref(0)
const logLoading = ref(false)

async function loadLog(reset = false) {
  if (reset) {
    logContent.value = ''
    logOffset.value = 0
  }
  logLoading.value = true
  try {
    const r = await api.readLog(activeLogTab.value, logOffset.value)
    if (r.content) {
      logContent.value += r.content
      logOffset.value = r.offset
    }
  } catch {
    toast('读取日志失败', 'error')
  }
  logLoading.value = false
}

async function openLogDir() {
  try {
    await api.openLogDir()
  } catch {
    toast('无法打开日志目录', 'error')
  }
}

watch(activeLogTab, () => loadLog(true))

onActivated(() => {
  nextTick(() => loadLog(true))
})

const config = ref<Config | null>(null)
const version = ref('')

onMounted(async () => {
  try { config.value = await api.getConfig() } catch {}
  try { const v = await api.getVersion(); version.value = v.version } catch {}
})

function autoDetect() {
  // Handled by SetupPanel
  toast('请使用初始设置页面更改路径', 'info')
}

function clearCaches() {
  avatarCache.value = []
  weaponCache.value = []
  equipCache.value = []
  cacheDirty.value = true
  toast('缓存已清除')
}

function resetConfig() {
  showConfirm('重置配置将返回初始化页面，继续？', () => {
    toast('请重启应用以重新配置', 'info')
  })
}

function goToShortcuts() {
  panel.value = 'shortcuts'
}
</script>

<template>
  <div>
    <div class="page-header">
      <h2>系统设置</h2>
      <span class="subtitle text-muted">应用配置与偏好</span>
    </div>

    <div class="settings-panel">
      <div class="panel-box__body">
        <!-- Data Management -->
        <div class="section-title">数据管理</div>
        <div class="form-field">
          <label class="form-label">State 目录</label>
          <div class="input-group">
            <input class="form-input form-input--readonly" type="text" :value="config?.state_dir || '未配置'" readonly />
          </div>
          <p class="form-hint">游戏存档数据所在目录</p>
        </div>
        <div class="btn-group" style="margin-top:8px;margin-bottom:16px">
          <button class="btn btn-ghost" @click="autoDetect">自动检测路径</button>
          <button class="btn btn-ghost" @click="clearCaches">清除缓存</button>
          <button class="btn btn-ghost" @click="resetConfig">重置配置</button>
        </div>

        <!-- Appearance -->
        <div class="section-title">界面偏好</div>
        <div class="form-row">
          <div class="form-field">
            <label class="form-label">主题模式</label>
            <div class="setting-toggle-group">
              <button class="btn" :class="currentTheme === 'light' ? 'btn-primary' : 'btn-ghost'" @click="setTheme('light')">浅色</button>
              <button class="btn" :class="currentTheme === 'dark' ? 'btn-primary' : 'btn-ghost'" @click="setTheme('dark')">深色</button>
            </div>
          </div>
        </div>

        <!-- Logs -->
        <div class="section-title">运行日志</div>
        <div class="log-tabs">
          <button v-for="tab in logTabs" :key="tab.key" class="btn btn-sm"
            :class="activeLogTab === tab.key ? 'btn-primary' : 'btn-ghost'"
            @click="activeLogTab = tab.key">{{ tab.name }}</button>
        </div>
        <pre class="log-viewer">{{ logContent || '（暂无日志）' }}</pre>
        <div class="btn-group" style="margin-top:6px;margin-bottom:16px">
          <button class="btn btn-ghost btn-sm" @click="loadLog(true)" :disabled="logLoading">刷新</button>
          <button class="btn btn-ghost btn-sm" @click="openLogDir">打开日志文件夹</button>
        </div>

        <!-- Shortcuts -->
        <div class="section-title">键盘快捷键</div>
        <p class="form-hint">Ctrl+S 保存 · Ctrl+F 搜索 · Ctrl+Z 撤销 · ESC 关闭 · 1-6 切换面板 · ↑↓ 调整数值</p>
        <button class="btn btn-ghost" style="margin-top:4px;margin-bottom:16px" @click="goToShortcuts">查看全部快捷键 →</button>


        <!-- About -->
        <div class="section-title">关于</div>
        <div class="about-info">
          <div class="about-row"><span class="about-label">应用</span><span class="about-value">Yoshunko Admin</span></div>
          <div class="about-row"><span class="about-label">版本</span><span class="about-value">{{ version }}</span></div>
          <div class="about-row"><span class="about-label">平台</span><span class="about-value">Windows (Tauri v2)</span></div>
          <div class="about-row"><span class="about-label">数据状态</span><span class="about-value">{{ config?.configured ? '已配置' : '未配置' }}</span></div>
        </div>
      </div>
    </div>
  </div>
</template>
