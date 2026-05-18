<script setup lang="ts">
import { ref, computed, onMounted, onActivated, nextTick, watch } from 'vue'
import {
  uid, panel, equipCache, cacheDirty, searchQuery,
  selectedEquipUid, equipView, markCacheDirty, markDirty, markClean, templates, pushUndo,
} from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { SUIT_PINYIN } from '@/assets/pinyin-data'
import type { EquipListItem, EquipDetail, EquipProperty, EquipCreate } from '@/lib/types'
import SearchBar from '@/components/shared/SearchBar.vue'
import Stepper from '@/components/shared/Stepper.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'
import { applyStaggeredAnimation, applyEditorSlideIn } from '@/composables/useStaggeredAnimation'

const loading = ref(true)
const refreshing = ref(false)
const editorData = ref<EquipDetail | null>(null)
const editorLoading = ref(false)
const saving = ref(false)

// Editor fields
const editLevel = ref(0)
const editStar = ref(5)
const editMainProp = ref<EquipProperty>({ key: 0, key_name: '', base_value: 0, add_value: 0 })
const editSubProps = ref<(EquipProperty | null)[]>([null, null, null, null])

// ─── 创建驱动盘状态 ────────────────────────────────
const showCreate = ref(false)
const createStep = ref<1 | 2 | 3>(1)
const createSuitType = ref(0)
const createSuitName = ref('')
const createEquipId = ref(0)
const createSlot = ref(0)
const createSlotName = ref('')
const createLevel = ref(15)
const createStar = ref(5)
const createMainKey = ref(0)
const createMainName = ref('')
const createMainBase = ref(0)
const createSubProps = ref<{ key: number; name: string; base: number; add: number }[]>([
  { key: 0, name: '', base: 0, add: 0 },
  { key: 0, name: '', base: 0, add: 0 },
  { key: 0, name: '', base: 0, add: 0 },
  { key: 0, name: '', base: 0, add: 0 },
])

const suitList = computed(() => {
  if (!templates.value) return []
  return Object.entries(templates.value.suit_groups)
    .sort((a, b) => Number(a[0]) - Number(b[0]))
    .map(([, group]) => group)
})

const currentSuitSlots = computed(() => {
  if (!templates.value || !createSuitType.value) return []
  const group = templates.value.suit_groups[String(createSuitType.value)]
  return group?.slots || []
})

const currentMainOptions = computed(() => {
  if (!templates.value || !createSlot.value) return []
  return templates.value.main_stat_options[String(createSlot.value)] || []
})

const subStatOptions = computed(() => {
  return templates.value?.sub_stat_options || []
})

const isSlotFixed = computed(() => createSlot.value >= 1 && createSlot.value <= 3)

const createEnhanceSum = computed(() => {
  return createSubProps.value.reduce((s, p) => s + (p.key > 0 ? p.add - 1 : 0), 0)
})

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
  // Assign global stagger index across all groups
  let idx = 0
  for (const [, items] of groups) {
    for (const eq of items) { (eq as any)._i = idx++ }
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

async function loadEditor(euid: number) {
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
  } finally {
    editorLoading.value = false
  }
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
  loadEditor(euid)
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
  nextTick(() => applyStaggeredAnimation())
}

function getEnhanceSum(): number {
  const filled = editSubProps.value.filter(p => p !== null).length
  const totalAdd = editSubProps.value.reduce((s, p) => s + (p?.add_value || 0), 0)
  return totalAdd - filled
}

async function saveEquip() {
  if (!editorData.value || !uid.value || !selectedEquipUid.value) return
  saving.value = true
  const euid = selectedEquipUid.value
  const savedUid = uid.value
  const oldData = {
    level: editLevel.value, star: editStar.value,
    properties: [editMainProp.value], sub_properties: editSubProps.value,
  }
  try {
    const r = await api.updateEquip(uid.value, euid, {
      level: editLevel.value,
      star: editStar.value,
      properties: [editMainProp.value],
      sub_properties: editSubProps.value,
    })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('驱动盘数据已保存', 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.updateEquip(savedUid, euid, oldData)
          markCacheDirty()
          await refreshCache()
          selectedEquipUid.value = euid
          equipView.value = 'editor'
          await loadEditor(euid)
          toast('已撤回保存', 'info')
        } catch { toast('撤回失败', 'error') }
      }
    })
    markClean()
    markCacheDirty()
    await refreshCache()
    backToGallery()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
  saving.value = false
}

