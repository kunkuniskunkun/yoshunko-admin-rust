<script setup lang="ts">
import { ref, computed, onMounted, onActivated, nextTick, watch } from 'vue'
import type { Ref } from 'vue'
import {
  uid, panel, avatarCache, cacheDirty, avatarGroupBy, searchQuery,
  selectedAvatarId, avatarView, templates, avatarMap,
  markCacheDirty, markDirty, markClean,
} from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { AVATAR_PINYIN } from '@/assets/pinyin-data'
import type { AvatarListItem, AvatarDetail, SkillTypeLevel } from '@/lib/types'
import SearchBar from '@/components/shared/SearchBar.vue'
import Stepper from '@/components/shared/Stepper.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'
import { applyStaggeredAnimation, applyEditorSlideIn } from '@/composables/useStaggeredAnimation'

const loading = ref(true)
const refreshing = ref(false)
const editorData = ref<AvatarDetail | null>(null)
const editorLoading = ref(false)

// Editor fields
const editLevel = ref(60)
const editTalent = ref(0)
const editPassive = ref(0)
const editAwake = ref(0)
const editWeaponUid = ref(0)
const editSkinId = ref(0)
const editSkills = ref<{ type: string; level: number }[]>([])
const saving = ref(false)

const PROFESSION_ORDER = ['强攻', '击破', '异常', '支援', '防护', '命破']
const SKILL_NAMES: Record<string, string> = {
  common_attack: '普攻',
  special_attack: '强化特殊技',
  evade: '闪避',
  cooperate_skill: '连携技',
  unique_skill: '终结技',
  assist_skill: '支援技',
  core_skill: '核心被动',
}

const EXCLUDED_AVATAR_IDS = [2071, 2121] // NPC 角色，不可编辑

const filteredAvatars = computed(() => {
  let list = avatarCache.value.filter(a => !EXCLUDED_AVATAR_IDS.includes(a.avatar_id))
  const q = searchQuery.avatars.toLowerCase()
  if (q) {
    list = list.filter(a => {
      const py = AVATAR_PINYIN[a.avatar_id]
      if (py && (py.full.includes(q) || py.initials.includes(q))) return true
      if (String(a.avatar_id).includes(q)) return true
      if (a.name.toLowerCase().includes(q)) return true
      if (a.en_name.toLowerCase().includes(q)) return true
      if (a.profession.toLowerCase().includes(q)) return true
      const t = avatarMap.value.get(a.avatar_id)
      if (t && t.camp_name.toLowerCase().includes(q)) return true
      return false
    })
  }
  return list
})

const groupedAvatars = computed(() => {
  const groups = new Map<string, AvatarListItem[]>()
  for (const a of filteredAvatars.value) {
    let key: string
    if (avatarGroupBy.value === 'camp') {
      const t = avatarMap.value.get(a.avatar_id)
      key = t?.camp_name || '未知'
    } else {
      key = a.profession || '未知'
    }
    if (!groups.has(key)) groups.set(key, [])
    groups.get(key)!.push(a)
  }
  // Sort within groups
  for (const [, avatars] of groups) {
    avatars.sort((a, b) => a.avatar_id - b.avatar_id)
  }
  // Sort groups
  let sorted: Map<string, AvatarListItem[]>
  if (avatarGroupBy.value === 'profession') {
    sorted = new Map<string, AvatarListItem[]>()
    for (const p of PROFESSION_ORDER) {
      if (groups.has(p)) sorted.set(p, groups.get(p)!)
    }
    for (const [k, v] of groups) {
      if (!sorted.has(k)) sorted.set(k, v)
    }
  } else {
    sorted = new Map([...groups.entries()].sort((a, b) => {
      const tA = avatarMap.value.get(a[1][0]?.avatar_id ?? 0)
      const tB = avatarMap.value.get(b[1][0]?.avatar_id ?? 0)
      return (tA?.camp_id ?? 0) - (tB?.camp_id ?? 0)
    }))
  }
  // Assign global stagger index across all groups
  let idx = 0
  for (const [, avatars] of sorted) {
    for (const a of avatars) { (a as any)._i = idx++ }
  }
  return sorted
})

function rarityLabel(rarity: string): string {
  if (rarity === 'S') return 'S'
  if (rarity === 'A') return 'A'
  return 'B'
}

function rarityClass(rarity: string): string {
  return rarity === 'S' ? 'avatar-gallery__card--s' : 'avatar-gallery__card--a'
}

function selectAvatar(id: number, event?: Event) {
  // Card press animation
  if (event?.currentTarget) {
    const el = event.currentTarget as HTMLElement
    el.style.transition = 'transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1)'
    el.style.transform = 'scale(0.92)'
    setTimeout(() => { el.style.transform = 'scale(1)' }, 120)
  }
  selectedAvatarId.value = id
  avatarView.value = 'editor'
  loadEditor(id)
  // Editor slide-in animation
  nextTick(() => {
    const mainEl = document.querySelector('.main-content') as HTMLElement
    if (mainEl) applyEditorSlideIn(mainEl)
  })
}

