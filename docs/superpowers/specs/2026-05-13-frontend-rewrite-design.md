# Yoshunko Admin 前端重写设计文档

> 将 Tauri v2 桌面应用的前端从原生 HTML/CSS/JS 重写为 Vue 3 + Naive UI。
> 后端需最小改动以补全缺失功能（快速启动、导出/导入、模板数据）。

---

## 1. 目标

- 完全复现 Python 版和当前 Rust 版的所有功能、UI、动画
- 前端代码从命令式 DOM 操作迁移到声明式组件架构
- 后端最小改动：仅新增/补全缺失的 IPC 命令，不重构现有代码
- 保持或超越当前的视觉质量和动画流畅度

## 2. 技术栈

| 层 | 选择 | 版本 | 理由 |
|---|---|---|---|
| 构建 | Vite | 6.x | Tauri v2 官方推荐，HMR 极快 |
| 框架 | Vue 3 + TypeScript | 3.5+ | 响应式系统适合 CRUD 密集型应用，模板语法直观 |
| UI 组件 | Naive UI | 2.x | 专为桌面端设计，内置虚拟滚动、数据表格、对话框 |
| 状态 | Vue 原生响应式 | - | composable 模式，零额外开销 |
| 动画 | Vue 内置 Transition | - | TransitionGroup 处理列表动画，零额外体积 |
| 样式 | Tailwind CSS 4 | 4.x | 快速布局 + 游戏风格主题定制 |
| IPC | @tauri-apps/api | 2.x | 直接调用 Tauri invoke，无需兼容 shim |
| 窗口 | @tauri-apps/api/window | 2.x | 原生窗口控制 API（minimize/maximize/close） |

## 3. 架构

### 3.1 目录结构

```
src/                              # 新增前端源码目录
├── main.ts                       # Vue 应用入口
├── App.vue                       # 根组件：Setup ↔ Main 布局切换
├── composables/
│   ├── useAppState.ts            # 全局状态：uid, panel, templates, cache, dirty, undo
│   ├── useTheme.ts               # 主题切换 + overlay fade 过渡
│   ├── usePinyinSearch.ts        # 拼音搜索 composable
│   └── useKeyboard.ts            # 全局键盘快捷键 composable
├── lib/
│   ├── api.ts                    # Tauri invoke 封装，26 个现有 + 7 个新增 IPC 命令
│   ├── types.ts                  # TypeScript 类型定义（所有 IPC 请求/响应类型）
│   └── utils.ts                  # 工具函数（toast 等）
├── components/
│   ├── layout/
│   │   ├── TitleBar.vue          # 无边框窗口标题栏（拖拽 + 最小化/最大化/关闭）
│   │   ├── Sidebar.vue           # 导航侧栏（品牌、玩家选择、导航项、设置、主题切换）
│   │   └── MainContent.vue       # 内容区：根据 panel 状态渲染对应面板
│   ├── shared/
│   │   ├── GameCard.vue          # 通用游戏卡片（角色/音擎/驱动盘复用）
│   │   ├── EditorPage.vue        # 编辑器页面布局（顶栏 + 表单区 + 操作栏）
│   │   ├── Stepper.vue           # 步进器组件（+/- 按钮 + 数字输入）
│   │   ├── SearchBar.vue         # 搜索栏（拼音支持、清除按钮、IME 组合守卫）
│   │   ├── SkillGrid.vue         # 技能网格（6 个技能 + 核心被动）
│   │   ├── RankDots.vue          # 命座点显示
│   │   ├── StarRating.vue        # 星级显示
│   │   └── SkeletonGrid.vue      # 骨架屏加载状态
│   └── panels/
│       ├── SetupPanel.vue        # 初始设置（路径检测 + 手动输入 + 连接）
│       ├── AvatarsPanel.vue      # 角色管理（画廊 + 编辑器）
│       ├── WeaponsPanel.vue      # 音擎仓库（画廊 + 编辑器）
│       ├── EquipsPanel.vue       # 驱动盘仓库（画廊 + 编辑器 + 创建流程）
│       ├── HadalPanel.vue        # 式舆防卫战（入口卡片 + 房间表格）
│       ├── PlayerPanel.vue       # 玩家信息（表单 + 导出/导入）
│       ├── QuickLaunchPanel.vue  # 快速启动（启动卡片 + 一键启动 FAB）
│       ├── SettingsPanel.vue     # 设置（路径管理、主题、缓存、关于）
│       └── ShortcutsPanel.vue    # 快捷键参考
├── assets/
│   ├── pinyin-data.ts            # 拼音映射（从 pinyin-data.js 迁移为 TS 模块）
│   └── icon.png                  # 侧栏 logo
└── styles/
    └── theme.css                 # 游戏风格主题变量 + Naive UI 主题覆盖
```