async function deleteEquip() {
  if (!uid.value || !selectedEquipUid.value || !editorData.value) return
  const euid = selectedEquipUid.value
  const savedUid = uid.value
  const snapData = { ...editorData.value }
  try {
    const r = await api.deleteEquip(uid.value, euid)
    if (r.ok === false) throw new Error(r.error || '删除失败')
    toast('驱动盘已删除', 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.createEquip(savedUid, {
            id: snapData.id, level: snapData.level, star: snapData.star,
            properties: snapData.properties, sub_properties: snapData.sub_properties,
          })
          markCacheDirty()
          await refreshCache()
          toast('已撤回删除', 'info')
        } catch { toast('撤回失败', 'error') }
      }
    })
    markCacheDirty()
    backToGallery()
    await refreshCache()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '删除失败', 'error')
  }
}

async function copyEquip() {
  if (!uid.value || !selectedEquipUid.value) return
  const savedUid = uid.value
  try {
    const r = await api.copyEquip(uid.value, selectedEquipUid.value)
    if (r.ok === false || r.uid == null) throw new Error(r.error || '复制失败')
    const newUid = r.uid
    toast(`驱动盘已复制为 #${newUid}`, 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.deleteEquip(savedUid, newUid)
          markCacheDirty()
          await refreshCache()
          toast('已撤回复制', 'info')
        } catch { toast('撤回失败', 'error') }
      }
    })
    markCacheDirty()
    await refreshCache()
    backToGallery()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '复制失败', 'error')
  }
}

// ─── 编辑器副属性操作 ──────────────────────────────

function onEditSubKeyChange(index: number, key: number) {
  const arr = [...editSubProps.value]
  if (key === 0) {
    arr[index] = null
  } else {
    const opt = subStatOptions.value.find(o => o.key === key)
    arr[index] = { key, key_name: opt?.name || '', base_value: opt?.base_value || SUB_STAT_BASE_VALUES[key] || 0, add_value: 1 }
  }
  editSubProps.value = arr
  markDirty()
}

function onEditSubBaseChange(index: number, value: number) {
  const arr = [...editSubProps.value]
  const p = arr[index]
  if (p) { arr[index] = { ...p, base_value: value }; markDirty() }
  editSubProps.value = arr
}

function onEditSubAddChange(index: number, value: number) {
  const arr = [...editSubProps.value]
  const p = arr[index]
  if (p) { arr[index] = { ...p, add_value: Math.max(0, value) }; markDirty() }
  editSubProps.value = arr
}

// ─── 创建驱动盘流程 ────────────────────────────────

function openCreate() {
  showCreate.value = true
  createStep.value = 1
  createSuitType.value = 0
  createSuitName.value = ''
  createEquipId.value = 0
  createSlot.value = 0
  createSlotName.value = ''
  createLevel.value = 15
  createStar.value = 5
  createMainKey.value = 0
  createMainName.value = ''
  createMainBase.value = 0
  createSubProps.value = [
    { key: 0, name: '', base: 0, add: 1 },
    { key: 0, name: '', base: 0, add: 1 },
    { key: 0, name: '', base: 0, add: 1 },
    { key: 0, name: '', base: 0, add: 1 },
  ]
}

function closeCreate() {
  showCreate.value = false
}

function selectCreateSuit(suitType: number, suitName: string) {
  createSuitType.value = suitType
  createSuitName.value = suitName
  createStep.value = 2
}

function selectCreateSlot(slot: { id: number; slot: number; slot_name: string }) {
  createEquipId.value = slot.id
  createSlot.value = slot.slot
  createSlotName.value = slot.slot_name
  createStep.value = 3
  const opts = currentMainOptions.value
  if (opts.length > 0) {
    createMainKey.value = opts[0].key
    createMainName.value = opts[0].name
    createMainBase.value = opts[0].base_value || MAIN_STAT_BASE_VALUES[opts[0].key] || 0
  }
}