function backToGallery() {
  avatarView.value = 'gallery'
  selectedAvatarId.value = null
  editorData.value = null
  nextTick(() => applyStaggeredAnimation())
}

async function loadEditor(aid: number) {
  editorLoading.value = true
  try {
    const r = await api.getAvatar(uid.value!, aid)
    if (selectedAvatarId.value !== aid) return
    if (!r) {
      toast('角色数据未找到', 'error')
      backToGallery()
      return
    }
    const av = r.avatar
    editorData.value = av
    editLevel.value = av.level
    editTalent.value = av.unlocked_talent_num
    editPassive.value = av.passive_skill_level
    editWeaponUid.value = av.cur_weapon_uid
    editSkinId.value = av.avatar_skin_id || 0
    // awake_id format: avatar_id*100 + (level-1), 0=未解锁
    editAwake.value = av.awake_id === 0 ? 0 : (av.awake_id % 100) + 1
    // Skills
    editSkills.value = av.skill_type_level
      .filter(s => s.type !== 'core_skill' && s.type !== 'unique_skill')
      .map(s => ({ type: s.type, level: s.level }))
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '加载失败', 'error')
    backToGallery()
  } finally {
    editorLoading.value = false
  }
}

async function saveAvatar() {
  if (!editorData.value || !uid.value || !selectedAvatarId.value) return
  saving.value = true
  const aid = selectedAvatarId.value
  const types = editorData.value.skill_type_level.map(s => s.type)

  // Build skill_type_level array
  const skillLevels: SkillTypeLevel[] = []
  for (const s of editSkills.value) {
    skillLevels.push({ type: s.type, level: s.level })
  }
  // unique_skill follows cooperate_skill
  const coopSkill = editSkills.value.find(s => s.type === 'cooperate_skill')
  const uniqueIdx = types.indexOf('unique_skill')
  if (uniqueIdx >= 0 && coopSkill) {
    skillLevels.splice(uniqueIdx, 0, { type: 'unique_skill', level: coopSkill.level })
  }
  // core_skill = 1 + passive_skill_level
  const coreIdx = types.indexOf('core_skill')
  skillLevels.splice(coreIdx >= 0 ? coreIdx : skillLevels.length, 0, { type: 'core_skill', level: 1 + editPassive.value })

  // awake_id
  const awakeId = editAwake.value === 0 ? 0 : aid * 100 + (editAwake.value - 1)

  try {
    const result = await api.updateAvatar(uid.value, aid, {
      level: editLevel.value,
      passive_skill_level: editPassive.value,
      unlocked_talent_num: editTalent.value,
      skill_type_level: skillLevels,
      awake_id: awakeId,
      avatar_skin_id: editSkinId.value,
      cur_weapon_uid: editWeaponUid.value,
    })
    if (result.ok === false) throw new Error(result.error || '保存失败')
    toast('角色数据已保存', 'success')
    markClean()
    markCacheDirty()
    await refreshCache()
    backToGallery()
  } catch (e: unknown) {
    toast(e instanceof Error ? e.message : '保存失败', 'error')
  }
  saving.value = false
}