### 3.2 与 Rust 后端的关系

```
┌─────────────────────────────────────────────────┐
│  Vue 3 Frontend (src/)                          │
│  ┌───────────┐  ┌──────────┐  ┌──────────────┐ │
│  │  Panels   │→ │ Composables│→ │  lib/api.ts  │ │
│  │ (9 views) │  │ (state)   │  │ (invoke)     │ │
│  └───────────┘  └──────────┘  └──────┬───────┘ │
│                                       │         │
├───────────────────────────────────────┼─────────┤
│  Tauri IPC Bridge                     │         │
├───────────────────────────────────────┼─────────┤
│  Rust Backend (src-tauri/src/)        │         │
│  ┌──────────┐  ┌──────────────┐  ┌───┴───────┐ │
│  │  api.rs   │  │data_manager.rs│  │ template  │ │
│  │ (31 cmds) │  │ (ZON I/O)    │  │ _loader.rs│ │
│  └──────────┘  └──────────────┘  └───────────┘ │
└─────────────────────────────────────────────────┘
```

`tauri.conf.json` 的 `frontendDist` 从 `../static` 改为 `../dist`（Vite 构建输出目录）。

### 3.3 构建流程

```
Vite 构建: src/ → dist/ (HTML + CSS + JS bundle)
Tauri 打包: dist/ 嵌入 WebView2
```

开发模式：`vite dev` 启动 dev server（端口 1420），Tauri WebView 加载 `http://localhost:1420`
生产模式：`vite build` 输出到 `dist/`，Tauri 打包嵌入

`tauri.conf.json` 配置：
- `beforeDevCommand`: `"npm run dev"`
- `beforeBuildCommand`: `"npm run build"`

## 4. 状态管理

### 4.1 useAppState composable

采用模块级单例模式（ref 直接导出）。权衡：简洁直接，适合单窗口桌面应用；缺点是无法在测试中替换实例。对于本项目的规模，简洁性优先。

```typescript
// composables/useAppState.ts
import { ref, reactive, shallowRef, computed } from 'vue'

// 全局单例状态
export const uid = ref<number | null>(null)
export const panel = ref<string>('avatars')
export const templates = shallowRef<Templates | null>(null)
export const avatarCache = shallowRef<Avatar[]>([])
export const weaponCache = shallowRef<Weapon[]>([])
export const equipCache = shallowRef<Equip[]>([])
export const cacheDirty = ref(false)
export const dirty = ref(false)

// 视图模式（画廊/编辑器）
export const avatarView = ref<'gallery' | 'editor'>('gallery')
export const weaponView = ref<'gallery' | 'editor'>('gallery')
export const equipView = ref<'gallery' | 'editor'>('gallery')

// 选中项
export const selectedAvatarId = ref<number | null>(null)
export const selectedWeaponUid = ref<number | null>(null)
export const selectedEquipUid = ref<number | null>(null)

// 分组模式
export const avatarGroupBy = ref<'camp' | 'profession'>('camp')

// 搜索
export const searchQuery = reactive({ avatars: '', weapons: '', equips: '' })

// 技能数据（角色编辑器保存时需要）
export const skillTypes = ref<Record<number, Record<string, string>>>({})
export const skillData = ref<Record<number, Record<string, number>>>({})

// 滚动位置保存（面板切换时恢复）
export const scrollPos = ref<Record<string, number>>({})

// 模板 Map（O(1) 查找）
export const avatarMap = computed(() => { ... })
export const weaponMap = computed(() => { ... })

// Undo 栈
const undoStack = ref<Snapshot[]>([])
const MAX_UNDO = 20
export function pushUndo(snap: Snapshot) {
  undoStack.value.push(snap)
  if (undoStack.value.length > MAX_UNDO) undoStack.value.shift()
}
export function popUndo(): Snapshot | undefined {
  return undoStack.value.pop()
}
```

