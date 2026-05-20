<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue'
import { uid } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { open } from '@tauri-apps/plugin-shell'
import { ChevronDown } from 'lucide-vue-next'
import type { HadalEntrance, HadalZone } from '@/lib/types'

const data = ref<HadalZone | null>(null)
const loading = ref(true)
const entranceEdits = ref<{ id: number; zone_id: number }[]>([])
const showHelp = ref(false)
const saving = ref(false)

const ACTIVE_ENTRANCE_IDS = [1, 3]

const ENTRANCE_NAMES: Record<number, string> = {
  1: '危局强袭战',
  3: '式舆防卫战·剧变',
}

function isPermanent(id: number): boolean {
  return id === 3
}

async function openUrl(url: string) {
  await open(url)
}

async function loadData() {
  if (!uid.value) return
  loading.value = true
  try {
    const hz = await api.getHadalZone(uid.value)
    if (hz) {
      data.value = hz
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
  saving.value = true
  try {
    const r = await api.updateHadalZone(uid.value, { entrances: entranceEdits.value })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('式舆防卫战配置已保存', 'success')
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
  saving.value = false
}

function resetDefaults() {
  if (!data.value) return
  entranceEdits.value = data.value.entrances
    .filter(e => ACTIVE_ENTRANCE_IDS.includes(e.id))
    .map(e => ({ ...e }))
  toast('已重置为原始值', 'info')
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
            <div
              v-for="(e, i) in entranceEdits"
              :key="e.id"
              class="entrance-card"
              :class="isPermanent(e.id) ? 'entrance-card--permanent' : 'entrance-card--limited'"
            >
              <div class="entrance-card__info" style="margin-left: 0;">
                <div class="entrance-card__name">{{ ENTRANCE_NAMES[e.id] || '入口 ' + e.id }}</div>
              </div>
              <div class="form-field">
                <label class="form-label">Zone ID</label>
                <input class="form-input" type="number" v-model.number="entranceEdits[i].zone_id" />
              </div>
            </div>
          </div>

          <div class="hadal-links">
            <span class="hadal-links-label">最新 ZONE ID 查看链接</span>
            <a class="hadal-link" @click.prevent="openUrl('https://zzz.nanoka.cc/boss')">CH: https://zzz.nanoka.cc/boss</a>
            <a class="hadal-link" @click.prevent="openUrl('https://www.buhflipexplode.org/home/')">En: https://www.buhflipexplode.org/home/</a>
          </div>

          <div class="btn-group" style="margin-top: 16px;">
            <button class="btn btn-primary" :class="{ 'btn--saving': saving }" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存更改' }}</button>
            <button class="btn btn-ghost" @click="resetDefaults">重置</button>
          </div>

          <!-- 使用说明 -->
          <div class="hadal-help">
            <button class="hadal-help__toggle" :class="{ 'hadal-help__toggle--open': showHelp }" @click="showHelp = !showHelp">
              <ChevronDown :size="16" />
              <span>使用说明</span>
            </button>
            <div v-if="showHelp" class="hadal-help__body">
              <p><strong>Zone ID</strong> 是防卫战每期的唯一标识。修改 Zone ID 可以切换到不同的防卫战期号。</p>
              <p><strong>危局强袭战</strong>（限时）：定期刷新，需要手动更新 Zone ID 以匹配最新期号。</p>
              <p><strong>式舆防卫战·剧变</strong>（常驻）：Zone ID 通常保持不变。</p>
              <p>点击上方链接可查看最新的 Zone ID。修改后点击<strong>保存更改</strong>，服务器会热加载即时生效。</p>
            </div>
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
