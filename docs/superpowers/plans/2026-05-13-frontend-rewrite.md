# Yoshunko Admin 前端重写实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Tauri v2 桌面应用的前端从原生 HTML/CSS/JS 完全重写为 Vue 3 + Naive UI，复现所有功能、UI、动画。

**Architecture:** Vue 3 SFC 组件 + Naive UI 组件库 + Tailwind CSS 4 布局 + @tauri-apps/api IPC。Rust 后端最小改动（新增快速启动命令 + 补全模板数据）。前端通过 `invoke()` 直接调用 Tauri 命令，不再需要 tauri-compat.js shim。

**Tech Stack:** Vue 3.5 + TypeScript + Vite 6 + Naive UI 2.x + Tailwind CSS 4 + @tauri-apps/api 2.x

**设计文档:** `docs/superpowers/specs/2026-05-13-frontend-rewrite-design.md`

**参考代码:** `static/js/` (当前前端) + `src-tauri/src/api.rs` (后端命令)

---

## 文件结构总览

```
yoshunko-admin-rust/
├── package.json                    # 新增
├── tsconfig.json                   # 新增
├── tsconfig.node.json              # 新增
├── vite.config.ts                  # 新增
├── index.html                      # 新增 (Vite 入口 HTML)
├── src/                            # 新增 (Vue 前端源码)
│   ├── main.ts
│   ├── App.vue
│   ├── composables/
│   │   ├── useAppState.ts
│   │   ├── useTheme.ts
│   │   └── useKeyboard.ts
│   ├── lib/
│   │   ├── api.ts
│   │   ├── types.ts
│   │   └── utils.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── TitleBar.vue
│   │   │   ├── Sidebar.vue
│   │   │   └── MainContent.vue
│   │   ├── shared/
│   │   │   ├── GameCard.vue
│   │   │   ├── EditorPage.vue
│   │   │   ├── Stepper.vue
│   │   │   ├── SearchBar.vue
│   │   │   ├── SkillGrid.vue
│   │   │   ├── RankDots.vue
│   │   │   ├── StarRating.vue
│   │   │   └── SkeletonGrid.vue
│   │   └── panels/
│   │       ├── SetupPanel.vue
│   │       ├── AvatarsPanel.vue
│   │       ├── WeaponsPanel.vue
│   │       ├── EquipsPanel.vue
│   │       ├── HadalPanel.vue
│   │       ├── PlayerPanel.vue
│   │       ├── QuickLaunchPanel.vue
│   │       ├── SettingsPanel.vue
│   │       └── ShortcutsPanel.vue
│   ├── assets/
│   │   ├── pinyin-data.ts
│   │   └── icon.png
│   └── styles/
│       └── theme.css
├── src-tauri/                      # 已有 (Rust 后端)
│   ├── tauri.conf.json             # 修改 frontendDist
│   ├── src/
│   │   ├── api.rs                  # 修改: 新增命令 + 补全模板
│   │   └── lib.rs                  # 修改: 注册新命令
│   └── Cargo.toml                  # 修改: 新增依赖
└── static/                         # 已有 (旧前端，迁移完成后删除)
```

---

## Phase 1: 脚手架

### Task 1: 初始化 Vite + Vue 3 + TypeScript 项目

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/vite-env.d.ts`

- [ ] **Step 1: 创建 package.json**

```json
{
  "name": "yoshunko-admin",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.5.13",
    "naive-ui": "^2.40.3",
    "@tauri-apps/api": "^2.2.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2.3",
    "vite": "^6.3.5",
    "vue-tsc": "^2.2.8",
    "typescript": "^5.7.3",
    "@tauri-apps/cli": "^2.2.7",
    "tailwindcss": "^4.1.7",
    "@tailwindcss/vite": "^4.1.7"
  }
}
```

- [ ] **Step 2: 创建 tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": false,
    "noUnusedParameters": false,
    "noFallthroughCasesInSwitch": true,
    "resolveJsonModule": true,
    "esModuleInterop": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 3: 创建 tsconfig.node.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 4: 创建 vite.config.ts**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
```

- [ ] **Step 5: 创建 index.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Yoshunko Admin</title>
  <link rel="icon" href="/src/assets/icon.png" />
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.ts"></script>
</body>
</html>
```

- [ ] **Step 6: 创建 src/main.ts**

```typescript
import { createApp } from 'vue'
import App from './App.vue'
import './styles/theme.css'

createApp(App).mount('#app')
```

- [ ] **Step 7: 创建 src/App.vue (最小占位)**

```vue
<script setup lang="ts">
</script>

<template>
  <div>Yoshunko Admin</div>
