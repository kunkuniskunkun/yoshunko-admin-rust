<script setup lang="ts">
import { ref, computed, onMounted, onActivated, nextTick, watch } from 'vue'
import {
  uid, panel, weaponCache, cacheDirty, searchQuery,
  selectedWeaponUid, weaponView, markCacheDirty, markDirty, markClean, pushUndo,
} from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { WEAPON_PINYIN } from '@/assets/pinyin-data'
import type { WeaponDetail } from '@/lib/types'
import SearchBar from '@/components/shared/SearchBar.vue'
import Stepper from '@/components/shared/Stepper.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'
import { applyStaggeredAnimation, applyEditorSlideIn } from '@/composables/useStaggeredAnimation'
import { PROFESSION_ORDER, NPC_WEAPON_ID_MIN, NPC_WEAPON_ID_MAX } from '@/constants'

const loading = ref(true)
const refreshing = ref(false)
const editorData = ref<WeaponDetail | null>(null)
const editorLoading = ref(false)
const editLevel = ref(60)
const editRefine = ref(1)
const editStar = ref(1)
const saving = ref(false)

const filteredWeapons = computed(() => {
  let list = weaponCache.value.filter(w => w.id < NPC_WEAPON_ID_MIN || w.id > NPC_WEAPON_ID_MAX)
  list.sort((a, b) => b.uid - a.uid)
  const q = searchQuery.weapons.toLowerCase()
  if (q) {
    list = list.filter(w => {
      const py = WEAPON_PINYIN[w.id]
      if (py && (py.full.includes(q) || py.initials.includes(q))) return true
      return String(w.id).includes(q) || String(w.uid).includes(q)
        || w.name.toLowerCase().includes(q)
        || w.profession.toLowerCase().includes(q)
    })
  }
  return list
})

const groupedWeapons = computed(() => {
  const groups = new Map<string, typeof filteredWeapons.value>()
  for (const w of filteredWeapons.value) {
    const key = w.profession || '未知'
    if (!groups.has(key)) groups.set(key, [])
    groups.get(key)!.push(w)
  }
  const sorted = new Map<string, typeof filteredWeapons.value>()
  for (const p of PROFESSION_ORDER) {
    if (groups.has(p)) sorted.set(p, groups.get(p)!)
  }
  for (const [k, v] of groups) {
    if (!sorted.has(k)) sorted.set(k, v)
  }
  // Assign global stagger index across all groups
  let idx = 0
  for (const [, weapons] of sorted) {
    for (const w of weapons) { (w as any)._i = idx++ }
  }
  return sorted
})

function rarityClass(id: number): 's' | 'a' | 'b' | undefined {
  if (id >= 14000) return 's'
  if (id >= 13000) return 'a'
  return undefined
}

function rarityLabel(id: number): string {
  if (id >= 14000) return 'S'
  if (id >= 13000) return 'A'
  return 'B'
}

async function loadEditor(wuid: number) {
  editorLoading.value = true
  try {
    const w = await api.getWeapon(uid.value!, wuid)
    if (selectedWeaponUid.value !== wuid) return
    if (!w) { toast('音擎数据未找到', 'error'); backToGallery(); return }
    editorData.value = w
    editLevel.value = w.level
    editStar.value = w.star
    editRefine.value = w.refine_level
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '加载失败', 'error')
    backToGallery()
  } finally {
    editorLoading.value = false
  }
}

async function selectWeapon(wuid: number, event?: Event) {
  // Card press animation
  if (event?.currentTarget) {
    const el = event.currentTarget as HTMLElement
    el.style.transition = 'transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1)'
    el.style.transform = 'scale(0.92)'
    setTimeout(() => { el.style.transform = 'scale(1)' }, 120)
  }
  selectedWeaponUid.value = wuid
  weaponView.value = 'editor'
  loadEditor(wuid)
  // Editor slide-in
  nextTick(() => {
    const mainEl = document.querySelector('.main-content') as HTMLElement
    if (mainEl) applyEditorSlideIn(mainEl)
  })
}

