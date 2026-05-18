<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  uid, weaponCache, searchQuery,
  selectedWeaponUid, weaponView, markDirty,
} from '@/composables/useAppState'
import { api } from '@/lib/api'
import { WEAPON_PINYIN } from '@/assets/pinyin-data'
import type { WeaponListItem, WeaponDetail } from '@/lib/types'
import { usePanelEditor } from '@/composables/usePanelEditor'
import SearchBar from '@/components/shared/SearchBar.vue'
import Stepper from '@/components/shared/Stepper.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'
import { PROFESSION_ORDER } from '@/constants'

// Edit refs
const editLevel = ref(60)
const editRefine = ref(1)
const editStar = ref(1)

const {
  loading, editorData, editorLoading, saving,
  filteredItems: filteredWeapons,
  groupedItems: groupedWeapons,
  staggerIndex,
  selectItem: selectWeapon,
  backToGallery,
  saveItem: saveWeapon,
  deleteItem: deleteWeapon,
  copyItem: copyWeapon,
} = usePanelEditor<WeaponListItem, WeaponDetail>({
  panelKey: 'weapons',
  entityName: '音擎',

  loadList: async () => {
    const data = await api.getWeapons(uid.value!)
    return { items: data.weapons }
  },
  loadDetail: (u, id) => api.getWeapon(u, id),
  saveDetail: (u, id) => api.updateWeapon(u, id, {
    level: editLevel.value,
    star: editStar.value,
    refine_level: editRefine.value,
  }),
  deleteDetail: (u, id) => api.deleteWeapon(u, id),
  copyDetail: (u, id) => api.copyWeapon(u, id),

  getItemId: (w) => w.uid,
  getDetailId: (w) => w.uid,

  cache: weaponCache,
  selectedId: selectedWeaponUid,
  viewRef: weaponView,

  filterFn: (w, q) => {
    q = q.toLowerCase()
    // 排除 B 级（ID < 13000）和 NPC 音擎（12000-12999）
    if (w.id < 13000) return false
    const py = WEAPON_PINYIN[w.id]
    if (py && (py.full.includes(q) || py.initials.includes(q))) return true
    return String(w.id).includes(q) || String(w.uid).includes(q)
      || w.name.toLowerCase().includes(q)
      || w.profession.toLowerCase().includes(q)
  },
  groupFn: (w) => w.profession || '未知',
  groupSort: (a, b) => {
    const ia = PROFESSION_ORDER.indexOf(a[0] as typeof PROFESSION_ORDER[number])
    const ib = PROFESSION_ORDER.indexOf(b[0] as typeof PROFESSION_ORDER[number])
    return (ia === -1 ? 999 : ia) - (ib === -1 ? 999 : ib)
  },

  mapDetailToEdit: (w) => {
    editLevel.value = w.level
    editStar.value = w.star
    editRefine.value = w.refine_level
  },
  buildSavePayload: () => ({
    level: editLevel.value,
    star: editStar.value,
    refine_level: editRefine.value,
  }),
  snapshotOldData: (w) => ({
    level: w.level, star: w.star, refine_level: w.refine_level,
    id: w.id, lock: w.lock, exp: 0,
  }),
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

// Track unsaved changes
watch([editLevel, editStar, editRefine], () => { if (weaponView.value === 'editor') markDirty('weapons') })
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
      <span class="text-xs text-muted" style="margin-right:auto;opacity:0.6">Ctrl+Z 可撤回操作</span>
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
            :style="{ '--i': staggerIndex.get(w.uid) }"
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
