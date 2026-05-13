<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'

interface LaunchItem {
  key: string
  name: string
  description: string
  isAuto: boolean
}

const launchItems: LaunchItem[] = [
  { key: 'yoshunko', name: 'Yoshunko Server', description: '自动检测 WSL 并启动游戏服务端', isAuto: true },
  { key: 'hoyosdk', name: 'HoyoSDK', description: '米哈游 SDK 服务', isAuto: false },
  { key: 'kcpsim', name: 'KCPSHIM', description: 'KCP 代理服务', isAuto: false },
  { key: 'yidhari', name: 'Yidhari Client', description: '游戏客户端（管理员模式）', isAuto: false },
]

const launchConfig = ref<Record<string, string>>({})
const editingKey = ref<string | null>(null)
const editPath = ref('')

const configuredCount = computed(() => {
  return launchItems.filter(item => {
    if (item.isAuto) return true
    return !!launchConfig.value[item.key]
  }).length
})

const allConfigured = computed(() => {
  return launchItems.filter(i => !i.isAuto).every(i => !!launchConfig.value[i.key])
})

onMounted(async () => {
  try {
    const cfg = await api.getConfig()
    launchConfig.value = cfg.launch_config || {}
  } catch {}
})

function startEdit(key: string) {
  editingKey.value = key
  editPath.value = launchConfig.value[key] || ''
}

function cancelEdit() {
  editingKey.value = null
  editPath.value = ''
}

async function savePath(key: string) {
  // Note: set_launch_path not yet implemented in backend
  launchConfig.value[key] = editPath.value
  editingKey.value = null
  toast('路径已保存（本地）', 'info')
}

async function pastePath() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) editPath.value = text.trim()
  } catch {
    toast('无法读取剪贴板', 'error')
  }
}

async function launch(key: string) {
  // Note: launch commands not yet implemented in backend
  toast('启动功能待后端命令实现', 'info')
}

async function launchAll() {
  toast('一键启动功能待后端命令实现', 'info')
}
</script>

<template>
  <div>
    <div class="page-header">
      <h2>快速启动</h2>
      <span class="subtitle text-muted">管理游戏服务端和客户端的启动路径</span>
    </div>

    <div class="launch-status-bar">
      已配置: {{ configuredCount }}/{{ launchItems.length }}
    </div>

    <div class="launch-grid">
      <div v-for="item in launchItems" :key="item.key" class="launch-card">
        <div class="launch-card__dot" :class="{ active: item.isAuto || !!launchConfig[item.key] }"></div>
        <div class="launch-card__info">
          <div class="launch-card__name">
            {{ item.name }}
            <span v-if="item.isAuto" class="badge badge--auto">自动</span>
            <span v-else-if="launchConfig[item.key]" class="badge badge--configured">已配置</span>
          </div>
          <div class="launch-card__desc">{{ item.description }}</div>
          <div v-if="launchConfig[item.key] && editingKey !== item.key" class="launch-card__path text-sm text-muted">
            {{ launchConfig[item.key] }}
          </div>
        </div>

        <!-- Edit mode -->
        <div v-if="editingKey === item.key" class="launch-card__edit">
          <div class="launch-input-row">
            <input class="form-input" v-model="editPath" placeholder="输入路径..." />
            <button class="btn btn-ghost" @click="pastePath">粘贴</button>
          </div>
          <div class="btn-group">
            <button class="btn btn-primary" @click="savePath(item.key)">保存</button>
            <button class="btn btn-ghost" @click="cancelEdit">取消</button>
          </div>
        </div>

        <!-- Actions -->
        <div v-else class="launch-card__actions">
          <button class="btn btn-primary" @click="launch(item.key)">启动</button>
          <button v-if="!item.isAuto" class="btn btn-ghost" @click="startEdit(item.key)">
            {{ launchConfig[item.key] ? '编辑' : '配置' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Launch All FAB -->
    <Transition name="fab">
      <button
        v-if="allConfigured"
        class="btn btn-success launch-all-fab"
        @click="launchAll"
      >
        一键启动
      </button>
    </Transition>
  </div>
</template>