### 4.2 缓存策略

与当前版本一致：
- `loadCounts()` 并行获取 avatar/weapon/equip 列表并填充缓存
- 保存后 `cacheDirty = true`，下次渲染时重新获取
- 搜索过滤使用 `computed` 避免重复计算

### 4.3 Dirty 追踪

- 编辑器内任何 input 事件设置 `dirty = true`
- 切换面板/玩家时检查 dirty，弹出确认对话框
- 导航项显示 dirty 指示器（脉动黄点）
- 保存成功后 `dirty = false`
- `beforeunload` 事件防止未保存关闭

## 5. 面板设计

### 5.0 初始设置 (SetupPanel)

应用启动时若未配置状态目录，显示全屏 Setup 视图：

- 品牌 logo + 应用名 + 版本号
- 状态目录输入框 + 粘贴按钮
- 自动检测路径逻辑：调用 `auto_detect_paths()`，候选路径显示为可点击列表
- 路径格式校验（参考 `player.js` L109 的正则）
- "连接"按钮：调用 `set_state_dir(path)` 验证并保存
- 验证成功后切换到主界面

### 5.1 角色管理 (AvatarsPanel)

**画廊视图：**
- 搜索栏 + 分组切换按钮（按阵营/按职业）
- CSS Grid 布局（`repeat(auto-fill, minmax(195px, 1fr))`）
- 卡片：稀有度边框（S 金色/A 紫色）、名称、等级、命座点、天赋计数、阵营标签、职业标签
- `TransitionGroup` 实现 stagger 入场动画
- 点击卡片：press 缩放动画 → 进入编辑器，保存滚动位置

**编辑器视图：**
- `EditorPage` 布局：返回链接 + 标题（中文名 + 英文名 + ID）
- 等级步进器 (1-60)、天赋步进器 (0-6)、命座自动计算显示
- 技能网格：6 个技能（普通攻击、强化特殊技、闪避、连携技、辅助技）+ 核心被动
- 觉醒等级步进器 (0-6)、武器 UID 输入、皮肤 ID 步进器
- 保存时从 `skillTypes`/`skillData` 重建 `skill_type_level` 数组

### 5.2 音擎仓库 (WeaponsPanel)

**画廊视图：**
- 按职业分组（6 个职业）
- 卡片：稀有度、名称、等级、星级、精炼、职业、UID
- 过滤 B 级音擎 (ID 12000-12999)

**编辑器视图：**
- 等级步进器 (1-60)、精炼步进器 (1-max_refine)
- 浮动保存按钮

### 5.3 驱动盘仓库 (EquipsPanel)

**画廊视图：**
- 按套装分组，每组有颜色标签（6 种套装颜色）
- 卡片：槽位号、UID、等级、星级、强化状态、主词条、副词条点

**创建流程（3 步 Modal）：**
- Step 1：选择套装（网格 + pop-in 动画，数据来自 `templates.suit_groups`）
- Step 2：选择槽位（I-VI 罗马数字）
- Step 3：配置主词条（数据来自 `templates.main_stat_options`，1-3 槽位固定）+ 4 个副词条行（数据来自 `templates.sub_stat_options`，每行含词条选择、基础值、强化步进器）
- 强化总和校验（必须 4-5）、重复词条检查

**编辑器视图：**
- 等级步进器 (0-15)、星级步进器 (1-5)
- 主词条选择（1-3 槽位固定禁用，词条名称来自 `templates.stat_names`）
- 4 个副词条行 + 实时强化总和显示
- 删除 + 保存浮动按钮组

### 5.4 式舆防卫战 (HadalPanel)

