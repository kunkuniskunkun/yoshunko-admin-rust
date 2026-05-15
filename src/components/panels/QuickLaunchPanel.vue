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
  { key: 'kcpshim', name: 'KCPSHIM', description: 'KCP 代理服务', isAuto: false },
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
    const r = await api.getLaunchConfig()
    launchConfig.value = r.config || {}
  } catch {
    try {
      const cfg = await api.getConfig()
      launchConfig.value = cfg.launch_config || {}
    } catch {}
  }
})

function startEdit(key: string) {
  editingKey.value = key
  editPath.value = (launchConfig.value[key] || '').replace(/^["']|["']$/g, '')
}

function cancelEdit() {
  editingKey.value = null
  editPath.value = ''
}

async function savePath(key: string) {
  const cleaned = editPath.value.trim().replace(/^["']|["']$/g, '')
  if (!cleaned) { toast('路径不能为空', 'error'); return }
  try {
    const r = await api.setLaunchPath(key, cleaned)
    if (r.ok) {
      launchConfig.value[key] = cleaned
      toast('路径已保存', 'success')
    } else {
      toast(r.error || '保存失败', 'error')
    }
  } catch (e: unknown) {
    launchConfig.value[key] = cleaned
    toast('路径已保存（本地）', 'info')
  }
  editingKey.value = null
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
  try {
    if (key === 'yoshunko') {
      const r = await api.launchYoshunko()
      if (r.ok) toast('Yoshunko 服务端已启动', 'success')
      else toast(r.error || '启动失败', 'error')
    } else if (key === 'yidhari') {
      const path = launchConfig.value[key]
      if (!path) { toast('请先配置路径', 'error'); return }
      const r = await api.launchProgramAdmin(path)
      if (r.ok) toast('游戏客户端已启动', 'success')
      else toast(r.error || '启动失败', 'error')
    } else {
      const r = await api.launchProgram(key)
      if (r.ok) toast('已启动', 'success')
      else toast(r.error || '启动失败', 'error')
    }
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '启动失败', 'error')
  }
}

async function launchAll() {
  for (const item of launchItems) {
    if (item.isAuto || launchConfig.value[item.key]) {
      await launch(item.key)
    }
  }
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
      <div v-for="item in launchItems" :key="item.key" class="launch-card" :class="{ 'launch-card--ready': item.isAuto || !!launchConfig[item.key] }">
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
            配置
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
