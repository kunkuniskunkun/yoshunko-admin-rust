<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue'
import { uid } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import type { HadalEntrance } from '@/lib/types'

interface HadalData {
  entrances: HadalEntrance[]
  saved_rooms?: { zone_id: number; layer_index: number; avatar_id_list: number[]; buddy_id: number }[]
}

const data = ref<HadalData | null>(null)
const loading = ref(true)
const entranceEdits = ref<{ id: number; zone_id: number }[]>([])

const ACTIVE_ENTRANCE_IDS = [1, 3]

const ENTRANCE_NAMES: Record<number, string> = {
  1: '危局强袭站',
  3: '式舆防卫战·剧变',
}

const ENTRANCE_ICONS: Record<number, string> = {
  1: '◆', 3: '◆',
}

function isPermanent(id: number): boolean {
  return id === 3
}

async function loadData() {
  if (!uid.value) return
  loading.value = true
  try {
    const hz = await api.getHadalZone(uid.value)
    if (hz) {
      data.value = hz as unknown as HadalData
      entranceEdits.value = hz.entrances.filter(e => ACTIVE_ENTRANCE_IDS.includes(e.id)).map(e => ({ ...e }))
    } else {
      data.value = null
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
  if (!uid.value || !data.value) return
  try {
    const r = await api.updateHadalZone(uid.value, { entrances: entranceEdits.value })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('式舆防卫战配置已保存', 'success')
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
}
</script>

<template>
  <div>
    <div class="page-header">
      <h2>防卫战·危局</h2>
      <span class="subtitle text-muted">修改 Zone ID 以切换期号，服务器热加载即时生效</span>
    </div>

    <div v-if="loading" class="loading-wrap"><div class="spinner"></div></div>

    <template v-else-if="data">
      <div class="settings-panel">
        <div class="panel-box__body">
          <div class="section-title">入口配置</div>
          <div class="entrance-grid">
            <div v-for="(e, i) in entranceEdits" :key="e.id" class="entrance-card">
              <div class="entrance-card__icon">{{ ENTRANCE_ICONS[e.id] || '◆' }}</div>
              <div class="entrance-card__info">
                <div class="entrance-card__name">{{ ENTRANCE_NAMES[e.id] || '入口 ' + e.id }}</div>
                <div class="entrance-card__type">{{ isPermanent(e.id) ? '常驻' : '限时' }} · ID: {{ e.id }}</div>
              </div>
              <div class="form-field">
                <label class="form-label">Zone ID</label>
                <input class="form-input" type="number" v-model.number="entranceEdits[i].zone_id" />
              </div>
            </div>
          </div>

          <div class="btn-group" style="margin-top: 16px;">
            <button class="btn btn-primary" @click="save">保存更改</button>
          </div>

          <template v-if="data.saved_rooms && data.saved_rooms.length > 0">
            <div class="section-title" style="margin-top: 24px;">已保存的房间</div>
            <div class="data-table-wrap">
              <table class="data-table">
                <thead><tr><th>Zone</th><th>Layer</th><th>Avatars</th><th>Buddy</th></tr></thead>
                <tbody>
                  <tr v-for="(room, i) in data.saved_rooms" :key="i">
                    <td>{{ room.zone_id }}</td>
                    <td>{{ room.layer_index }}</td>
                    <td>{{ (room.avatar_id_list || []).join(', ') || '—' }}</td>
                    <td>{{ room.buddy_id || 0 }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </template>
        </div>
      </div>
    </template>

    <div v-else class="empty-state">
      <p>未找到式舆防卫战数据</p>
    </div>
  </div>
</template>