- 入口卡片网格（危局强袭站、稳定/剧变/特殊防卫战）
- 每个入口有图标、名称、类型（常驻/限时）、Zone ID 输入
- 已保存房间表格（zone_id, layer_index, avatar_id_list, buddy_id）
- 保存按钮

### 5.5 玩家信息 (PlayerPanel)

- 表单：昵称、等级、经验、展示角色 ID、控制角色 ID、伪装角色 ID
- 按钮：保存、导出（JSON 下载）、导入（文件选择 + 确认）
- **导出**：前端通过组合现有 IPC 命令（`get_player_basic` + `get_avatars` + `get_weapons` + `get_equips` + `get_hadal_zone`）自行组装 JSON，调用浏览器 `Blob` + `URL.createObjectURL` 下载
- **导入**：前端解析 JSON 文件，逐个调用 `update_*` 命令写入（非原子性，失败时提示已成功写入的部分）

### 5.6 快速启动 (QuickLaunchPanel)

需要后端新增命令（见第 8 节）。

- 启动项卡片：Yoshunko 服务端（自动检测 WSL）、HoyoSDK、KCPSHIM、Yidhari（管理员模式）
- 状态栏：已配置数量 (X/3)
- 卡片：状态点、名称、徽章（自动/已配置）、描述、保存的路径
- 一键启动 FAB（所有路径配置完成时出现）
- 粘贴按钮

### 5.7 设置 (SettingsPanel)

- 数据管理：状态目录（只读 + 更改按钮）、自动检测路径、清除缓存、重置配置
- 外观：主题切换（亮/暗按钮）
- 快捷键摘要 + 链接到快捷键面板
- 关于：应用名、版本、平台、数据状态

### 5.8 快捷键 (ShortcutsPanel)

- 三个区块：面板导航 (1-7)、编辑操作 (Ctrl+S/Z)、导航搜索 (Ctrl+F/ESC/Tab)

## 6. 动画系统

使用 Vue 内置 `<Transition>` 和 `<TransitionGroup>` 复现所有现有动画：

| 动画 | 实现方式 |
|---|---|
| 卡片 stagger 入场 | `<TransitionGroup>` + CSS `transition-delay` 按索引递增 |
| Editor slide-in | `<Transition name="editor-slide">` + CSS keyframes |
| Modal bounce | `<Transition name="modal">` + CSS `cubic-bezier(0.34, 1.56, 0.64, 1)` |
| Theme overlay fade | JS 创建 overlay div + CSS animation（与当前实现一致） |
| Skeleton shimmer | CSS `@keyframes shimmer` + gradient sweep |
| 卡片 press | `@mousedown` → `scale(0.92)` → `@mouseup` → `scale(1)` |
| Suit/slot pop-in | `<TransitionGroup>` + stagger delay |
| Toast 入场/离场 | `<Transition name="toast">` |
| 确认对话框 slide-up | `<Transition name="confirm">` |
| 导航激活条 | CSS transition on `scaleY` |

CSS 动画参数完全复用当前 `app.css` 中的 keyframes 和 timing functions。

### 6.1 无障碍动画支持

```typescript
// composables/useAnimation.ts
const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
const animationsOn = ref(localStorage.getItem('yos-animations') !== 'off')
export const animationsEnabled = computed(() => !prefersReducedMotion && animationsOn.value)
```

- `prefers-reduced-motion: reduce` 媒体查询禁用所有动画
- `data-animations="off"` 属性手动禁用
- 所有 Transition 组件通过 `:css="animationsEnabled"` 控制

## 7. 主题系统

### 7.1 实现

- Naive UI 内置 dark mode 支持，通过 `n-config-provider` 的 `theme` prop 切换
- 自定义 CSS 变量覆盖 Naive UI 默认值，匹配当前游戏风格配色
- `localStorage` 存储主题偏好，key `'yos-theme'`

### 7.2 Tailwind CSS 与 Naive UI 样式共存

Tailwind 的 Preflight（CSS Reset）会重置浏览器默认样式，可能影响 Naive UI 组件渲染。

