<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import {
  uid, equipCache, cacheDirty, searchQuery,
  selectedEquipUid, equipView, markCacheDirty, templates,
} from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { SUIT_PINYIN } from '@/assets/pinyin-data'
import type { EquipListItem, EquipDetail, EquipProperty } from '@/lib/types'
import SearchBar from '@/components/shared/SearchBar.vue'
import Stepper from '@/components/shared/Stepper.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'
import { applyStaggeredAnimation, applyEditorSlideIn } from '@/composables/useStaggeredAnimation'

const loading = ref(true)
const editorData = ref<EquipDetail | null>(null)
const editorLoading = ref(false)

// Editor fields
const editLevel = ref(0)
const editStar = ref(5)
const editMainProp = ref<EquipProperty>({ key: 0, key_name: '', base_value: 0, add_value: 0 })
const editSubProps = ref<(EquipProperty | null)[]>([null, null, null, null])

const filteredEquips = computed(() => {
  let list = [...equipCache.value]
  list.sort((a, b) => b.uid - a.uid)
  const q = searchQuery.equips.toLowerCase()
  if (q) {
    list = list.filter(eq => {
      const py = SUIT_PINYIN[String(eq.id)]
      if (py && (py.full.includes(q) || py.initials.includes(q))) return true
      return String(eq.uid).includes(q)
        || eq.suit_name.toLowerCase().includes(q)
        || eq.slot_name.toLowerCase().includes(q)
        || String(eq.id).includes(q)
    })
  }
  return list
})

const groupedEquips = computed(() => {
  const groups = new Map<string, EquipListItem[]>()
  for (const eq of filteredEquips.value) {
    const key = eq.suit_name || 'Unknown'
    if (!groups.has(key)) groups.set(key, [])
    groups.get(key)!.push(eq)
  }
  return groups
})

function suitColorClass(suitName: string): string {
  // Simple hash-based color assignment
  let hash = 0
  for (let i = 0; i < suitName.length; i++) {
    hash = suitName.charCodeAt(i) + ((hash << 5) - hash)
  }
  return 'suit-' + (Math.abs(hash) % 6)
}

function slotNumber(id: number): number {
  return id % 10
}

function getMainStatName(key: number): string {
  if (!templates.value) return '属性' + key
  return templates.value.stat_names[key] || '属性' + key
}

async function selectEquip(euid: number, event?: Event) {
  // Card press animation
  if (event?.currentTarget) {
    const el = event.currentTarget as HTMLElement
    el.style.transition = 'transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1)'
    el.style.transform = 'scale(0.92)'
    setTimeout(() => { el.style.transform = 'scale(1)' }, 120)
  }
  selectedEquipUid.value = euid
  equipView.value = 'editor'
  editorLoading.value = true
  try {
    const eq = await api.getEquip(uid.value!, euid)
    if (selectedEquipUid.value !== euid) return
    if (!eq) { toast('驱动盘数据未找到', 'error'); backToGallery(); return }
    editorData.value = eq
    editLevel.value = eq.level
    editStar.value = eq.star
    editMainProp.value = eq.properties[0] || { key: 0, key_name: '', base_value: 0, add_value: 0 }
    editSubProps.value = [...eq.sub_properties]
    while (editSubProps.value.length < 4) editSubProps.value.push(null)
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '加载失败', 'error')
    backToGallery()
  }
  editorLoading.value = false
  // Editor slide-in
  nextTick(() => {
    const mainEl = document.querySelector('.main-content') as HTMLElement
    if (mainEl) applyEditorSlideIn(mainEl)
  })
}

function backToGallery() {
  equipView.value = 'gallery'
  selectedEquipUid.value = null
  editorData.value = null
  nextTick(() => applyStaggeredAnimation('.equip-card'))
}

function getEnhanceSum(): number {
  const filled = editSubProps.value.filter(p => p !== null).length
  const totalAdd = editSubProps.value.reduce((s, p) => s + (p?.add_value || 0), 0)
  return totalAdd - filled
}

async function saveEquip() {
  if (!editorData.value || !uid.value || !selectedEquipUid.value) return
  try {
    const r = await api.updateEquip(uid.value, selectedEquipUid.value, {
      level: editLevel.value,
      star: editStar.value,
      properties: [editMainProp.value],
      sub_properties: editSubProps.value,
    })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('驱动盘数据已保存', 'success')
    markCacheDirty()
    backToGallery()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
}

async function deleteEquip() {
  if (!uid.value || !selectedEquipUid.value) return
  try {
    const r = await api.deleteEquip(uid.value, selectedEquipUid.value)
    if (r.ok === false) throw new Error(r.error || '删除失败')
    toast('驱动盘已删除', 'success')
    markCacheDirty()
    backToGallery()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '删除失败', 'error')
  }
}

