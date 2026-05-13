// ─── Config ────────────────────────────────────────

export interface Config {
  configured: boolean
  config_exists: boolean
  state_dir: string
  version: string
  launch_config: Record<string, string>
}

// ─── Player ────────────────────────────────────────

export interface PlayerBasic {
  nickname: string
  level: number
  exp: number
  avatar_id: number
  control_avatar_id: number
  control_guise_avatar_id: number
}

export interface PlayerBasicUpdate {
  nickname?: string
  level?: number
  exp?: number
  avatar_id?: number
  control_avatar_id?: number
  control_guise_avatar_id?: number
}

// ─── Avatar ────────────────────────────────────────

export interface AvatarListItem {
  avatar_id: number
  name: string
  en_name: string
  rarity: string
  profession: string
  level: number
  unlocked_talent_num: number
  is_favorite: boolean
  camp_id: number
}

export interface AvatarDetail {
  avatar_id: number
  name: string
  en_name: string
  rarity: string
  profession: string
  level: number
  exp: number
  rank: number
  unlocked_talent_num: number
  talent_switch_list: boolean[]
  passive_skill_level: number
  cur_weapon_uid: number
  is_favorite: boolean
  avatar_skin_id: number
  is_awake_available: boolean
  awake_id: number
  cur_form_id: number
  is_awake_enabled: boolean
  dressed_equip: (number | null)[]
  show_weapon_type: string
  skill_type_level: SkillTypeLevel[]
}

export interface SkillTypeLevel {
  type: string
  level: number
}

export interface AvatarUpdate {
  level?: number
  passive_skill_level?: number
  unlocked_talent_num?: number
  skill_type_level?: SkillTypeLevel[]
  awake_id?: number
  avatar_skin_id?: number
  cur_weapon_uid?: number
}

// ─── Weapon ────────────────────────────────────────

export interface WeaponListItem {
  uid: number
  id: number
  name: string
  en_name: string
  profession: string
  level: number
  star: number
  refine_level: number
  max_star: number
  max_refine: number
}

export interface WeaponDetail {
  uid: number
  id: number
  name: string
  en_name: string
  profession: string
  level: number
  star: number
  refine_level: number
  lock: boolean
  max_star: number
  max_refine: number
}

export interface WeaponUpdate {
  level?: number
  star?: number
  refine_level?: number
}

// ─── Equip ─────────────────────────────────────────

export interface EquipListItem {
  uid: number
  id: number
  suit_name: string
  suit_en_name: string
  slot: number
  slot_name: string
  level: number
  star: number
}

export interface EquipDetail {
  uid: number
  id: number
  suit_name: string
  suit_en_name: string
  slot: number
  slot_name: string
  level: number
  exp: number
  star: number
  lock: boolean
  properties: EquipProperty[]
  sub_properties: EquipProperty[]
}

export interface EquipProperty {
  key: number
  key_name: string
  base_value: number
  add_value: number
}

export interface EquipUpdate {
  level: number
  star: number
  properties: EquipProperty[]
  sub_properties: (EquipProperty | null)[]
}

export interface EquipCreate {
  id: number
  level: number
  star: number
  properties: EquipProperty[]
  sub_properties: (EquipProperty | null)[]
}

// ─── Hadal Zone ────────────────────────────────────

export interface HadalZone {
  entrances: HadalEntrance[]
}

export interface HadalEntrance {
  id: number
  zone_id: number
}

export interface HadalZoneUpdate {
  entrances: HadalEntrance[]
}

// ─── Templates ─────────────────────────────────────

export interface AvatarTemplate {
  id: number
  name: string
  rarity: number
  camp_id: number
  camp_name: string
}

export interface WeaponTemplate {
  id: number
  name: string
  rarity: number
  profession: string
  max_star: number
  max_refine: number
}

export interface SuitGroup {
  suit_type: number
  suit_name: string
  suit_en_name: string
  slots: number[]
}

export interface StatOption {
  key: number
  name: string
}

export interface Templates {
  avatars: AvatarTemplate[]
  weapons: WeaponTemplate[]
  profession_names: Record<string, string>
  suit_groups: Record<string, SuitGroup>
  main_stat_options: Record<string, StatOption[]>
  sub_stat_options: StatOption[]
  stat_names: Record<number, string>
  fixed_main_slots: number[]
}

// ─── Debug ─────────────────────────────────────────

export interface DebugListDirResult {
  path: string
  exists: boolean
  is_dir: boolean
  entries: { name: string; is_dir: boolean }[]
}

export interface DebugAvatarIdsResult {
  count: number
  first_result: Record<string, unknown>
}