解决方案：
- 在 `tailwind.config.ts` 中禁用 Preflight：`corePlugins: { preflight: false }`
- 或使用 Tailwind CSS 4 的 `@layer` 机制确保 Naive UI 样式优先级
- 自定义样式使用 `@layer utilities` 避免覆盖 Naive UI 组件样式

### 7.3 过渡动画

复现当前的 overlay fade 效果：
1. 捕获当前背景色
2. 瞬间切换主题
3. 创建 `.theme-fade-overlay` div，用旧背景色覆盖
4. overlay 淡出（0.62s），露出新主题

## 8. IPC 层

### 8.1 现有命令（26 个，无需改动）

直接使用 `@tauri-apps/api` 的 `invoke()`：

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { Config, PlayerBasic, PlayerBasicUpdate, AvatarListItem, AvatarDetail,
  AvatarUpdate, WeaponListItem, WeaponDetail, WeaponUpdate, EquipListItem, EquipDetail,
  EquipUpdate, EquipCreate, HadalZone, HadalZoneUpdate, Templates,
  DebugListDirResult, DebugAvatarIdsResult } from './types'

export const api = {
  // Config
  getConfig: () => invoke<Config>('get_config'),
  getVersion: () => invoke<{ version: string }>('get_version'),
  setStateDir: (path: string) => invoke('set_state_dir', { path }),
  autoDetectPaths: () => invoke<{ candidates: string[] }>('auto_detect_paths'),

  // Debug
  debugListDir: (path: string) => invoke<DebugListDirResult>('debug_list_dir', { path }),
  debugAvatarIds: (uid: number) => invoke<DebugAvatarIdsResult>('debug_avatar_ids', { uid }),

  // Templates
  getTemplates: () => invoke<Templates>('get_templates'),

  // Players
  getPlayerList: () => invoke<{ players: number[] }>('get_player_list'),
  getPlayerBasic: (uid: number) => invoke<PlayerBasic>('get_player_basic', { uid }),
  updatePlayerBasic: (uid: number, data: PlayerBasicUpdate) => invoke('update_player_basic', { uid, data }),

  // Avatars
  getAvatars: (uid: number) => invoke<{ avatars: AvatarListItem[] }>('get_avatars', { uid }),
  getAvatar: (uid: number, avatarId: number) => invoke<AvatarDetail>('get_avatar', { uid, avatarId }),
  updateAvatar: (uid: number, avatarId: number, data: AvatarUpdate) => invoke('update_avatar', { uid, avatarId, data }),

  // Weapons
  getWeapons: (uid: number) => invoke<{ weapons: WeaponListItem[] }>('get_weapons', { uid }),
  getWeapon: (uid: number, weaponUid: number) => invoke<WeaponDetail>('get_weapon', { uid, weaponUid }),
  updateWeapon: (uid: number, weaponUid: number, data: WeaponUpdate) => invoke('update_weapon', { uid, weaponUid, data }),

  // Equips
  getEquips: (uid: number) => invoke<{ equips: EquipListItem[] }>('get_equips', { uid }),
  getEquip: (uid: number, equipUid: number) => invoke<EquipDetail>('get_equip', { uid, equipUid }),
  updateEquip: (uid: number, equipUid: number, data: EquipUpdate) => invoke('update_equip', { uid, equipUid, data }),
  createEquip: (uid: number, data: EquipCreate) => invoke<{ ok: boolean; uid?: number; error?: string }>('create_equip', { uid, data }),
  deleteEquip: (uid: number, equipUid: number) => invoke('delete_equip', { uid, equipUid }),

  // Hadal Zone
  getHadalZone: (uid: number) => invoke<HadalZone>('get_hadal_zone', { uid }),
  updateHadalZone: (uid: number, data: HadalZoneUpdate) => invoke('update_hadal_zone', { uid, data }),
}
```

### 8.2 需新增的后端命令（7 个）

#### 快速启动（5 个命令）

当前 Rust 后端缺少快速启动功能。`tauri-plugin-shell` 的权限白名单机制不适合动态路径，且管理员权限提升（`ShellExecuteW` with `runas`）无法通过 shell 插件实现。**推荐在 Rust 后端新增命令。**

| 新增命令 | 功能 | 实现方式 |
|---|---|---|
| `get_launch_config` | 获取启动路径配置 | 从 `config.json` 读取 `launch_config` 字段 |
| `set_launch_path` | 保存启动路径 | 写入 `config.json` 的 `launch_config` |
| `launch_program` | 启动程序 | `std::process::Command::new(path)` |
| `launch_program_admin` | 管理员启动 | Windows: `ShellExecuteW` with `runas` |
| `launch_yoshunko` | 启动 Yoshunko 服务端 | `wsl -d <distro> -e bash -c "..."` |

#### 模板数据补全（1 个命令修改）

当前 `get_templates` 返回以下空字段：

```rust
// api.rs L183-191 — 当前返回值
json!({
    "avatars": avatars,
    "profession_names": {},       // 空 — 前端 equips.js 需要
    "suit_groups": {},            // 空 — 前端 equips.js L300, L333 需要
    "main_stat_options": {},      // 空 — 前端 equips.js L132 需要
    "sub_stat_options": [],       // 空 — 前端 equips.js L133 需要
    "stat_names": {},             // 空 — 前端 equips.js L72 需要
    "fixed_main_slots": [1,2,3],  // 有值
})
```

`template_loader.rs` 中已有 `equip_suit_types`、`equip_types`、`suit_names` 等数据，需在 `get_templates` 中补全序列化。

#### 窗口控制（不需新增命令）

窗口控制使用 `@tauri-apps/api/window` 的 JS API，无需经过 IPC：

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window'

export const windowApi = {
  minimize: () => getCurrentWindow().minimize(),
  toggleMax: () => getCurrentWindow().toggleMaximize(),
  close: () => getCurrentWindow().close(),
}
```

