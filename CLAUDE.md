# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Yoshunko Admin is a Tauri v2 desktop application for managing ZZZ (Zenless Zone Zero) game save data. The backend is Rust, the frontend is Vue 3 + Naive UI + Tailwind CSS 4. Data is stored as ZON (Zig Object Notation) files on the filesystem, not in a database.

## Development Commands

```bash
# Frontend dev server (Vite HMR on port 1420)
npm run dev

# Full Tauri dev (frontend + Rust backend with hot reload)
npm run tauri dev

# Type-check frontend
npx vue-tsc --noEmit

# Build frontend only
npm run build

# Build full release (outputs to src-tauri/target/release/)
# PowerShell:
./build-run.ps1
# Or manually:
npm run build && cd src-tauri && cargo tauri build
```

All Rust commands run from `src-tauri/`. Cargo workspace is not used.

## Architecture

### Backend (`src-tauri/src/`)

| File | Role |
|---|---|
| `lib.rs` | Tauri app builder, `AppState` definition, registers 31 IPC commands |
| `api.rs` | All `#[tauri::command]` handlers (config, templates, window, players, avatars, weapons, equips, hadal zone, quick launch) |
| `data_manager.rs` | `DataManager` — ZON file I/O with atomic writes (.tmp + rename), backup rotation (5 per file), audit logging |
| `template_loader.rs` | `TemplateLoader` — loads game metadata from JSON at startup (18 HashMaps for names, rarity, stats, etc.) |
| `zon.rs` | ZON format parser/serializer. `ZonValue` enum: Null, Bool, Int, String, Enum, Array, Object |

Key pattern: `AppState` holds `Option<DataManager>` and `TemplateLoader`. Commands access state via `tauri::State<'_, AppState>`. The `DataManager` is `None` until the user configures `state_dir`.

### Frontend (`src/`)

| Path | Role |
|---|---|
| `composables/useAppState.ts` | Core state singleton — uid, panel, templates, caches, dirty tracking, undo stack (max 20), computed avatarMap/weaponMap |
| `composables/useTheme.ts` | Light/dark theme with localStorage persistence |
| `composables/useKeyboard.ts` | Global shortcuts (1-7 panel switch, Ctrl+S save, Ctrl+Z undo, Ctrl+F search, ESC back) |
| `lib/api.ts` | Typed wrappers around `@tauri-apps/api/core` invoke() for all 31 commands |
| `lib/types.ts` | TypeScript interfaces matching Rust backend JSON responses |
| `components/layout/` | TitleBar (frameless window controls), Sidebar (nav + player select), MainContent (panel router with lazy loading) |
| `components/panels/` | 9 feature panels: Setup, Avatars, Weapons, Equips, HadalZone, Player, QuickLaunch, Settings, Shortcuts |
| `components/shared/` | Reusable: EditorPage, GameCard, SkeletonGrid, SearchBar, Stepper, SkillGrid, RankDots, StarRating |

Pattern: Each panel has a gallery view (card grid) and an editor view. Panels switch via `panel` ref in `useAppState`. App starts with SetupPanel if unconfigured.

### Data Layer

No database. ZON files stored at `{state_dir}/player/{uid}/`:
- `info/` — player basic info
- `avatar/{avatar_id}` — one file per character
- `weapon/{weapon_uid}` — one file per weapon
- `equip/{equip_uid}` — one file per equipment
- `hadal_zone/info` — Hadal Zone data
- `.backup/` — timestamped copies (rotation: keep 5)
- `audit.log` — write timestamps

### Game Data (`src-tauri/data/`)

- `templates/*.json` — 5 template files (AvatarBase, Weapon, Equipment, etc.)
- `en/` — English name mappings
- Root `*.json` — name/rarity/camp/profession/stat mappings

## Versioning Rules

- **当前版本**: v0.709-r
- **版本号格式**: `主版本.次版本.修订号`（如 `0.615.0`），显示为 `v0.615-r`
- **递增规则**: 每次针对底层代码（Rust 后端、核心逻辑、数据结构、IPC 接口等）的修改需递增修订号
- **满30进1**: 修订号满30则次版本+1，修订号归0（如 `0.629` → `0.700`）
- **不递增的情况**: 同一问题的重复修改（如反复调整 UI 字号/字重）、外围修改（文档、配置、构建脚本等）
- **CHANGELOG**: 每次递增版本号必须同步更新 `CHANGELOG.md`

## Conventions

- **Auto-imports**: Vue APIs and Naive UI composables/components are auto-imported via `unplugin-auto-import` and `unplugin-vue-components`. No explicit imports needed for Vue reactivity APIs or Naive UI components.
- **Path alias**: `@` maps to `src/` (configured in vite.config.ts and tsconfig.json).
- **Tauri IPC**: Frontend calls Rust via `invoke('command_name', { args })`. All 31 commands are registered in `lib.rs`.
- **Atomic writes**: All file saves go through `DataManager` which writes to `.tmp` then renames, with backup rotation.
- **No tests**: No automated tests exist. Verification is manual against the Python reference implementation.
- **Legacy code**: `static/` contains the old vanilla JS frontend (V0.609), kept for reference only.