</template>
```

- [ ] **Step 8: 创建 src/vite-env.d.ts**

```typescript
/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
```

- [ ] **Step 9: 创建 src/styles/theme.css (最小占位)**

```css
@import "tailwindcss";
```

- [ ] **Step 10: 安装依赖并验证构建**

```bash
cd D:/3.0.1/tools/yoshunko-admin-rust
npm install
npm run build
```

Expected: `vite build` 成功，`dist/` 目录生成。

- [ ] **Step 11: 修改 tauri.conf.json**

将 `frontendDist` 从 `../static` 改为 `../dist`，添加构建命令：

```json
{
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": false
  }
}
```

- [ ] **Step 12: 验证 Tauri dev 模式**

```bash
cd D:/3.0.1/tools/yoshunko-admin-rust
npm run tauri dev
```

Expected: Tauri 窗口打开，显示 "Yoshunko Admin" 文字。

---

## Phase 2: 核心基础设施

### Task 2: TypeScript 类型定义

**Files:**
- Create: `src/lib/types.ts`

参照 `src-tauri/src/api.rs` 的实际返回值定义所有类型。类型定义已在设计文档第 10 节中完整列出。

- [ ] **Step 1: 创建 src/lib/types.ts**

从设计文档第 10 节复制完整的类型定义。包含：
- `Config`
- `PlayerBasic`, `PlayerBasicUpdate`
- `AvatarListItem`, `AvatarDetail`, `SkillTypeLevel`, `AvatarUpdate`
- `WeaponListItem`, `WeaponDetail`, `WeaponUpdate`
- `EquipListItem`, `EquipDetail`, `EquipProperty`, `EquipUpdate`, `EquipCreate`
- `HadalZone`, `HadalEntrance`, `HadalZoneUpdate`
- `AvatarTemplate`, `WeaponTemplate`, `SuitGroup`, `StatOption`, `Templates`
- `DebugListDirResult`, `DebugAvatarIdsResult`

- [ ] **Step 2: 验证类型编译**

```bash
cd D:/3.0.1/tools/yoshunko-admin-rust
npx vue-tsc --noEmit
```

Expected: 无类型错误。

---

### Task 3: IPC API 层

**Files:**
- Create: `src/lib/api.ts`

- [ ] **Step 1: 创建 src/lib/api.ts**

```typescript
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
  updatePlayerBasic: (uid: number, data: PlayerBasicUpdate) => invoke<{ ok: boolean; error?: string }>('update_player_basic', { uid, data }),

  // Avatars
  getAvatars: (uid: number) => invoke<{ avatars: AvatarListItem[] }>('get_avatars', { uid }),
  getAvatar: (uid: number, avatarId: number) => invoke<{ avatar: AvatarDetail; forms: unknown[] } | null>('get_avatar', { uid, avatarId }),
  updateAvatar: (uid: number, avatarId: number, data: AvatarUpdate) => invoke<{ ok: boolean; error?: string }>('update_avatar', { uid, avatarId, data }),

  // Weapons
  getWeapons: (uid: number) => invoke<{ weapons: WeaponListItem[] }>('get_weapons', { uid }),
  getWeapon: (uid: number, weaponUid: number) => invoke<WeaponDetail | null>('get_weapon', { uid, weaponUid }),
  updateWeapon: (uid: number, weaponUid: number, data: WeaponUpdate) => invoke<{ ok: boolean; error?: string }>('update_weapon', { uid, weaponUid, data }),

  // Equips
  getEquips: (uid: number) => invoke<{ equips: EquipListItem[] }>('get_equips', { uid }),
  getEquip: (uid: number, equipUid: number) => invoke<EquipDetail | null>('get_equip', { uid, equipUid }),
  updateEquip: (uid: number, equipUid: number, data: EquipUpdate) => invoke<{ ok: boolean; error?: string }>('update_equip', { uid, equipUid, data }),
  createEquip: (uid: number, data: EquipCreate) => invoke<{ ok: boolean; uid?: number; error?: string }>('create_equip', { uid, data }),
  deleteEquip: (uid: number, equipUid: number) => invoke<{ ok: boolean; error?: string }>('delete_equip', { uid, equipUid }),

  // Hadal Zone
  getHadalZone: (uid: number) => invoke<HadalZone | null>('get_hadal_zone', { uid }),
  updateHadalZone: (uid: number, data: HadalZoneUpdate) => invoke<{ ok: boolean; error?: string }>('update_hadal_zone', { uid, data }),
}
```

- [ ] **Step 2: 创建 src/lib/utils.ts**

```typescript
import { ref } from 'vue'

/** HTML 转义 */
export function escHtml(s: string): string {
  const div = document.createElement('div')
  div.textContent = s
  return div.innerHTML
}

/** Toast 通知 */
export interface ToastItem {
  id: number
  message: string
  type: 'success' | 'error' | 'info'
}

export const toasts = ref<ToastItem[]>([])
let toastId = 0

export function toast(message: string, type: ToastItem['type'] = 'info') {
  const id = ++toastId
  toasts.value.push({ id, message, type })
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 3000)
}

export function removeToast(id: number) {
  toasts.value = toasts.value.filter(t => t.id !== id)
}

/** 确认对话框 */
export interface ConfirmState {
  visible: boolean
  message: string
  onConfirm: (() => void) | null
}

export const confirmState = ref<ConfirmState>({
  visible: false,
  message: '',
  onConfirm: null,
})

export function showConfirm(message: string, onConfirm: () => void) {
  confirmState.value = { visible: true, message, onConfirm }
}

