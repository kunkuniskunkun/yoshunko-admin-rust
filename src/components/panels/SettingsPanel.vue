<script setup lang="ts">
import { ref, watch, onMounted, onActivated, onDeactivated } from 'vue'
import { panel, avatarCache, weaponCache, equipCache, markAllCacheDirty } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast, showConfirm } from '@/lib/utils'
import { currentTheme, setTheme, currentAccent, setAccent, ACCENT_COLORS } from '@/composables/useTheme'
import { bgUrl, bgOpacity, bgPath, setBackground } from '@/composables/useBackground'
import { checkUpdate, updateInfo, openReleasePage } from '@/composables/useUpdater'
import type { Config } from '@/lib/types'

// ─── Logs ────────────────────────────────────────────
const logTabs = [
  { key: 'hoyosdk', name: 'HoyoSDK' },
  { key: 'yoshunko', name: 'Yoshunko' },
  { key: 'kcpshim', name: 'KCPSHIM' },
]
const showLogViewer = ref(false)
const activeLogTab = ref('hoyosdk')
const selectedLogFile = ref('')
const logContent = ref('')
const logOffset = ref(0)
const logLoading = ref(false)
let pollTimer: ReturnType<typeof setInterval> | null = null

async function loadLatestLog() {
  try {
    const r = await api.listLogs(activeLogTab.value)
    const files = r.logs || []
    if (files.length) {
      const latest = files[files.length - 1].filename
      if (latest !== selectedLogFile.value) {
        selectedLogFile.value = latest
        logContent.value = ''
        logOffset.value = 0
      }
      await loadLogContent()
    } else {
      selectedLogFile.value = ''
      logContent.value = ''
    }
  } catch {}
}

async function loadLogContent() {
  if (!selectedLogFile.value) { logContent.value = ''; return }
  logLoading.value = true
  try {
    const r = await api.readLog(selectedLogFile.value, logOffset.value)
    if (r.content) {
      logContent.value += r.content
      logOffset.value = r.offset
    }
  } catch {
    toast('读取日志失败', 'error')
  }
  logLoading.value = false
}

function openLogViewer() {
  showLogViewer.value = true
  loadLatestLog()
  startPolling()
}

function closeLogViewer() {
  showLogViewer.value = false
  stopPolling()
}

function startPolling() {
  stopPolling()
  pollTimer = setInterval(() => { loadLogContent() }, 3000)
}

function stopPolling() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

async function switchLogTab(key: string) {
  activeLogTab.value = key
  selectedLogFile.value = ''
  logContent.value = ''
  logOffset.value = 0
  await loadLatestLog()
}

async function openLogDir() {
  try { await api.openLogDir() } catch { toast('无法打开日志目录', 'error') }
}

onDeactivated(() => { stopPolling(); showLogViewer.value = false })

watch(panel, (val) => { if (val !== 'settings') { stopPolling(); showLogViewer.value = false } })

// ─── Config ──────────────────────────────────────────
const config = ref<Config | null>(null)
const version = ref('')
const checking = ref(false)
async function handleCheckUpdate() {
  checking.value = true
  const found = await checkUpdate()
  checking.value = false
  if (found) {
    // Use NMessage to show toast (auto-imported)
    toast(`发现新版本 v${updateInfo.value?.version}`, 'success')
  } else {
    toast('已是最新版本', 'info')
  }
}

onMounted(async () => {
  try { config.value = await api.getConfig() } catch {}
  try { const v = await api.getVersion(); version.value = v.version } catch {}
})

function autoDetect() {
  toast('请使用初始设置页面更改路径', 'info')
}