onMounted(async () => {
  if (!uid.value) return
  if (equipCache.value.length && !cacheDirty.value) { loading.value = false; return }
  try {
    const data = await api.getEquips(uid.value)
    equipCache.value = data.equips
    cacheDirty.value = false
  } catch (e: unknown) {
    toast('加载驱动盘失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
  loading.value = false
  if (equipView.value === 'gallery') {
    applyStaggeredAnimation('.equip-card')
  }
})
</script>

<template>
  <!-- Editor -->
  <div v-if="equipView === 'editor' && selectedEquipUid" class="editor-page">
    <div class="editor-page__top">
      <a class="editor-back" href="#" @click.prevent="backToGallery">← 驱动盘仓库</a>
      <div class="editor-page__header" v-if="editorData">
        <h2>{{ editorData.suit_name }} · {{ editorData.slot_name }}</h2>
        <span class="sub text-muted">#{{ selectedEquipUid }}</span>
      </div>
    </div>

    <div v-if="editorLoading" class="loading-wrap"><div class="spinner"></div></div>

    <div v-else-if="editorData" class="editor-page__body">
      <div class="section-title">基础属性</div>
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">等级</label>
          <Stepper v-model="editLevel" :min="0" :max="15" label="等级" />
        </div>
        <div class="form-field">
          <label class="form-label">星级</label>
          <Stepper v-model="editStar" :min="1" :max="5" label="星级" />
        </div>
      </div>

      <!-- Main stat -->
      <div class="section-title">主属性</div>
      <div class="form-field">
        <span class="stat-pill stat-pill--main">
          {{ getMainStatName(editMainProp.key) }} {{ editMainProp.base_value }}
        </span>
      </div>

      <!-- Sub stats -->
      <div class="section-title">副属性</div>
      <div v-for="(prop, i) in editSubProps" :key="i" class="sub-stat-row">
        <span v-if="prop" class="sub-stat-item">
          {{ getMainStatName(prop.key) }}: {{ prop.base_value }} +{{ prop.add_value }}
        </span>
        <span v-else class="text-muted">空</span>
      </div>
      <div class="enhance-sum" :class="{
        'enhance-sum--valid': getEnhanceSum() >= 4 && getEnhanceSum() <= 5,
        'enhance-sum--warn': getEnhanceSum() < 4 || getEnhanceSum() > 5,
      }">
        强化总和: +{{ getEnhanceSum() }}
      </div>
    </div>

    <div class="editor-page__actions editor-fab-group" v-if="editorData">
      <button class="btn btn-danger" @click="deleteEquip">删除</button>
      <button class="btn btn-primary" @click="saveEquip">保存更改</button>
    </div>
  </div>

  <!-- Gallery -->
  <div v-else>
    <div class="page-header flex-between">
      <div>
        <h2>驱动盘仓库</h2>
        <span class="subtitle text-muted">管理驱动盘数据，包括主属性与副属性</span>
      </div>
    </div>

    <SearchBar v-model="searchQuery.equips" placeholder="搜索驱动盘 UID、套装、槽位..." />

    <SkeletonGrid v-if="loading" />

    <div v-else-if="filteredEquips.length === 0" class="empty-state">
      <div class="empty-state__icon"></div><p>没有找到匹配的驱动盘</p>
    </div>

    <div v-else>
      <div v-for="[suit, items] in groupedEquips" :key="suit" class="equip-suit-section">
        <div class="equip-suit-title">
          <span class="suit-chip" :class="suitColorClass(suit)">{{ suit }}</span>
          <span class="text-muted"> · {{ items.length }} 件</span>
        </div>
        <div class="equip-grid">
          <div
            v-for="eq in items"
            :key="eq.uid"
            class="game-card equip-card"
            :class="suitColorClass(eq.suit_name)"
            tabindex="0" role="button"
            @click="selectEquip(eq.uid, $event)"
          >
            <div class="card-header">
              <span>
                <span class="slot-tag">{{ slotNumber(eq.id) }}号位</span>
                <span class="card-title">#{{ eq.uid }}</span>
              </span>
              <span class="game-card__level">Lv.{{ eq.level }}</span>
            </div>
            <div class="card-meta">
              <span class="star-rating">
                <span v-for="i in 5" :key="i" class="star" :class="{ active: i <= eq.star }">★</span>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