export function closeConfirm() {
  confirmState.value = { ...confirmState.value, visible: false, onConfirm: null }
}
```

- [ ] **Step 3: 验证编译**

```bash
npx vue-tsc --noEmit
```

---

### Task 4: 全局状态 Composable

**Files:**
- Create: `src/composables/useAppState.ts`

- [ ] **Step 1: 创建 src/composables/useAppState.ts**

```typescript
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
export const cacheDirty = ref(false)

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

// Dirty
export const dirty = ref(false)

// ─── 模板 Map ──────────────────────────────────────

export const avatarMap = computed(() => {
  const map = new Map<number, { name: string; en_name: string; rarity: number; camp_id: number; camp_name: string; profession: string }>()
  if (!templates.value) return map
  const t = templates.value
  for (const a of t.avatars) {
    map.set(a.id, {
      name: a.name,
      en_name: '',
      rarity: a.rarity,
      camp_id: a.camp_id,
      camp_name: a.camp_name,
      profession: '',
    })
  }
  return map
})

export const weaponMap = computed(() => {
  const map = new Map<number, { name: string; en_name: string; rarity: number; profession: string; max_star: number; max_refine: number }>()
  if (!templates.value) return map
  for (const w of templates.value.weapons) {
    map.set(w.id, {
      name: w.name,
      en_name: '',
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

export function avatarEnName(id: number): string {
  return avatarMap.value.get(id)?.en_name || ''
}

export function avatarRarity(id: number): number {
  return avatarMap.value.get(id)?.rarity || 0
}

export function avatarCamp(id: number): string {
  return avatarMap.value.get(id)?.camp_name || ''
}

export function avatarProfession(id: number): string {
  return avatarMap.value.get(id)?.profession || ''
}

export function weaponName(id: number): string {
  return weaponMap.value.get(id)?.name || `#${id}`
}

export function weaponEnName(id: number): string {
  return weaponMap.value.get(id)?.en_name || ''
}

export function weaponProfession(id: number): string {
  return weaponMap.value.get(id)?.profession || ''
}

export function markDirty() { dirty.value = true }
export function markClean() { dirty.value = false }
export function markCacheDirty() { cacheDirty.value = true }
```

- [ ] **Step 2: 验证编译**

```bash
npx vue-tsc --noEmit
```

---

## Phase 3: 布局

### Task 5: App.vue + TitleBar + Sidebar + MainContent

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/layout/TitleBar.vue`
- Create: `src/components/layout/Sidebar.vue`
- Create: `src/components/layout/MainContent.vue`

- [ ] **Step 1: 创建 TitleBar.vue**

```vue
<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

function minimize() { appWindow.minimize() }
function toggleMax() { appWindow.toggleMaximize() }
function close() { appWindow.close() }
</script>

<template>
  <div data-tauri-drag-region class="title-bar">
    <div class="title-bar__title" data-tauri-drag-region>
      <span class="title-bar__brand">Yoshunko Admin</span>
    </div>
    <div class="title-bar__controls">
      <button class="tb-btn" aria-label="最小化" @click="minimize">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="2" fill="currentColor"/></svg>
      </button>
      <button class="tb-btn" aria-label="最大化" @click="toggleMax">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
      </button>
      <button class="tb-btn tb-btn--close" aria-label="关闭" @click="close">
        <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 创建 Sidebar.vue**

```vue
<script setup lang="ts">
import { uid, panel, avatarCache, weaponCache, equipCache, dirty } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { ref, onMounted } from 'vue'

const emit = defineEmits<{
  (e: 'panel-change', p: string): void
}>()

const players = ref<number[]>([])
const avatarCount = ref(0)
const weaponCount = ref(0)
const equipCount = ref(0)

const navItems = [
  { key: 'avatars', label: '角色管理', icon: 'avatar' },
  { key: 'weapons', label: '音擎仓库', icon: 'weapon' },
  { key: 'equips', label: '驱动盘仓库', icon: 'equip' },
  { key: 'hadal_zone', label: '式舆防卫战', icon: 'hadal' },
  { key: 'player_info', label: '玩家信息', icon: 'player' },
  { key: 'quick_launch', label: '快速启动', icon: 'launch' },
]

function selectPanel(key: string) {
  panel.value = key
  emit('panel-change', key)
}

function onPlayerChange(e: Event) {
  const val = parseInt((e.target as HTMLSelectElement).value)
  uid.value = isNaN(val) ? null : val
}

onMounted(async () => {
  try {
    const data = await api.getPlayerList()
    players.value = data.players
    if (data.players.length > 0) {
      uid.value = data.players[0]
    }
  } catch (e) {
    console.error('Failed to load players:', e)
  }
})

function updateCounts() {
  avatarCount.value = avatarCache.value.length
  weaponCount.value = weaponCache.value.length
  equipCount.value = equipCache.value.length
}

defineExpose({ updateCounts })
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-brand">
      <img src="@/assets/icon.png" alt="Logo" class="sidebar-brand__logo" />
      <div>
        <h1 class="sidebar-brand__title">Yoshunko Admin</h1>
        <p class="sidebar-brand__sub">Game Data Manager</p>
      </div>
    </div>

    <div class="sidebar-player">
      <select class="form-select" :value="uid" @change="onPlayerChange">
        <option value="">-- 选择玩家 --</option>
        <option v-for="pid in players" :key="pid" :value="pid">玩家 UID: {{ pid }}</option>
      </select>
    </div>

    <nav class="sidebar-nav" role="navigation" aria-label="功能导航">
      <div role="tablist" aria-label="功能面板">
        <div
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: panel === item.key, dirty: dirty && panel === item.key }"
          role="tab"
          :aria-selected="panel === item.key"
          @click="selectPanel(item.key)"
        >
          <span class="nav-icon" :class="'nav-icon--' + item.icon"></span>
          <span class="nav-item__label">{{ item.label }}</span>
          <span v-if="item.key === 'avatars'" class="nav-badge">{{ avatarCount }}</span>
          <span v-if="item.key === 'weapons'" class="nav-badge">{{ weaponCount }}</span>
          <span v-if="item.key === 'equips'" class="nav-badge">{{ equipCount }}</span>
        </div>
      </div>
    </nav>

    <div class="sidebar-footer">
      <button class="sidebar-settings-btn" aria-label="设置" @click="selectPanel('settings')">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83"/>
        </svg>
      </button>
    </div>
  </aside>
</template>
```

- [ ] **Step 3: 创建 MainContent.vue (占位)**

```vue
<script setup lang="ts">
import { panel, uid } from '@/composables/useAppState'
</script>

<template>
  <main class="main-content" role="main" aria-label="主内容区">
    <div v-if="!uid" class="empty-state">
      <p>选择一个玩家开始管理游戏数据</p>
    </div>
    <div v-else class="panel-content">
      <p>面板: {{ panel }}</p>
    </div>
  </main>
</template>
```

- [ ] **Step 4: 更新 App.vue**

```vue
<script setup lang="ts">
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
</script>

<template>
  <div class="app-layout">
    <TitleBar />
    <div class="app-body">
      <Sidebar />
      <MainContent />
    </div>
  </div>
</template>
```

- [ ] **Step 5: 验证布局渲染**

```bash
npm run tauri dev
```

Expected: 窗口显示标题栏 + 侧栏 + 内容区布局。

---

## Phase 4: 共享组件

### Task 6: Stepper + SearchBar + GameCard + SkeletonGrid

**Files:**
- Create: `src/components/shared/Stepper.vue`
- Create: `src/components/shared/SearchBar.vue`
- Create: `src/components/shared/GameCard.vue`
- Create: `src/components/shared/SkeletonGrid.vue`

- [ ] **Step 1: 创建 Stepper.vue**

```vue
<script setup lang="ts">
const props = defineProps<{
  modelValue: number
  min: number
  max: number
  label?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: number): void
}>()

function step(delta: number) {
  const next = props.modelValue + delta
  if (next >= props.min && next <= props.max) {
    emit('update:modelValue', next)
  }
}

function onInput(e: Event) {
  let v = parseInt((e.target as HTMLInputElement).value) || 0
  if (v < props.min) v = props.min
  if (v > props.max) v = props.max
  emit('update:modelValue', v)
}
</script>

<template>
  <div class="input-stepper">
    <button class="stepper-btn" :disabled="modelValue <= min" @click="step(-1)" :aria-label="'减少' + (label || '')">−</button>
    <input type="number" class="stepper-input" :value="modelValue" :min="min" :max="max" @change="onInput" />
    <button class="stepper-btn" :disabled="modelValue >= max" @click="step(1)" :aria-label="'增加' + (label || '')">+</button>
  </div>
</template>
```

- [ ] **Step 2: 创建 SearchBar.vue**

```vue
<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  modelValue: string
  placeholder?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void
}>()

const isComposing = ref(false)

function onInput(e: Event) {
  if (isComposing.value) return
  emit('update:modelValue', (e.target as HTMLInputElement).value)
}

function onCompositionStart() { isComposing.value = true }
function onCompositionEnd(e: CompositionEvent) {
  isComposing.value = false
  emit('update:modelValue', (e.target as HTMLInputElement).value)
}

function clear() {
  emit('update:modelValue', '')
}
</script>

<template>
  <div class="search-wrap">
    <input
      type="text"
      class="search-input"
      :value="modelValue"
      :placeholder="placeholder || '搜索...'"
      @input="onInput"
      @compositionstart="onCompositionStart"
      @compositionend="onCompositionEnd"
    />
    <button v-if="modelValue" class="search-clear" aria-label="清除搜索" @click="clear">×</button>
  </div>
</template>
```

- [ ] **Step 3: 创建 GameCard.vue**

```vue
<script setup lang="ts">
defineProps<{
  rarity?: 's' | 'a' | 'b'
  title: string
  subtitle?: string
  tags?: string[]
  level?: number
}>()

const emit = defineEmits<{
  (e: 'click'): void
}>()

function onPress(el: Event) {
  const target = el.currentTarget as HTMLElement
  target.style.transform = 'scale(0.92)'
  setTimeout(() => { target.style.transform = '' }, 120)
}
</script>

<template>
  <div
    class="game-card"
    :class="{
      'rarity-s-card': rarity === 's',
      'rarity-a-card': rarity === 'a',
    }"
    tabindex="0"
    role="button"
    @click="onPress($event); emit('click')"
    @keydown.enter="emit('click')"
    @keydown.space.prevent="emit('click')"
  >
    <div class="card-header">
      <span v-if="rarity" class="game-card__rarity" :class="'rarity-' + rarity">{{ rarity.toUpperCase() }}</span>
      <span v-if="level !== undefined" class="game-card__level">Lv.{{ level }}</span>
    </div>
    <div class="card-title">{{ title }}</div>
    <div v-if="subtitle" class="card-subtitle">{{ subtitle }}</div>
    <div v-if="tags && tags.length" class="card-tags">
      <span v-for="tag in tags" :key="tag" class="card-tag">{{ tag }}</span>
    </div>
  </div>
</template>
```

- [ ] **Step 4: 创建 SkeletonGrid.vue**

```vue
<script setup lang="ts">
defineProps<{
  count?: number
}>()
</script>

<template>
  <div class="skeleton-wrap">
    <div class="skeleton skeleton--title"></div>
    <div class="skeleton-grid">
      <div v-for="i in (count || 6)" :key="i" class="skeleton skeleton--card"></div>
    </div>
  </div>
</template>
```

- [ ] **Step 5: 创建 EditorPage.vue**

```vue
<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'save'): void
}>()
</script>