### 8.3 错误处理

Rust 后端返回格式不统一：
- 读取命令（`get_*`）直接返回数据对象或 `null`（未找到时）
- 写入命令（`update_*`）返回 `{ ok: true }` 或 `{ ok: false, error: "..." }`
- `create_equip` 返回 `{ ok: true, uid: number }` 或 `{ ok: false, error: "..." }`

统一错误处理策略：

```typescript
// lib/api.ts 内部
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const result = await invoke<T>(cmd, args)
  // 部分命令返回 { ok: false, error: "..." }
  const r = result as any
  if (r && typeof r === 'object' && 'ok' in r && r.ok === false) {
    throw new Error(r.error || 'Unknown error')
  }
  return result
}
```

全局错误提示通过 Naive UI 的 `n-notification` 或自定义 Toast 组件显示。

## 9. 配置变更

### 9.1 tauri.conf.json

```json
{
  "build": {
    "frontendDist": "../dist",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "withGlobalTauri": false
  }
}
```

CSP 配置：
- 开发模式：需允许 `ws://localhost:1420`（Vite HMR WebSocket）
- 生产模式：保持 `null`（当前设置）

### 9.2 capabilities/default.json

快速启动通过 Rust 后端命令实现（非 shell 插件），无需额外权限：

```json
{
  "permissions": [
    "core:default"
  ]
}
```

### 9.3 package.json（新增）

```json
{
  "name": "yoshunko-admin",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.5",
    "naive-ui": "^2.x",
    "@tauri-apps/api": "^2"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5",
    "vite": "^6",
    "vue-tsc": "^2",
    "typescript": "^5.6",
    "@tauri-apps/cli": "^2",
    "tailwindcss": "^4",
    "@tailwindcss/vite": "^4"
  }
}
```

### 9.4 vite.config.ts（新增）

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
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

## 10. TypeScript 类型定义

`lib/types.ts` 中定义所有 IPC 请求/响应类型，参照 `api.rs` 实际返回值结构。类型定义与代码保持同步，以 `api.rs` 为唯一真实来源。

