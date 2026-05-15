import { invoke } from '@tauri-apps/api/core'
import type {
  Config, PlayerBasic, PlayerBasicUpdate,
  AvatarListItem, AvatarDetail, AvatarUpdate,
  WeaponListItem, WeaponDetail, WeaponUpdate,
  EquipListItem, EquipDetail, EquipUpdate, EquipCreate,
  HadalZone, HadalZoneUpdate,
  Templates,
  DebugListDirResult, DebugAvatarIdsResult,
} from './types'

export const api = {
  // Config
  getConfig: () => invoke<Config>('get_config'),
  getVersion: () => invoke<{ version: string }>('get_version'),
  setStateDir: (path: string) => invoke<{ ok: boolean; error?: string }>('set_state_dir', { path }),
  autoDetectPaths: () => invoke<{ candidates: string[] }>('auto_detect_paths'),

  // Debug
  debugListDir: (path: string) => invoke<DebugListDirResult>('debug_list_dir', { path }),
  debugAvatarIds: (uid: number) => invoke<DebugAvatarIdsResult>('debug_avatar_ids', { uid }),

  // Templates
  getTemplates: () => invoke<Templates>('get_templates'),

  // Players
  getPlayerList: () => invoke<{ players: number[] }>('get_player_list'),
  getPlayerBasic: (uid: number) => invoke<PlayerBasic | null>('get_player_basic', { uid }),
  updatePlayerBasic: (uid: number, data: PlayerBasicUpdate) =>
    invoke<{ ok: boolean; error?: string }>('update_player_basic', { uid, data }),

  // Avatars
  getAvatars: (uid: number) => invoke<{ avatars: AvatarListItem[] }>('get_avatars', { uid }),
  getAvatar: (uid: number, avatarId: number) =>
    invoke<{ avatar: AvatarDetail; forms: unknown[] } | null>('get_avatar', { uid, avatarId }),
  updateAvatar: (uid: number, avatarId: number, data: AvatarUpdate) =>
    invoke<{ ok: boolean; error?: string }>('update_avatar', { uid, avatarId, data }),

  // Weapons
  getWeapons: (uid: number) => invoke<{ weapons: WeaponListItem[] }>('get_weapons', { uid }),
  getWeapon: (uid: number, weaponUid: number) => invoke<WeaponDetail | null>('get_weapon', { uid, weaponUid }),
  updateWeapon: (uid: number, weaponUid: number, data: WeaponUpdate) =>
    invoke<{ ok: boolean; error?: string }>('update_weapon', { uid, weaponUid, data }),
  deleteWeapon: (uid: number, weaponUid: number) =>
    invoke<{ ok: boolean; error?: string }>('delete_weapon', { uid, weaponUid }),
  copyWeapon: (uid: number, weaponUid: number) =>
    invoke<{ ok: boolean; uid?: number; error?: string }>('copy_weapon', { uid, weaponUid }),

  // Equips
  getEquips: (uid: number) => invoke<{ equips: EquipListItem[] }>('get_equips', { uid }),
  getEquip: (uid: number, equipUid: number) => invoke<EquipDetail | null>('get_equip', { uid, equipUid }),
  updateEquip: (uid: number, equipUid: number, data: EquipUpdate) =>
    invoke<{ ok: boolean; error?: string }>('update_equip', { uid, equipUid, data }),
  createEquip: (uid: number, data: EquipCreate) =>
    invoke<{ ok: boolean; uid?: number; error?: string }>('create_equip', { uid, data }),
  deleteEquip: (uid: number, equipUid: number) =>
    invoke<{ ok: boolean; error?: string }>('delete_equip', { uid, equipUid }),
  copyEquip: (uid: number, equipUid: number) =>
    invoke<{ ok: boolean; uid?: number; error?: string }>('copy_equip', { uid, equipUid }),

  // Hadal Zone
  getHadalZone: (uid: number) => invoke<HadalZone | null>('get_hadal_zone', { uid }),
  updateHadalZone: (uid: number, data: HadalZoneUpdate) =>
    invoke<{ ok: boolean; error?: string }>('update_hadal_zone', { uid, data }),

  // Quick Launch
  getLaunchConfig: () => invoke<{ config: Record<string, string> }>('get_launch_config'),
  setLaunchPath: (key: string, path: string) =>
    invoke<{ ok: boolean; error?: string }>('set_launch_path', { key, path }),
  launchProgram: (key: string) => invoke<{ ok: boolean; error?: string }>('launch_program', { key }),
  launchProgramAdmin: (path: string) => invoke<{ ok: boolean; error?: string }>('launch_program_admin', { path }),
  launchYoshunko: () => invoke<{ ok: boolean; error?: string }>('launch_yoshunko'),
}