function clearCaches() {
  avatarCache.value = []
  weaponCache.value = []
  equipCache.value = []
  markAllCacheDirty()
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

// ─── Background image ──────────────────────────────────
async function selectBackground() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    multiple: false,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (!selected) return
  const path = selected as string
  console.log('[selectBackground] selected:', path)
  try {
    const r = await api.setBackground(path, bgOpacity.value)
    if (!r.ok) { toast('保存失败: ' + r.error, 'error'); return }
    await setBackground(path, bgOpacity.value)
    toast('背景图已设置', 'success')
  } catch (e: unknown) {
    toast('设置失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
}

async function clearBackground() {
  try {
    const r = await api.setBackground('', 0)
    if (!r.ok) { toast('清除失败: ' + r.error, 'error'); return }
    await setBackground('', 0.85)
    toast('背景图已清除', 'success')
  } catch (e: unknown) {
    toast('清除失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
}

async function setBgOpacity(val: number) {
  const opacity = Math.round(val * 100) / 100
  bgOpacity.value = opacity
  if (bgPath.value) {
    try { await api.setBackground(bgPath.value, opacity) } catch {}
  }
}
</script>

<template>
  <div>
    <!-- Log Viewer (overlay) -->
    <div v-if="showLogViewer" class="log-viewer-overlay">
      <div class="log-viewer-panel">
        <div class="log-viewer-header">
          <h3>运行日志</h3>
          <button class="btn btn-ghost btn-sm" @click="closeLogViewer">✕ 关闭</button>
        </div>
        <div class="log-tabs">
          <button v-for="tab in logTabs" :key="tab.key" class="btn btn-sm"
            :class="activeLogTab === tab.key ? 'btn-primary' : 'btn-ghost'"
            style="transition: none;"
            @click="switchLogTab(tab.key)">{{ tab.name }}</button>
        </div>
        <pre class="log-content">{{ logContent || '（暂无日志）' }}</pre>
        <div class="log-viewer-footer">
          <button class="btn btn-ghost btn-sm" @click="openLogDir">打开日志文件夹</button>
          <span class="text-muted text-xs" style="margin-left:auto">自动刷新 · {{ selectedLogFile || '暂无日志文件' }}</span>
        </div>
      </div>
    </div>

    <!-- Settings Main -->
    <div v-if="!showLogViewer">
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
          <div class="form-field" style="margin-top: 12px;">
            <label class="form-label">主题色</label>
            <div class="accent-picker">
              <button
                v-for="color in ACCENT_COLORS"
                :key="color.key"
                class="accent-dot"
                :class="{ active: currentAccent === color.key }"
                :style="{ '--dot-color': color.hex }"
                :title="color.label"
                @click="setAccent(color.key)"
              />
            </div>
          </div>
          <div class="form-field" style="margin-top: 16px;">
            <label class="form-label">背景图</label>
            <div class="btn-group" style="margin-bottom: 8px;">
              <button class="btn btn-ghost" @click="selectBackground">选择图片</button>
              <button class="btn btn-ghost" @click="clearBackground">清除背景</button>
            </div>
            <div v-if="bgPath" class="form-field" style="margin-top: 8px;">
              <label class="form-label">遮罩透明度</label>
              <div style="display: flex; align-items: center; gap: 12px;">
                <input
                  type="range"
                  min="0.3"
                  max="0.95"
                  step="0.05"
                  :value="bgOpacity"
                  style="flex: 1;"
                  @input="setBgOpacity(parseFloat(($event.target as HTMLInputElement).value))"
                />
                <span class="text-muted text-xs" style="min-width: 36px;">{{ Math.round(bgOpacity * 100) }}%</span>
              </div>
            </div>
          </div>

          <!-- Logs -->
          <div class="section-title">运行日志</div>
          <div class="btn-group" style="margin-bottom:16px">
            <button class="btn btn-ghost" @click="openLogViewer">查看运行日志</button>
            <button class="btn btn-ghost" @click="openLogDir">打开日志文件夹</button>
          </div>

          <!-- Shortcuts -->
          <div class="section-title">键盘快捷键</div>
          <p class="form-hint">Ctrl+S 保存 · Ctrl+F 搜索 · Ctrl+Z 撤回操作 · ESC 关闭 · 1-7 切换面板 · ↑↓ 调整数值</p>
          <button class="btn btn-ghost" style="margin-top:4px;margin-bottom:16px" @click="goToShortcuts">查看全部快捷键 →</button>

          <!-- 更新 -->
          <div class="section-title">更新</div>
          <div class="update-info-box">
            <p class="form-hint">启动时自动检查新版本，发现更新后标题栏会出现通知</p>
            <div class="btn-group">
              <n-button size="small" @click="handleCheckUpdate" :loading="checking">
                {{ checking ? '检查中...' : '手动检查更新' }}
              </n-button>
              <n-button size="small" text @click="openReleasePage">手动下载</n-button>
            </div>
          </div>

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
  </div>
</template>