```typescript
// ─── Config ────────────────────────────────────────

export interface Config {
  configured: boolean
  config_exists: boolean
  state_dir?: string
  version: string
  launch_config?: Record<string, string>
}

// ─── Player ────────────────────────────────────────

export interface PlayerBasic {
  nickname: string
  level: number
  exp: number
  avatar_id: number                 // 展示角色 ID
  control_avatar_id: number
  control_guise_avatar_id: number   // 伪装角色 ID
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

/** 列表项（get_avatars 返回） */
export interface AvatarListItem {
  avatar_id: number
  name: string
  en_name: string
  rarity: number
  profession: string
  level: number
  unlocked_talent_num: number
  is_favorite: boolean
  camp_id: number
}

/** 详情（get_avatar 返回，21 字段） */
export interface AvatarDetail {
  avatar_id: number
  name: string
  en_name: string
  rarity: number
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
  dressed_equip: (number | null)[]  // 6 个槽位，每个为 null 或装备 UID
  show_weapon_type: string          // "active" 等枚举值
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
  skill_type_level?: SkillTypeLevel[]   // 数组，非 Record
  awake_id?: number
  avatar_skin_id?: number              // 非 skin_id
  cur_weapon_uid?: number              // 非 dressed_weapon_uid
}

// ─── Weapon ────────────────────────────────────────

/** 列表项（get_weapons 返回） */
export interface WeaponListItem {
  id: number
  uid: number
  name: string
  en_name: string
  profession: string
  level: number
  star: number
  refine_level: number
  max_star: number
  max_refine: number
}

/** 详情（get_weapon 返回，扁平对象，无嵌套包装） */
export type WeaponDetail = WeaponListItem & {
  lock: boolean
}

export interface WeaponUpdate {
  level?: number
  star?: number
  refine_level?: number
}

// ─── Equip ─────────────────────────────────────────

/** 列表项（get_equips 返回） */
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

/** 详情（get_equip 返回，扁平对象，无嵌套包装） */
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
  sub_properties: (EquipProperty | null)[]  // 元素可为 null
}

export interface EquipCreate {
  id: number                        // 装备模板 ID（非 suit_type + slot）
  level: number
  star: number
  properties: EquipProperty[]
  sub_properties: (EquipProperty | null)[]
}

// ─── Hadal Zone ────────────────────────────────────

export interface HadalZone {
  entrances: HadalEntrance[]        // 无 rooms 字段
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
  avatars: AvatarTemplate[]         // 数组，非 Record
  weapons: WeaponTemplate[]
  profession_names: Record<string, string>
  suit_groups: Record<string, SuitGroup>
  main_stat_options: Record<number, StatOption[]>
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
```

## 11. 性能优化

1. **Naive UI 按需导入**：使用 `unplugin-vue-components` + `unplugin-auto-import` 自动按需导入 Naive UI 组件，避免全量引入（~500KB+）
2. **面板懒加载**：使用 `defineAsyncComponent` 实现路由级代码分割，仅加载当前面板
3. **虚拟滚动**：角色/音擎/驱动盘画廊使用 Naive UI 的 `n-virtual-list` 处理长列表
4. **搜索防抖**：`computed` + 150ms debounce，避免每次按键触发过滤

## 12. 键盘快捷键

使用 `composables/useKeyboard.ts` 统一管理全局快捷键：

```typescript
// composables/useKeyboard.ts
export function useKeyboard() {
  useEventListener(document, 'keydown', (e: KeyboardEvent) => {
    // ESC: 关闭确认/Modal/返回画廊
    // 1-7: 切换面板
    // Ctrl+S: 保存
    // Ctrl+Z: 撤销
    // Ctrl+F: 聚焦搜索
    // Tab: Modal 焦点陷阱
  })
}
```

IME 组合守卫：`compositionstart`/`compositionend` 事件追踪，搜索不在组合期间触发。

## 13. 迁移策略

### 13.1 文件映射

