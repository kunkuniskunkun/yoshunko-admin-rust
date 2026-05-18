import { ref, reactive, shallowRef, computed } from 'vue'
import type { AvatarListItem, WeaponListItem, EquipListItem, Templates, SkillTypeLevel } from '@/lib/types'

// ─── 核心状态 ──────────────────────────────────────

export const uid = ref<number | null>(null)
export const panel = ref<string>('avatars')
export const templates = shallowRef<Templates | null>(null)

// 缓存
export const avatarCache = shallowRef<AvatarListItem[]>([])
export const weaponCache = shallowRef<WeaponListItem[]>([])
export const equipCache = shallowRef<EquipListItem[]>([])
export const cacheDirty = reactive({ avatars: false, weapons: false, equips: false })

// 视图模式
export const avatarView = ref<'gallery' | 'editor'>('gallery')
export const weaponView = ref<'gallery' | 'editor'>('gallery')
export const equipView = ref<'gallery' | 'editor'>('gallery')

// 选中项
export const selectedAvatarId = ref<number | null>(null)
export const selectedWeaponUid = ref<number | null>(null)
export const selectedEquipUid = ref<number | null>(null)

// 分组
export const avatarGroupBy = ref<'camp' | 'profession'>('camp')

// 搜索
export const searchQuery = reactive({ avatars: '', weapons: '', equips: '' })

// 技能数据
export const skillTypes = ref<Record<number, Record<string, string>>>({})
export const skillData = ref<Record<number, Record<string, number>>>({})

// 滚动位置
export const scrollPos = ref<Record<string, number>>({})

// Dirty (per-panel)
export const dirty = reactive({ avatars: false, weapons: false, equips: false })

// Config state
export const configured = ref(false)

// ─── 模板 Map ──────────────────────────────────────

export const avatarMap = computed(() => {
  const map = new Map<number, { name: string; rarity: string; camp_id: number; camp_name: string }>()
  if (!templates.value) return map
  for (const a of templates.value.avatars) {
    map.set(a.id, {
      name: a.name,
      rarity: a.rarity,
      camp_id: a.camp_id,
      camp_name: a.camp_name,
    })
  }
  return map
})

export const weaponMap = computed(() => {
  const map = new Map<number, { name: string; rarity: string; profession: string; max_star: number; max_refine: number }>()
  if (!templates.value) return map
  for (const w of templates.value.weapons) {
    map.set(w.id, {
      name: w.name,
      rarity: w.rarity,
      profession: w.profession,
      max_star: w.max_star,
      max_refine: w.max_refine,
    })
  }
  return map
})

// ─── Undo 栈 ──────────────────────────────────────

interface Snapshot {
  restore: () => void
}

const undoStack = ref<Snapshot[]>([])
const MAX_UNDO = 20

export function pushUndo(snap: Snapshot) {
  undoStack.value.push(snap)
  if (undoStack.value.length > MAX_UNDO) undoStack.value.shift()
}

export function popUndo(): Snapshot | undefined {
  return undoStack.value.pop()
}

// ─── 辅助函数 ──────────────────────────────────────

export function avatarName(id: number): string {
  return avatarMap.value.get(id)?.name || `#${id}`
}

export function avatarRarity(id: number): string {
  return avatarMap.value.get(id)?.rarity || ''
}

export function avatarCamp(id: number): string {
  return avatarMap.value.get(id)?.camp_name || ''
}

export function weaponName(id: number): string {
  return weaponMap.value.get(id)?.name || `#${id}`
}

export function markDirty(panelKey: 'avatars' | 'weapons' | 'equips') { dirty[panelKey] = true }
export function markClean(panelKey: 'avatars' | 'weapons' | 'equips') { dirty[panelKey] = false }
export function markCacheDirty(panelKey: 'avatars' | 'weapons' | 'equips') { cacheDirty[panelKey] = true }
export function markAllCacheDirty() { cacheDirty.avatars = true; cacheDirty.weapons = true; cacheDirty.equips = true }