function backToGallery() {
  weaponView.value = 'gallery'
  selectedWeaponUid.value = null
  editorData.value = null
  nextTick(() => applyStaggeredAnimation())
}

async function saveWeapon() {
  if (!editorData.value || !uid.value || !selectedWeaponUid.value) return
  saving.value = true
  const wuid = selectedWeaponUid.value
  const savedUid = uid.value
  const oldData = { level: editLevel.value, star: editStar.value, refine_level: editRefine.value }
  try {
    const r = await api.updateWeapon(uid.value, wuid, {
      level: editLevel.value,
      star: editStar.value,
      refine_level: editRefine.value,
    })
    if (r.ok === false) throw new Error(r.error || '保存失败')
    toast('音擎数据已保存', 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.updateWeapon(savedUid, wuid, oldData)
          markCacheDirty()
          await refreshCache()
          selectedWeaponUid.value = wuid
          weaponView.value = 'editor'
          await loadEditor(wuid)
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

async function copyWeapon() {
  if (!uid.value || !selectedWeaponUid.value) return
  const savedUid = uid.value
  try {
    const r = await api.copyWeapon(uid.value, selectedWeaponUid.value)
    if (r.ok === false) throw new Error(r.error || '复制失败')
    const newUid = r.uid
    toast(`音擎已复制为 #${newUid}`, 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.deleteWeapon(savedUid, newUid)
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

async function deleteWeapon() {
  if (!uid.value || !selectedWeaponUid.value || !editorData.value) return
  const wuid = selectedWeaponUid.value
  const savedUid = uid.value
  const snapData = { ...editorData.value }
  try {
    const r = await api.deleteWeapon(uid.value, wuid)
    if (r.ok === false) throw new Error(r.error || '删除失败')
    toast('音擎已删除', 'success')
    pushUndo({
      restore: async () => {
        try {
          await api.updateWeapon(savedUid, wuid, {
            id: snapData.id, level: snapData.level,
            star: snapData.star, refine_level: snapData.refine_level,
            lock: snapData.lock, exp: 0,
          })
          markCacheDirty()
          await refreshCache()
          selectedWeaponUid.value = wuid
          weaponView.value = 'editor'
          await loadEditor(wuid)
          toast('已撤回删除', 'info')
        } catch { toast('撤回失败', 'error') }
      }
    })
    markClean()
    markCacheDirty()
    backToGallery()
    await refreshCache()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '删除失败', 'error')
  }
}

async function refreshCache() {
  if (!uid.value) return
  if (weaponCache.value.length && !cacheDirty.value) {
    loading.value = false
    return
  }
  if (refreshing.value) return
  refreshing.value = true
  try {
    const data = await api.getWeapons(uid.value)
    weaponCache.value = data.weapons
    cacheDirty.value = false
  } catch (e: unknown) {
    toast('加载音擎失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
  refreshing.value = false
  loading.value = false
}

onMounted(async () => {
  await refreshCache()
  if (weaponView.value === 'editor' && selectedWeaponUid.value) {
    loadEditor(selectedWeaponUid.value)
  }
  if (weaponView.value === 'gallery') {
    applyStaggeredAnimation()
  }
})

// 离开面板时重置为仓库视图
watch(panel, (_, old) => { if (old === 'weapons') { weaponView.value = 'gallery'; selectedWeaponUid.value = null; searchQuery.weapons = '' } })

// Track unsaved changes
watch([editLevel, editStar, editRefine], () => { if (weaponView.value === 'editor') markDirty() })

onActivated(async () => {
  await refreshCache()
  nextTick(() => applyStaggeredAnimation())
})
</script>

<template>
  <!-- Editor -->
  <div v-if="weaponView === 'editor' && selectedWeaponUid" class="editor-page">
    <div class="editor-page__top">
      <a class="editor-back" href="#" @click.prevent="backToGallery">← 音擎仓库</a>
      <div class="editor-page__header" v-if="editorData">
        <h2>{{ editorData.name }}</h2>
        <span v-if="editorData.en_name" class="sub en-name text-muted">{{ editorData.en_name }}</span>
        <div class="editor-header-meta">
          <span class="slot-tag">{{ editorData.profession }}</span>
          <span class="star-rating">
            <span v-for="i in (editorData.max_refine || 5)" :key="i" class="star" :class="{ active: i <= editorData.refine_level }">★</span>
          </span>
          <span class="sub text-muted">#{{ selectedWeaponUid }}</span>
        </div>
      </div>
    </div>
    <div v-if="editorLoading" class="loading-wrap"><div class="spinner"></div></div>
    <div v-else-if="editorData" class="editor-page__body">
      <div class="section-title">基础属性</div>
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">等级</label>
          <Stepper v-model="editLevel" :min="1" :max="60" label="等级" />
        </div>
        <div class="form-field">
          <label class="form-label">星级</label>
          <Stepper v-model="editStar" :min="1" :max="editorData.max_star || 5" label="星级" />
          <span class="text-xs text-muted">max: {{ editorData.max_star || 5 }}</span>
        </div>
        <div class="form-field">
          <label class="form-label">精炼等级</label>
          <Stepper v-model="editRefine" :min="1" :max="editorData.max_refine || 5" label="精炼" />
          <span class="text-xs text-muted">max: {{ editorData.max_refine || 5 }}</span>
        </div>
      </div>
    </div>
    <div class="editor-page__actions editor-fab-group" v-if="editorData">
      <button class="btn btn-danger" @click="deleteWeapon">删除</button>
      <button class="btn btn-ghost" @click="copyWeapon">复制</button>
      <button class="btn btn-primary" :class="{ 'btn--saving': saving }" :disabled="saving" @click="saveWeapon">{{ saving ? '保存中...' : '保存更改' }}</button>
    </div>
  </div>

  <!-- Gallery -->
  <div v-else>
    <div class="page-header">
      <h2>音擎仓库</h2>
      <span class="subtitle text-muted">管理音擎等级、星级突破与精炼等级</span>
    </div>
    <SearchBar v-model="searchQuery.weapons" placeholder="搜索音擎 ID、名称、职业..." />
    <SkeletonGrid v-if="loading" />
    <div v-else-if="filteredWeapons.length === 0" class="empty-state">
      <div class="empty-state__icon"></div><p>没有找到匹配的音擎</p>
    </div>
    <div v-else class="avatar-gallery">
      <div v-for="[prof, weapons] in groupedWeapons" :key="prof" class="avatar-gallery__camp-section">
        <div class="avatar-gallery__camp-header">{{ prof }} <span class="text-xs text-muted">({{ weapons.length }})</span></div>
        <div class="avatar-gallery__grid">
          <div
            v-for="w in weapons"
            :key="w.uid"
            class="game-card avatar-gallery__card staggered-anim"
            :class="rarityClass(w.id) === 's' ? 'avatar-gallery__card--s' : rarityClass(w.id) === 'a' ? 'avatar-gallery__card--a' : ''"
            :style="{ '--i': (w as any)._i }"
            tabindex="0" role="button"
            @click="selectWeapon(w.uid, $event)"
          >
            <div class="card-header">
              <span>
                <span v-if="rarityClass(w.id)" class="game-card__rarity" :class="rarityClass(w.id) === 's' ? 'rarity-s' : 'rarity-a'">{{ rarityLabel(w.id) }}</span>
                <span class="card-title">{{ w.name }}</span>
              </span>
              <span class="game-card__level">Lv.{{ w.level }}</span>
            </div>
            <div class="card-meta">
              <span class="star-rating">
                <span v-for="i in (w.max_refine || 5)" :key="i" class="star" :class="{ active: i <= w.refine_level }">★</span>
              </span>
              <span class="refine-pill">R{{ w.refine_level }}</span>
            </div>
            <div class="avatar-card__footer">
              <span class="avatar-card__camp-tag">{{ prof }}</span>
              <span class="text-xs text-muted">#{{ w.uid }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