<template>
  <div class="editor-page">
    <div class="editor-page__top">
      <a class="editor-back" @click.prevent="emit('back')">← 返回</a>
      <div class="editor-page__header">
        <h2 class="editor-title">{{ title }}</h2>
        <span v-if="subtitle" class="editor-subtitle">{{ subtitle }}</span>
      </div>
    </div>
    <div class="editor-page__body">
      <slot />
    </div>
    <div class="editor-page__actions">
      <button class="btn btn-primary" @click="emit('save')">保存</button>
    </div>
  </div>
</template>
```

- [ ] **Step 6: 验证编译**

```bash
npx vue-tsc --noEmit
```

---

## Phase 5: 面板迁移

### Task 7: SetupPanel

**Files:**
- Create: `src/components/panels/SetupPanel.vue`
- Modify: `src/components/layout/MainContent.vue`

- [ ] **Step 1: 创建 SetupPanel.vue**

参照 `static/js/panels/player.js` L77-128 的 `renderSetup()` 和 `onSetupConnect()` 实现。

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'

const stateDir = ref('')
const candidates = ref<string[]>([])
const version = ref('')
const loading = ref(false)

const emit = defineEmits<{
  (e: 'connected'): void
}>()

onMounted(async () => {
  try {
    const v = await api.getVersion()
    version.value = v.version
  } catch {}
  try {
    const r = await api.autoDetectPaths()
    candidates.value = r.candidates || []
    if (candidates.value.length > 0) {
      stateDir.value = candidates.value[0]
    }
  } catch {}
})

function selectCandidate(path: string) {
  stateDir.value = path
}

async function connect() {
  if (!stateDir.value.trim()) {
    toast('请输入状态目录路径', 'error')
    return
  }
  loading.value = true
  try {
    const r = await api.setStateDir(stateDir.value.trim())
    if (r.ok) {
      toast('连接成功', 'success')
      emit('connected')
    } else {
      toast(r.error || '连接失败', 'error')
    }
  } catch (e: any) {
    toast(e.message || '连接失败', 'error')
  } finally {
    loading.value = false
  }
}

async function paste() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) stateDir.value = text.trim()
  } catch {
    toast('无法读取剪贴板', 'error')
  }
}
</script>

<template>
  <div class="setup-page">
    <div class="setup-card">
      <div class="setup-brand">
        <img src="@/assets/icon.png" alt="Logo" class="setup-logo" />
        <h1>Yoshunko Admin</h1>
        <p class="text-muted">{{ version }}</p>
      </div>

      <div class="setup-form">
        <label class="form-label">状态目录路径</label>
        <div class="setup-input-row">
          <input v-model="stateDir" class="form-input" placeholder="例如: D:\3.0.1\state" />
          <button class="btn btn-ghost" @click="paste">粘贴</button>
        </div>

        <div v-if="candidates.length" class="setup-candidates">
          <p class="text-sm text-muted mb-1 mt-2">检测到的路径（点击填入）：</p>
          <div
            v-for="c in candidates"
            :key="c"
            class="candidate-path"
            @click="selectCandidate(c)"
          >{{ c }}</div>
        </div>

        <button class="btn btn-primary mt-3" :disabled="loading" @click="connect">
          {{ loading ? '连接中...' : '连接' }}
        </button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 更新 MainContent.vue 集成 SetupPanel**

更新 MainContent，在未配置时显示 SetupPanel：

```vue
<script setup lang="ts">
import { panel, uid, templates, cacheDirty, avatarCache, weaponCache, equipCache } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { ref, onMounted, watch } from 'vue'
import SetupPanel from '@/components/panels/SetupPanel.vue'
// 后续面板组件在此导入