| 旧文件 | 新文件 | 变更类型 |
|---|---|---|
| `static/index.html` | `src/index.html` + `src/App.vue` | 重写为 Vue SFC |
| `static/js/tauri-compat.js` | 删除 | 不再需要 |
| `static/js/api.js` | `src/lib/api.ts` | 重写为 TS 模块 |
| `static/js/app.js` | `src/App.vue` + `src/components/layout/` + `src/composables/useKeyboard.ts` | 拆分为 Vue 组件 + composable |
| `static/js/state.js` | `src/composables/useAppState.ts` | 迁移为 composable |
| `static/js/utils.js` | `src/lib/utils.ts` | 迁移为 TS 工具函数 |
| `static/js/pinyin-data.js` | `src/assets/pinyin-data.ts` | 迁移为 TS 模块 |
| `static/js/panels/avatars.js` | `src/components/panels/AvatarsPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/weapons.js` | `src/components/panels/WeaponsPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/equips.js` | `src/components/panels/EquipsPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/hadal.js` | `src/components/panels/HadalPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/player.js` | `src/components/panels/PlayerPanel.vue` + `SetupPanel.vue` | 拆分设置和玩家信息 |
| `static/js/panels/quicklaunch.js` | `src/components/panels/QuickLaunchPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/settings.js` | `src/components/panels/SettingsPanel.vue` | 重写为 Vue SFC |
| `static/js/panels/shortcuts.js` | `src/components/panels/ShortcutsPanel.vue` | 重写为 Vue SFC |
| `static/css/app.css` | `src/styles/theme.css` | 迁移为 Tailwind + CSS 变量 |
| `static/icon.png` | `src/assets/icon.png` | 移动 |

### 13.2 实施顺序

1. **脚手架**：Vite + Vue 3 + Naive UI + Tailwind 项目初始化（`npm create tauri-app`）
2. **后端补全**：新增快速启动命令 + 补全 `get_templates` 返回数据
3. **IPC 层**：api.ts 封装全部命令 + types.ts 类型定义
4. **状态层**：useAppState composable
5. **布局**：TitleBar + Sidebar + MainContent
6. **共享组件**：GameCard, EditorPage, Stepper, SearchBar 等
7. **面板迁移**（按优先级）：
   - SetupPanel → PlayerPanel → AvatarsPanel → WeaponsPanel → EquipsPanel → HadalPanel → QuickLaunchPanel → SettingsPanel → ShortcutsPanel
8. **动画**：逐个面板添加 Transition 动画
9. **主题**：亮/暗主题 + overlay fade
10. **键盘快捷键**：useKeyboard composable
11. **测试验证**：逐功能对比 Python 版，确保无遗漏

## 14. 不在范围内

- Python 版代码不改动
- 数据文件（data/）不改动
- 应用图标不改动

## 15. 后端改动清单

| 文件 | 改动 | 说明 |
|---|---|---|
| `api.rs` | 新增 `get_launch_config` | 从 config.json 读取启动配置 |
| `api.rs` | 新增 `set_launch_path` | 写入启动路径到 config.json |
| `api.rs` | 新增 `launch_program` | `std::process::Command` 启动程序 |
| `api.rs` | 新增 `launch_program_admin` | Windows `ShellExecuteW` + `runas` |
| `api.rs` | 新增 `launch_yoshunko` | WSL 启动命令 |
| `api.rs` | 修改 `get_templates` | 补全 `suit_groups`、`main_stat_options`、`sub_stat_options`、`stat_names` |
| `lib.rs` | 注册新命令 | `invoke_handler` 添加 5 个新命令（共 31 个） |
| `tauri.conf.json` | 修改 `frontendDist` | `../static` → `../dist` |
| `tauri.conf.json` | 添加 `beforeDevCommand` / `beforeBuildCommand` | Vite 构建集成 |

## 16. 风险与缓解

| 风险 | 缓解 |
|---|---|
| Naive UI 主题与当前游戏风格不匹配 | CSS 变量覆盖 + `n-config-provider` 自定义主题 |
| Tailwind Preflight 与 Naive UI 样式冲突 | 禁用 Preflight 或使用 `@layer` 机制 |
| 动画效果与原版有差异 | 逐个对比 CSS 参数，精确复用 keyframes |
| 快速启动的管理员权限提升 | Rust 后端使用 Windows `ShellExecuteW` API |
| 导出/导入非原子性 | 前端实现，失败时提示已写入的部分 |
| Naive UI 全量引入体积大 | 使用 `unplugin-vue-components` 按需导入 |
| Rust 后端返回格式不统一 | `safeInvoke` 统一处理 `{ ok }` 格式 |
