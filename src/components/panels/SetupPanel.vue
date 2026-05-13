<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'

const stateDir = ref('')
const candidates = ref<string[]>([])
const version = ref('')
const loading = ref(false)

const emit = defineEmits<{
  (e: 'connected'): void
}>()

onMounted(async () => {
  try {
    const v = await api.getVersion()
    version.value = v.version
  } catch {}
  try {
    const r = await api.autoDetectPaths()
    candidates.value = r.candidates || []
    if (candidates.value.length > 0) {
      stateDir.value = candidates.value[0]
    }
  } catch {}
})

function selectCandidate(path: string) {
  stateDir.value = path
}

async function connect() {
  if (!stateDir.value.trim()) {
    toast('请输入状态目录路径', 'error')
    return
  }
  loading.value = true
  try {
    const r = await api.setStateDir(stateDir.value.trim())
    if (r.ok) {
      toast('连接成功', 'success')
      emit('connected')
    } else {
      toast(r.error || '连接失败', 'error')
    }
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '连接失败', 'error')
  } finally {
    loading.value = false
  }
}

async function paste() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) stateDir.value = text.trim()
  } catch {
    toast('无法读取剪贴板', 'error')
  }
}
</script>

<template>
  <div class="setup-page">
    <div class="setup-card">
      <div class="setup-brand">
        <img src="@/assets/icon.png" alt="Logo" class="setup-logo" width="85" height="85" />
        <h1>Yoshunko Admin</h1>
        <p class="text-muted">{{ version }}</p>
      </div>

      <div class="setup-form">
        <label class="form-label">状态目录路径</label>
        <div class="setup-input-row">
          <input
            id="setup-state-dir"
            v-model="stateDir"
            class="form-input"
            placeholder="例如: D:\3.0.1\state"
          />
          <button class="btn btn-ghost" @click="paste">粘贴</button>
        </div>

        <div v-if="candidates.length" id="setup-candidates" class="setup-candidates">
          <p class="text-sm text-muted mb-1 mt-2">检测到的路径（点击填入）：</p>
          <div
            v-for="c in candidates"
            :key="c"
            class="candidate-path"
            @click="selectCandidate(c)"
          >{{ c }}</div>
        </div>

        <button class="btn btn-primary mt-3" :disabled="loading" @click="connect">
          {{ loading ? '连接中...' : '连接' }}
        </button>
      </div>
    </div>
  </div>
</template>