const configured = ref(false)
const loading = ref(true)

async function checkConfig() {
  try {
    const cfg = await api.getConfig()
    if (cfg.configured && cfg.config_exists) {
      configured.value = true
      templates.value = await api.getTemplates()
    }
  } catch {}
  loading.value = false
}

async function loadCounts() {
  if (!uid.value) return
  try {
    const [av, wp, eq] = await Promise.all([
      api.getAvatars(uid.value),
      api.getWeapons(uid.value),
      api.getEquips(uid.value),
    ])
    avatarCache.value = av.avatars
    weaponCache.value = wp.weapons
    equipCache.value = eq.equips
    cacheDirty.value = false
  } catch {}
}

onMounted(checkConfig)

watch(uid, async () => {
  if (uid.value && configured.value) {
    await loadCounts()
  }
})

function onConnected() {
  configured.value = true
  checkConfig()
}
</script>

<template>
  <main class="main-content" role="main" aria-label="主内容区">
    <div v-if="loading" class="loading-wrap"><div class="spinner"></div></div>
    <SetupPanel v-else-if="!configured" @connected="onConnected" />
    <div v-else-if="!uid" class="empty-state">
      <p>选择一个玩家开始管理游戏数据</p>
    </div>
    <div v-else class="panel-content">
      <!-- 面板内容由后续 Task 添加 -->
      <p>面板: {{ panel }}</p>
    </div>
  </main>
