<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue'
import { uid, markCacheDirty } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast, showConfirm } from '@/lib/utils'
import type { PlayerBasic } from '@/lib/types'

const info = ref<PlayerBasic | null>(null)
const loading = ref(true)

// Edit fields
const editNickname = ref('')
const editLevel = ref(60)
const editExp = ref(0)
const editAvatarId = ref(0)
const editControlId = ref(0)
const editGuiseId = ref(0)
const saving = ref(false)
const saved = ref(false)

async function loadData() {
  if (!uid.value) return
  loading.value = true
  try {
    const d = await api.getPlayerBasic(uid.value)
    if (d) {
      info.value = d
      editNickname.value = d.nickname
      editLevel.value = d.level
      editExp.value = d.exp
      editAvatarId.value = d.avatar_id
      editControlId.value = d.control_avatar_id
      editGuiseId.value = d.control_guise_avatar_id
    }
  } catch (e: unknown) {
    toast('加载失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
  loading.value = false
}

onMounted(loadData)
onActivated(loadData)
watch(uid, loadData)

async function save() {
  if (!uid.value) return
  if (editLevel.value < 1 || editLevel.value > 60) { toast('等级需在 1-60 之间', 'error'); return }
  if (!editNickname.value.trim()) { toast('昵称不能为空', 'error'); return }
  saving.value = true
  try {
    const r = await api.updatePlayerBasic(uid.value, {
      nickname: editNickname.value,
      level: editLevel.value,
      exp: editExp.value,
      avatar_id: editAvatarId.value,
      control_avatar_id: editControlId.value,
      control_guise_avatar_id: editGuiseId.value,
    })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('玩家信息已保存', 'success')
    saved.value = true
    setTimeout(() => { saved.value = false }, 1500)
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
  saving.value = false
}

async function exportData() {
  if (!uid.value) return
  try {
    const [basic, avatars, weapons, equips, hadal] = await Promise.all([
      api.getPlayerBasic(uid.value),
      api.getAvatars(uid.value),
      api.getWeapons(uid.value),
      api.getEquips(uid.value),
      api.getHadalZone(uid.value),
    ])
    const exportObj = {
      uid: uid.value,
      format_version: 1,
      exported_at: new Date().toISOString(),
      info: basic,
      avatars: avatars.avatars,
      weapons: weapons.weapons,
      equips: equips.equips,
      hadal_zone: hadal,
    }
    const blob = new Blob([JSON.stringify(exportObj, null, 2)], { type: 'application/json' })
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = `yoshunko_player_${uid.value}_${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    setTimeout(() => URL.revokeObjectURL(a.href), 1000)
    toast('数据已导出', 'success')
  } catch (e: unknown) {
    toast('导出失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
}

function importData() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file || !uid.value) return
    try {
      const text = await file.text()
      const json = JSON.parse(text)
      if (!json.uid) { toast('无效的导出文件', 'error'); return }
      if (json.info && (typeof json.info.nickname !== 'string' || typeof json.info.level !== 'number')) {
        toast('导入数据格式无效', 'error'); return
      }
      showConfirm(`导入将覆盖当前玩家 ${uid.value} 的数据，继续？`, async () => {
        try {
          if (json.info && uid.value) await api.updatePlayerBasic(uid.value, json.info)
          toast('导入完成', 'success')
          markCacheDirty()
        } catch (e: unknown) {
          toast('导入失败: ' + (e instanceof Error ? e.message : ''), 'error')
        }
      })
    } catch (e: unknown) {
      toast('导入失败: ' + (e instanceof Error ? e.message : ''), 'error')
    }
  }
  input.click()
}
</script>

<template>
  <div>
    <div class="page-header">
      <h2>玩家信息</h2>
      <span class="subtitle text-muted">修改基础账号信息</span>
    </div>

    <div v-if="loading" class="loading-wrap"><div class="spinner"></div></div>

    <div v-else-if="info" class="settings-panel">
      <div class="panel-box__body">
        <div class="section-title">基本信息</div>
        <div class="form-row">
          <div class="form-field">
            <label class="form-label">昵称</label>
            <input class="form-input" type="text" v-model="editNickname" />
          </div>
          <div class="form-field">
            <label class="form-label">等级</label>
            <input class="form-input" type="number" v-model.number="editLevel" min="1" max="60" />
          </div>
        </div>
        <div class="form-field">
          <label class="form-label">经验</label>
          <input class="form-input" type="number" v-model.number="editExp" />
        </div>

        <div class="section-title">角色展示</div>
        <div class="form-row-3">
          <div class="form-field">
            <label class="form-label">展示角色 ID</label>
            <input class="form-input" type="number" v-model.number="editAvatarId" />
          </div>
          <div class="form-field">
            <label class="form-label">操控角色 ID</label>
            <input class="form-input" type="number" v-model.number="editControlId" />
          </div>
          <div class="form-field">
            <label class="form-label">伪装角色 ID</label>
            <input class="form-input" type="number" v-model.number="editGuiseId" />
          </div>
        </div>

        <div class="section-title">数据管理</div>
        <div class="btn-group">
          <button class="btn btn-primary" :class="{ 'btn--saving': saving, 'btn--saved': saved }" :disabled="saving" @click="save">{{ saved ? '✓ 已保存' : saving ? '保存中...' : '保存更改' }}</button>
          <button class="btn btn-ghost" @click="exportData">导出数据</button>
          <button class="btn btn-ghost" @click="importData">导入数据</button>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>未找到玩家信息</p>
    </div>
  </div>
</template>