function backToSuits() { createStep.value = 1 }
function backToSlots() { createStep.value = 2 }

// ZZZ 驱动盘主属性默认基础值 (from Python version MAIN_STAT_BASE_VALUES)
const MAIN_STAT_BASE_VALUES: Record<number, number> = {
  11103: 550,   // 生命值
  12103: 79,    // 攻击力
  13103: 46,    // 防御力
  11102: 750,   // 生命值%
  12102: 750,   // 攻击力%
  13102: 1200,  // 防御力%
  20103: 600,   // 暴击率
  21103: 1200,  // 暴击伤害
  23103: 600,   // 穿透率
  31203: 23,    // 异常精通
  31402: 750,   // 异常掌控
  12202: 450,   // 冲击力
  30502: 1500,  // 能量自动回复
  31503: 750,   // 物理伤害加成
  31603: 750,   // 火属性伤害加成
  31703: 750,   // 冰属性伤害加成
  31803: 750,   // 电属性伤害加成
  31903: 750,   // 以太属性伤害加成
}

// 副属性基础值 (from Python version SUB_STAT_BASE_VALUES)
const SUB_STAT_BASE_VALUES: Record<number, number> = {
  11103: 112,   // 生命值
  11102: 300,   // 生命值%
  12103: 19,    // 攻击力
  12102: 300,   // 攻击力%
  13103: 15,    // 防御力
  13102: 480,   // 防御力%
  23203: 9,     // 穿透值
  31203: 9,     // 异常精通
  21103: 480,   // 暴击伤害
  20103: 240,   // 暴击率
}

function onMainKeyChange() {
  const opt = currentMainOptions.value.find(o => o.key === createMainKey.value)
  createMainName.value = opt?.name || ''
  createMainBase.value = opt?.base_value || MAIN_STAT_BASE_VALUES[createMainKey.value] || 0
}

function onSubKeyChange(index: number, key: number) {
  const prop = createSubProps.value[index]
  prop.key = key
  if (key === 0) {
    prop.name = ''
    prop.base = 0
    prop.add = 1  // Keep add=1 to avoid negative enhance sum
  } else {
    const opt = subStatOptions.value.find(o => o.key === key)
    prop.name = opt?.name || ''
    prop.base = opt?.base_value || SUB_STAT_BASE_VALUES[key] || 0
    prop.add = 1  // Default add_value = 1 (no enhancement)
  }
}

async function submitCreate() {
  if (!uid.value || !createEquipId.value) return

  if (createEnhanceSum.value < 4 || createEnhanceSum.value > 5) {
    toast(`副属性追加强化总和必须为 4-5，当前为 ${createEnhanceSum.value}`, 'error')
    return
  }

  const activeKeys = createSubProps.value.filter(p => p.key > 0)
  const keys = activeKeys.map(p => p.key)
  if (new Set(keys).size !== keys.length) {
    toast('副属性种类不能重复', 'error')
    return
  }

  const data: EquipCreate = {
    id: createEquipId.value,
    level: createLevel.value,
    star: createStar.value,
    properties: [{
      key: createMainKey.value,
      key_name: '',
      base_value: createMainBase.value,
      add_value: 0,
    }],
    sub_properties: createSubProps.value.map(p => {
      if (p.key === 0) return null
      return {
        key: p.key,
        key_name: '',
        base_value: p.base,
        add_value: p.add,
      }
    }),
  }

  try {
    const r = await api.createEquip(uid.value, data)
    if (r.ok === false || r.uid == null) throw new Error(r.error || '创建失败')
    const newUid = r.uid
    const savedUid = uid.value
    toast(`驱动盘 #${newUid} 已创建`, 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.deleteEquip(savedUid, newUid)
          markCacheDirty()
          await refreshCache()
          toast('已撤回创建', 'info')
        } catch { toast('撤回失败', 'error') }
      }
    })
    closeCreate()
    markCacheDirty()
    await refreshCache()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '创建失败', 'error')
  }
}