</template>
```

- [ ] **Step 3: 验证 Setup 流程**

```bash
npm run tauri dev
```

Expected: 显示设置页面，可输入路径并连接。

---

### Task 8: AvatarsPanel — 画廊视图

**Files:**
- Create: `src/components/panels/AvatarsPanel.vue`
- Create: `src/assets/pinyin-data.ts`

- [ ] **Step 1: 迁移拼音数据**

将 `static/js/pinyin-data.js` 的内容迁移为 TypeScript 模块 `src/assets/pinyin-data.ts`。数据量大，直接复制并添加类型声明。

- [ ] **Step 2: 创建 AvatarsPanel.vue 画廊部分**

参照 `static/js/panels/avatars.js` 的 `renderAvatars()` 和 `avatarCardHTML()` 实现。

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { uid, avatarCache, cacheDirty, avatarGroupBy, searchQuery, selectedAvatarId, avatarView, templates } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { AVATAR_PINYIN } from '@/assets/pinyin-data'
import SearchBar from '@/components/shared/SearchBar.vue'
import GameCard from '@/components/shared/GameCard.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'

const loading = ref(true)

const PROFESSION_ORDER = ['强攻', '击破', '异常', '支援', '防护', '命破']

const filteredAvatars = computed(() => {
  let list = avatarCache.value
  const q = searchQuery.avatars.toLowerCase()
  if (q) {
    list = list.filter(a => {
      const py = AVATAR_PINYIN[a.avatar_id]
      if (py && (py.full.includes(q) || py.initials.includes(q))) return true
      if (String(a.avatar_id).includes(q)) return true
      if (a.name.toLowerCase().includes(q)) return true
      if (a.en_name.toLowerCase().includes(q)) return true
      if (a.profession.toLowerCase().includes(q)) return true
      return false
    })
  }
  return list
})

const groupedAvatars = computed(() => {
  const groups = new Map<string, typeof filteredAvatars.value>()
  for (const a of filteredAvatars.value) {
    const key = avatarGroupBy.value === 'camp'
      ? (templates.value?.avatars.find(t => t.id === a.avatar_id)?.camp_name || '未知')
      : a.profession
    if (!groups.has(key)) groups.set(key, [])
    groups.get(key)!.push(a)
  }
  // Sort by profession order if grouping by profession
  if (avatarGroupBy.value === 'profession') {
    const sorted = new Map<string, typeof filteredAvatars.value>()
    for (const p of PROFESSION_ORDER) {
      if (groups.has(p)) sorted.set(p, groups.get(p)!)
    }
    for (const [k, v] of groups) {
      if (!sorted.has(k)) sorted.set(k, v)
    }
    return sorted
  }
  return groups
})

function rarityClass(rarity: number): 's' | 'a' | 'b' | undefined {
  if (rarity >= 4) return 's'
  if (rarity >= 3) return 'a'
  return undefined
}

function selectAvatar(id: number) {
  selectedAvatarId.value = id
  avatarView.value = 'editor'
}

onMounted(async () => {
  if (!uid.value) return
  if (avatarCache.value.length && !cacheDirty.value) {
    loading.value = false
    return
  }
  try {
    const data = await api.getAvatars(uid.value)
    avatarCache.value = data.avatars
  } catch (e: any) {
    toast('加载角色失败: ' + e.message, 'error')
  }
  loading.value = false
})
</script>

<template>
  <div>
    <div class="search-bar-row">
      <SearchBar v-model="searchQuery.avatars" placeholder="搜索角色（支持拼音）..." />
      <button class="btn btn-ghost search-group-toggle" @click="avatarGroupBy = avatarGroupBy === 'camp' ? 'profession' : 'camp'">
        {{ avatarGroupBy === 'camp' ? '按职业' : '按阵营' }}
      </button>
    </div>

    <SkeletonGrid v-if="loading" />

    <div v-else>
      <div v-for="[group, avatars] in groupedAvatars" :key="group" class="avatar-gallery__camp-section">
        <h3 class="camp-section-header">{{ group }}</h3>
        <div class="avatar-gallery__grid">
          <GameCard
            v-for="a in avatars"
            :key="a.avatar_id"
            :rarity="rarityClass(a.rarity)"
            :title="a.name"
            :subtitle="a.en_name"
            :level="a.level"
            :tags="[a.profession]"
            @click="selectAvatar(a.avatar_id)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 3: 验证画廊渲染**

```bash
npm run tauri dev
```

Expected: 角色画廊显示，卡片按阵营分组，搜索可用。

---

### Task 9: AvatarsPanel — 编辑器视图

**Files:**
- Modify: `src/components/panels/AvatarsPanel.vue`
- Create: `src/components/shared/SkillGrid.vue`
- Create: `src/components/shared/RankDots.vue`

参照 `static/js/panels/avatars.js` 的 `avatarEditorHTML()` 和 `saveAvatar()` 实现。

- [ ] **Step 1: 创建 RankDots.vue**

```vue
<script setup lang="ts">
defineProps<{
  rank: number
  max?: number
}>()
</script>

