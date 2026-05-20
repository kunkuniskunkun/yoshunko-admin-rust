<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { ref, onMounted } from 'vue'
import { updateAvailable, updateInfo, installUpdate, getLastError } from '@/composables/useUpdater'

const appWindow = getCurrentWindow()
const version = ref('---')
const showModal = ref(false)
const installing = ref(false)
const progress = ref(0)

function minimize() { appWindow.minimize() }
function toggleMax() { appWindow.toggleMaximize() }
function close() { appWindow.close() }

async function doInstall() {
  installing.value = true
  const ok = await installUpdate((pct) => { progress.value = pct })
  installing.value = false
  if (!ok) {
    showModal.value = false
    toast(`更新下载失败: ${getLastError() || '未知错误'}`, 'error')
  }
}

onMounted(async () => {
  try { const data = await api.getVersion(); version.value = data.version } catch { version.value = '---' }
})
</script>

<template>
  <div data-tauri-drag-region class="title-bar">
    <div class="title-bar__left" data-tauri-drag-region>
      <span class="title-bar__brand">Yoshunko Admin</span>
      <span class="title-bar__version">{{ version }}</span>
      <span
        v-if="updateAvailable"
        class="title-bar__update-badge"
        @click.stop="showModal = true"
      >
        新版本 v{{ updateInfo?.version }}
      </span>
    </div>
    <div class="title-bar__controls">
      <button class="tb-btn" aria-label="最小化" @click="minimize">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="2" fill="currentColor"/></svg>
      </button>
      <button class="tb-btn" aria-label="最大化" @click="toggleMax">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
      </button>
      <button class="tb-btn tb-btn--close" aria-label="关闭" @click="close">
        <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>
    </div>
  </div>

  <n-modal v-model:show="showModal" title="发现新版本" :mask-closable="false">
    <div class="update-modal">
      <p>新版本: <strong>v{{ updateInfo?.version }}</strong></p>
      <pre class="update-notes">{{ updateInfo?.body || '无更新说明' }}</pre>
      <p v-if="installing">下载进度: {{ progress }}%</p>
      <div class="update-modal__actions">
        <n-button @click="showModal = false">稍后提醒</n-button>
        <n-button type="primary" @click="doInstall" :loading="installing">
          {{ installing ? '下载安装中...' : '立即更新' }}
        </n-button>
      </div>
    </div>
  </n-modal>
</template>