async function refreshCache() {
  if (!uid.value) return
  if (equipCache.value.length && !cacheDirty.value) {
    loading.value = false
    return
  }
  if (refreshing.value) return
  refreshing.value = true
  try {
    const data = await api.getEquips(uid.value)
    equipCache.value = data.equips
    cacheDirty.value = false
  } catch (e: unknown) {
    toast('加载驱动盘失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
  refreshing.value = false
  loading.value = false
}

onMounted(async () => {
  await refreshCache()
  if (equipView.value === 'editor' && selectedEquipUid.value) {
    loadEditor(selectedEquipUid.value)
  }
  if (equipView.value === 'gallery') {
    applyStaggeredAnimation()
  }
})

// 离开面板时重置为仓库视图
watch(panel, (_, old) => { if (old === 'equips') { equipView.value = 'gallery'; selectedEquipUid.value = null; searchQuery.equips = '' } })

// Track unsaved changes for level/star (sub-properties already tracked via change handlers)
watch([editLevel, editStar], () => { if (equipView.value === 'editor') markDirty() })

onActivated(async () => {
  await refreshCache()
  nextTick(() => applyStaggeredAnimation())
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
      <div class="section-title">副属性 · {{ editSubProps.filter(p => p).length }} 条</div>
      <div class="prop-header">
        <span>#</span><span>属性</span><span>强化次数</span>
      </div>
      <div v-for="(prop, i) in editSubProps" :key="i" class="prop-row">
        <span class="prop-index">{{ i + 1 }}</span>
        <select class="form-select form-select--flex2"
          :value="prop?.key || 0"
          @change="onEditSubKeyChange(i, Number(($event.target as HTMLSelectElement).value))"
        >
          <option :value="0">— 无 —</option>
          <option v-for="opt in subStatOptions" :key="opt.key" :value="opt.key">
            {{ opt.name }}
          </option>
        </select>
        <div class="input-stepper">
          <button class="stepper-btn" @click="onEditSubAddChange(i, Math.max(1, (prop?.add_value || 1) - 1))" :disabled="!prop">−</button>
          <input class="stepper-input" type="number" :value="(prop?.add_value || 1) - 1" min="0" max="19"
            @input="onEditSubAddChange(i, Number(($event.target as HTMLInputElement).value) + 1)"
            :disabled="!prop">
          <button class="stepper-btn" @click="onEditSubAddChange(i, (prop?.add_value || 1) + 1)" :disabled="!prop">+</button>
        </div>
      </div>
      <div class="enhance-sum" :class="{
        'enhance-sum--valid': getEnhanceSum() >= 4 && getEnhanceSum() <= 5,
        'enhance-sum--warn': getEnhanceSum() < 4 || getEnhanceSum() > 5,
      }">
        强化总和: +{{ getEnhanceSum() }}
      </div>
    </div>

    <div class="editor-page__actions editor-fab-group" v-if="editorData">
      <span class="text-xs text-muted" style="margin-right:auto;opacity:0.6">Ctrl+Z 可撤回操作</span>
      <button class="btn btn-danger" @click="deleteEquip">删除</button>
      <button class="btn btn-ghost" @click="copyEquip">复制</button>
      <button class="btn btn-primary" :class="{ 'btn--saving': saving }" :disabled="saving" @click="saveEquip">{{ saving ? '保存中...' : '保存更改' }}</button>
    </div>
  </div>

  <!-- Gallery -->
  <div v-else>
    <div class="page-header flex-between">
      <div>
        <h2>驱动盘仓库</h2>
        <span class="subtitle text-muted">管理驱动盘数据，包括主属性与副属性</span>
      </div>
      <button class="btn btn-success" @click="openCreate">+ 创建驱动盘</button>
    </div>

    <!-- 创建驱动盘 — Modal overlay (Python-style) -->
    <Teleport to="body">
      <div v-if="showCreate" class="modal-overlay" @click.self="closeCreate">
        <div class="modal modal--wide">
          <div class="create-steps">
            <span class="create-step" :class="{ active: createStep === 1 }">1. 选择套装</span>
            <span class="create-step-arrow">→</span>
            <span class="create-step" :class="{ active: createStep === 2 }">2. 选择位置</span>
            <span class="create-step-arrow">→</span>
            <span class="create-step" :class="{ active: createStep === 3 }">3. 配置属性</span>
          </div>
          <h2>
            创建驱动盘
            <span class="text-accent" v-if="createStep === 1"> — 选择套装</span>
            <span class="text-accent" v-else-if="createStep === 2"> — {{ createSuitName }}</span>
            <span class="text-accent" v-else> — {{ createSuitName }} · {{ createSlotName }}号位</span>
          </h2>

          <!-- Step 1: 选择套装 -->
          <div v-if="createStep === 1" class="suit-grid-scroll">
            <div
              v-for="suit in suitList"
              :key="suit.suit_type"
              class="suit-card"
              tabindex="0" role="button"
              @click="selectCreateSuit(suit.suit_type, suit.suit_name)"
            >
              <div class="suit-card__name">{{ suit.suit_name }}</div>
            </div>
          </div>

          <!-- Step 2: 选择位置 -->
          <div v-if="createStep === 2">
            <button class="btn btn-ghost mb-3" @click="backToSuits">← 返回选择套装</button>
            <div class="slot-grid">
              <div
                v-for="n in 6"
                :key="n"
                class="slot-card"
                tabindex="0" role="button"
                @click="selectCreateSlot(currentSuitSlots[n-1] || { id: 0, slot: n, slot_name: n + '号位' })"
              >
                <div class="slot-card__num">{{ n }}</div>
              </div>
            </div>
          </div>

          <!-- Step 3: 配置属性 -->
          <div v-if="createStep === 3">
            <button class="btn btn-ghost mb-3" @click="backToSlots">← 返回选择位置</button>
            <div class="section-title">{{ createSuitName }} · {{ createSlotName }}号位 · 主属性</div>
            <div class="prop-row">
              <select
                v-model="createMainKey"
                class="form-select form-select--flex2"
                :disabled="isSlotFixed"
                @change="onMainKeyChange"
              >
                <option v-for="opt in currentMainOptions" :key="opt.key" :value="opt.key">
                  {{ opt.name }}
                </option>
              </select>
              <input
                class="form-input prop-value-readonly"
                type="number"
                v-model.number="createMainBase"
                placeholder="基础值"
              >
              <span class="text-muted prop-add-label">+0</span>
            </div>
            <div v-if="isSlotFixed" class="text-sm text-muted hint-text">1-3号位主属性固定，不可更改</div>

            <div class="section-title">副属性 · 4 条 <span class="text-sm text-muted">（追加强化 0-4）</span></div>
            <div class="prop-header">
              <span>#</span><span>属性</span><span>强化次数</span>
            </div>
            <div v-for="(prop, i) in createSubProps" :key="i" class="prop-row">
              <span class="prop-index">{{ i + 1 }}</span>
              <select
                class="form-select form-select--flex2"
                :value="prop.key"
                @change="onSubKeyChange(i, Number(($event.target as HTMLSelectElement).value))"
              >
                <option :value="0">— 无 —</option>
                <option v-for="opt in subStatOptions" :key="opt.key" :value="opt.key">
                  {{ opt.name }}
                </option>
              </select>
              <div class="input-stepper">
                <button class="stepper-btn" @click="prop.add = Math.max(1, prop.add - 1)" :disabled="prop.key === 0">−</button>
                <input class="stepper-input" type="number" :value="prop.add - 1" min="0" max="3"
                  @input="prop.add = Number(($event.target as HTMLInputElement).value) + 1"
                  :disabled="prop.key === 0">
                <button class="stepper-btn" @click="prop.add = Math.min(4, prop.add + 1)" :disabled="prop.key === 0">+</button>
              </div>
            </div>

            <div
              class="enhance-sum"
              :class="{
                'enhance-sum--valid': createEnhanceSum >= 4 && createEnhanceSum <= 5,
                'enhance-sum--warn': createEnhanceSum < 4 || createEnhanceSum > 5,
              }"
            >
              追加强化总和: <strong>{{ createEnhanceSum }}</strong> / 5
            </div>

            <div class="btn-group">
              <button class="btn btn-ghost" @click="closeCreate">取消</button>
              <button class="btn btn-success btn-lg" @click="submitCreate">创建驱动盘</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

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
            class="game-card equip-card staggered-anim"
            :class="suitColorClass(eq.suit_name)"
            :style="{ '--i': (eq as any)._i }"
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