<template>
  <div class="rank-dots">
    <span v-for="i in (max || 6)" :key="i" class="rank-dot" :class="{ active: i <= rank }"></span>
  </div>
</template>
```

- [ ] **Step 2: 创建 SkillGrid.vue**

```vue
<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  skills: { type: string; name: string; level: number; max: number }[]
}>()

const emit = defineEmits<{
  (e: 'update', type: string, level: number): void
}>()

function onInput(type: string, e: Event) {
  let v = parseInt((e.target as HTMLInputElement).value) || 0
  const skill = props.skills.find(s => s.type === type)
  if (skill) {
    if (v < 0) v = 0
    if (v > skill.max) v = skill.max
  }
  emit('update', type, v)
}
</script>

<template>
  <div class="skill-grid">
    <div v-for="skill in skills" :key="skill.type" class="skill-card">
      <div class="skill-name">{{ skill.name }}</div>
      <input
        type="number"
        class="skill-input"
        :value="skill.level"
        :min="0"
        :max="skill.max"
        @change="onInput(skill.type, $event)"
      />
      <div class="skill-hint">最大 {{ skill.max }}</div>
    </div>
  </div>
</template>
```

- [ ] **Step 3: 在 AvatarsPanel.vue 中添加编辑器视图**

在 `<template>` 中添加编辑器部分，在 `<script setup>` 中添加编辑器逻辑。参照 `avatars.js` L150-280 的编辑器实现，包含：
- 等级步进器 (1-60)
- 天赋步进器 (0-6)
- 命座自动计算 (`unlocked_talent_num`)
- 技能网格（6 个技能 + 核心被动）
- 觉醒等级步进器 (0-6)
- 武器 UID 输入
- 皮肤 ID 步进器
- 保存逻辑

- [ ] **Step 4: 验证编辑器**

```bash
npm run tauri dev
```

Expected: 点击角色卡片进入编辑器，显示所有字段，可修改并保存。

---

### Task 10: WeaponsPanel

**Files:**
- Create: `src/components/panels/WeaponsPanel.vue`
- Create: `src/components/shared/StarRating.vue`

参照 `static/js/panels/weapons.js` 实现。

- [ ] **Step 1: 创建 StarRating.vue**

```vue
<script setup lang="ts">
defineProps<{
  star: number
  max?: number
}>()
</script>

<template>
  <div class="star-rating">
    <span v-for="i in (max || 5)" :key="i" class="star" :class="{ active: i <= star }">★</span>
  </div>