async function refreshCache() {
  if (!uid.value) return
  if (avatarCache.value.length && !cacheDirty.value) {
    loading.value = false
    return
  }
  if (refreshing.value) return
  refreshing.value = true
  try {
    const data = await api.getAvatars(uid.value)
    avatarCache.value = data.avatars
    cacheDirty.value = false
  } catch (e: unknown) {
    toast('加载角色失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
  refreshing.value = false
  loading.value = false
}

onMounted(async () => {
  await refreshCache()
  if (avatarView.value === 'editor' && selectedAvatarId.value) {
    loadEditor(selectedAvatarId.value)
  }
  if (avatarView.value === 'gallery') {
    applyStaggeredAnimation()
  }
})

// 离开面板时重置为仓库视图
watch(panel, (_, old) => { if (old === 'avatars') { avatarView.value = 'gallery'; selectedAvatarId.value = null; searchQuery.avatars = '' } })

// Track unsaved changes
watch([editLevel, editTalent, editPassive, editAwake, editWeaponUid, editSkinId], () => { if (avatarView.value === 'editor') markDirty() })
watch(editSkills, () => { if (avatarView.value === 'editor') markDirty() }, { deep: true })

onActivated(async () => {
  await refreshCache()
  nextTick(() => applyStaggeredAnimation())
})

watch(filteredAvatars, () => {
  if (avatarView.value === 'gallery') {
    applyStaggeredAnimation()
  }
})
</script>

<template>
  <!-- Editor View -->
  <div v-if="avatarView === 'editor' && selectedAvatarId" class="editor-page">
    <div class="editor-page__top">
      <a class="editor-back" href="#" @click.prevent="backToGallery">← 角色仓库</a>
      <div class="editor-page__header" v-if="editorData">
        <h2>{{ editorData.name }}</h2>
        <span v-if="editorData.en_name" class="sub en-name text-muted">{{ editorData.en_name }}</span>
        <span class="sub text-muted">ID: {{ selectedAvatarId }}</span>
      </div>
    </div>

    <div v-if="editorLoading" class="loading-wrap"><div class="spinner"></div></div>

    <div v-else-if="editorData" class="editor-page__body">
      <!-- Basic Stats -->
      <div class="section-title">基础属性</div>
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">等级</label>
          <Stepper v-model="editLevel" :min="0" :max="60" label="等级" />
        </div>
        <div class="form-field">
          <label class="form-label">影画 (命座)</label>
          <Stepper v-model="editTalent" :min="0" :max="6" label="影画" />
        </div>
      </div>

      <!-- Skills -->
      <div class="section-title">技能等级</div>
      <div class="skill-grid">
        <div v-for="(skill, i) in editSkills" :key="skill.type" class="skill-card">
          <div class="skill-name">{{ SKILL_NAMES[skill.type] || skill.type }}</div>
          <Stepper v-model="editSkills[i].level" :min="1" :max="12" />
          <div class="text-xs text-muted">max 12</div>
        </div>
        <!-- Core passive -->
        <div class="skill-card">
          <div class="skill-name">核心被动</div>
          <Stepper v-model="editPassive" :min="0" :max="6" />
          <div class="text-xs text-muted">max 6</div>
        </div>
      </div>

      <!-- Awakening -->
      <div class="section-title">潜能激发</div>
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">潜能激发等级 (0-6)</label>
          <Stepper v-model="editAwake" :min="0" :max="6" label="潜能激发" />
        </div>
        <div class="form-field">
          <label class="form-label">当前武器 UID</label>
          <input class="form-input" type="number" v-model.number="editWeaponUid" min="0" />
        </div>
      </div>
      <div class="form-row">
        <div class="form-field">
          <label class="form-label">皮肤 ID (0=默认)</label>
          <Stepper v-model="editSkinId" :min="0" :max="999999" label="皮肤ID" />
        </div>
        <div class="form-field"></div>
      </div>
    </div>

    <div class="editor-page__actions" v-if="editorData">
      <button class="btn btn-primary" :class="{ 'btn--saving': saving }" :disabled="saving" @click="saveAvatar">{{ saving ? '保存中...' : '保存更改' }}</button>
    </div>
  </div>

  <!-- Gallery View -->
  <div v-else>
    <div class="page-header">
      <h2>角色仓库</h2>
      <span class="subtitle text-muted">管理等级、影画、技能、潜能激发</span>
    </div>

    <div class="search-bar-row">
      <SearchBar v-model="searchQuery.avatars" placeholder="搜索角色 ID、名称、阵营或职业..." />
      <button
        class="btn btn-ghost search-group-toggle"
        @click="avatarGroupBy = avatarGroupBy === 'camp' ? 'profession' : 'camp'"
      >
        {{ avatarGroupBy === 'camp' ? '按职业' : '按阵营' }}
      </button>
    </div>

    <SkeletonGrid v-if="loading" />

    <div v-else-if="filteredAvatars.length === 0" class="empty-state">
      <div class="empty-state__icon"></div>
      <p>没有找到匹配的角色</p>
    </div>

    <div v-else class="avatar-gallery">
      <div v-for="[group, avatars] in groupedAvatars" :key="group" class="avatar-gallery__camp-section">
        <div class="avatar-gallery__camp-header">
          {{ group }} <span class="text-xs text-muted">({{ avatars.length }})</span>
        </div>
        <div class="avatar-gallery__grid">
          <div
            v-for="a in avatars"
            :key="a.avatar_id"
            class="game-card avatar-gallery__card staggered-anim"
            :class="rarityClass(a.rarity)"
            :style="{ '--i': (a as any)._i }"
            tabindex="0"
            role="button"
            @click="selectAvatar(a.avatar_id, $event)"
            @keydown.enter="selectAvatar(a.avatar_id)"
            @keydown.space.prevent="selectAvatar(a.avatar_id)"
          >
            <div v-if="a.is_favorite" class="avatar-card__fav">★</div>
            <div class="card-header">
              <span>
                <span class="game-card__rarity" :class="a.rarity === 'S' ? 'rarity-s' : 'rarity-a'">
                  {{ a.rarity }}
                </span>
                <span class="card-title">{{ a.name }}</span>
              </span>
              <span class="game-card__level">Lv.{{ a.level }}</span>
            </div>
            <div class="card-meta">
              <span class="rank-dots">
                <span v-for="i in 6" :key="i" class="rank-dot" :class="{ active: i <= (a.unlocked_talent_num || 0) }"></span>
              </span>
              <span class="text-sm text-accent ml-1">{{ a.unlocked_talent_num || 0 }}影</span>
            </div>
            <div class="avatar-card__footer">
              <span class="avatar-card__camp-tag">{{ avatarMap.get(a.avatar_id)?.camp_name || '' }}</span>
              <span v-if="a.profession" class="avatar-card__camp-tag ml-1">{{ a.profession }}</span>
              <span class="text-xs text-muted">ID:{{ a.avatar_id }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