</template>
```

- [ ] **Step 2: 创建 WeaponsPanel.vue**

包含画廊视图（按职业分组）和编辑器视图（等级 + 精炼步进器）。参照 `weapons.js` 的 `renderWeapons()`、`weaponCardHTML()`、`weaponEditorHTML()`、`saveWeapon()` 实现。

- [ ] **Step 3: 验证**

---

### Task 11: EquipsPanel — 画廊 + 编辑器

**Files:**
- Create: `src/components/panels/EquipsPanel.vue`

参照 `static/js/panels/equips.js` 实现。最复杂的面板，包含：
- 画廊（按套装分组）
- 创建流程（3 步 Modal）
- 编辑器（主词条 + 副词条 + 强化校验）

- [ ] **Step 1: 创建 EquipsPanel.vue 画廊**

- [ ] **Step 2: 添加创建流程 Modal**

- [ ] **Step 3: 添加编辑器视图**

- [ ] **Step 4: 验证**

---

### Task 12: HadalPanel

**Files:**
- Create: `src/components/panels/HadalPanel.vue`

参照 `static/js/panels/hadal.js` 实现。

- [ ] **Step 1: 创建 HadalPanel.vue**

- [ ] **Step 2: 验证**

---

### Task 13: PlayerPanel

**Files:**
- Create: `src/components/panels/PlayerPanel.vue`

参照 `static/js/panels/player.js` 的 `renderPlayerInfo()` 和 `savePlayerInfo()` 实现。导出/导入功能通过组合现有 IPC 命令实现。

- [ ] **Step 1: 创建 PlayerPanel.vue**

- [ ] **Step 2: 验证**

---

### Task 14: QuickLaunchPanel

**Files:**
- Create: `src/components/panels/QuickLaunchPanel.vue`

需要后端新增命令支持。先创建 UI，后端命令在 Phase 6 实现。

- [ ] **Step 1: 创建 QuickLaunchPanel.vue**

- [ ] **Step 2: 验证（预期：部分功能不可用直到后端命令就绪）**

---

### Task 15: SettingsPanel + ShortcutsPanel

**Files:**
- Create: `src/components/panels/SettingsPanel.vue`
- Create: `src/components/panels/ShortcutsPanel.vue`

- [ ] **Step 1: 创建 SettingsPanel.vue**

- [ ] **Step 2: 创建 ShortcutsPanel.vue**

- [ ] **Step 3: 验证**

---

## Phase 6: 后端改动

### Task 16: 补全 get_templates 返回数据

**Files:**
- Modify: `src-tauri/src/api.rs` L170-192
- Modify: `src-tauri/src/template_loader.rs`

- [ ] **Step 1: 在 template_loader.rs 中添加缺失数据加载**

补全 `suit_groups`、`main_stat_options`、`sub_stat_options`、`stat_names` 的数据加载逻辑。参照 `data/templates/EquipmentSuitTemplateTb.json` 和 `data/templates/EquipmentTemplateTb.json`。

- [ ] **Step 2: 在 api.rs 的 get_templates 中补全返回值**

将空的 `suit_groups`、`main_stat_options`、`sub_stat_options`、`stat_names` 替换为实际数据。

- [ ] **Step 3: 编译验证**

```bash
cd D:/3.0.1/tools/yoshunko-admin-rust/src-tauri
cargo build
```

---

### Task 17: 新增快速启动命令

**Files:**
- Modify: `src-tauri/src/api.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 api.rs 中新增 5 个命令**

```rust
#[tauri::command]
pub fn get_launch_config(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    json!(config.get("launch").cloned().unwrap_or(json!({})))
}

#[tauri::command]
pub fn set_launch_path(state: State<AppState>, key: String, path: String) -> Value {
    // 读取 config.json，更新 launch[key]，写回
    // ...
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_program(path: String) -> Value {
    std::process::Command::new(&path).spawn().map_err(|e| e.to_string())?;
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_program_admin(path: String) -> Value {
    // Windows: ShellExecuteW with "runas"
    // ...
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_yoshunko(state: State<AppState>) -> Value {
    // 从 config 读取 WSL distro，执行 wsl -d <distro> -e bash -c "..."
    // ...
    json!({"ok": true})
}
```

- [ ] **Step 2: 在 lib.rs 的 invoke_handler 中注册新命令**

- [ ] **Step 3: 编译验证**

```bash
cargo build
```

---

## Phase 7: 主题 + 动画 + 快捷键

### Task 18: 主题系统

**Files:**
- Create: `src/composables/useTheme.ts`
- Modify: `src/styles/theme.css`

- [ ] **Step 1: 创建 useTheme.ts**

实现亮/暗主题切换 + overlay fade 过渡动画。参照 `static/js/app.js` L410-435 的 `revealTheme()` 和 `toggleTheme()`。

- [ ] **Step 2: 在 theme.css 中添加完整 CSS 变量**

从 `static/css/app.css` 迁移所有 CSS 自定义属性和暗色主题变量。

- [ ] **Step 3: 验证主题切换**

---

### Task 19: 动画系统

**Files:**
- Modify: `src/styles/theme.css`

- [ ] **Step 1: 迁移所有 keyframes 和动画类**

从 `static/css/app.css` 迁移：
- `editorSlideIn`、`suitPopIn`、`modalIn`、`slideUp`、`fadeIn`、`toastIn`、`themeFadeOut`、`shimmer`、`dirtyPulse`、`spin`
- 所有 `.skeleton`、`.toast`、`.modal`、`.confirm` 相关样式

- [ ] **Step 2: 在各面板组件中添加 Transition/TransitionGroup**

- [ ] **Step 3: 验证所有动画**

---

### Task 20: 键盘快捷键

**Files:**
- Create: `src/composables/useKeyboard.ts`

- [ ] **Step 1: 创建 useKeyboard.ts**

实现全局快捷键：1-7 面板切换、Ctrl+S 保存、Ctrl+Z 撤销、Ctrl+F 搜索、ESC 返回。

- [ ] **Step 2: 在 App.vue 中集成**

- [ ] **Step 3: 验证快捷键**

---

## Phase 8: 集成验证

### Task 21: 全功能验证

- [ ] **Step 1: 逐面板对比 Python 版功能**

对照 `yoshunko-admin-python` 的每个面板，确认所有功能已复现。

- [ ] **Step 2: 动画对比**

确认所有动画效果与原版一致。

- [ ] **Step 3: 主题切换验证**

亮/暗主题切换 + overlay fade 效果。

- [ ] **Step 4: 构建发布版本**

```bash
npm run tauri build
```

- [ ] **Step 5: 更新 CHANGELOG.md 和版本号**

- [ ] **Step 6: 清理旧 static/ 目录（确认迁移完成后）**
