# Yoshunko Admin 包体结构优化方案

> 版本: v0.700-r | 日期: 2026-05-14

## 一、包体构成分析

Tauri v2 应用的最终安装包（NSIS）由以下部分组成：

| 组成部分 | 说明 | 是否在安装包中 |
|----------|------|:---:|
| Rust 二进制 | 包含嵌入的前端 dist/ 资源 | ✅ |
| data/ 模板数据 | 游戏元数据 JSON（17 个文件，454 KB） | ❌ 未配置 |
| icons/ | 安装包图标（138 KB） | ✅ |
| static/ 旧前端 | V0.609 遗留代码（435 KB） | ❌ 不参与构建 |
| gen/schemas/ | Tauri 开发时生成（339 KB） | ❌ 不参与构建 |

**关键发现**：`tauri.conf.json` 中缺少 `bundle.resources` 配置，导致 `data/` 目录**不会被打包进 NSIS 安装包**。发布版运行时 `lib.rs` 会尝试 `{exe_dir}/data`，如果不存在则回退 `CARGO_MANIFEST_DIR/data`（发布版中不存在），**这意味着当前安装包安装后应用无法加载模板数据**。这是一个必须修复的缺陷。

### 各资源文件详情

**data/ 目录（454 KB）**：

| 文件 | 大小 | 代码实际使用字段 |
|------|------|-----------------|
| `templates/EquipmentTemplateTb.json` | 272 KB | `item_id`, `equipment_type`, `suit_type`（仅 3/8 字段） |
| `templates/WeaponTemplateTb.json` | 114 KB | `item_id`, `weapon_name`, `star_limit`, `refine_limit`（仅 4/27 字段） |
| `templates/EquipmentSuitTemplateTb.json` | 36 KB | `id`, `name`（仅 2/≈8 字段） |
| `templates/AvatarBaseTemplateTb.json` | 25 KB | `id`, `code_name`, `camp`（仅 3/≈8 字段） |
| `templates/AvatarFormTemplateTb.json` | 304 B | 未使用 |
| 其余 12 个名称映射 JSON | 17 KB | 全部使用 |

**src/assets/ 目录（532 KB）**：

| 文件 | 大小 | 引用情况 |
|------|------|---------|
| `background.webp` | 317 KB | **无任何引用**（死资源） |
| `icon.ico` | 110 KB | Sidebar Logo（85×85 显示，加载 110KB ICO） |
| `icon.png` | 87 KB | `index.html` favicon（87KB 过大） |
| `pinyin-data.ts` | 17 KB | 3 个面板使用（AvatarsPanel, WeaponsPanel, EquipsPanel） |

**CSS 文件（85 KB 合计）**：

| 文件 | 大小 | 说明 |
|------|------|------|
| `theme.css` | 62 KB | 设计系统 + 全部组件样式 |
| `vue-extras.css` | 23 KB | Vue 过渡 + 组件样式（与 theme.css 大量重复） |

经逐类比对，两个 CSS 文件中存在 **98 个重复定义的 CSS 类**，包括 `.btn`、`.form-input`、`.search-wrap`、`.skeleton`、`.launch-card`、`.setup-card`、`.empty-state`、`.toast`、`.confirm-dialog`、`.data-table`、`.panel-box` 等核心组件样式。

---

## 二、优化方案

### P0 — 必须修复 / 零风险清理

#### 1. 配置 `bundle.resources` 打包 data/ 目录

**问题**：当前 `tauri.conf.json` 缺少 `bundle.resources`，导致安装包不包含模板数据，发布版无法正常运行。

**方案**：在 `tauri.conf.json` 的 `bundle` 中添加：

```json
"resources": [
  "data/*",
  "data/en/*",
  "data/templates/*"
]
```

**影响**：安装包增加 ~454 KB（精简后可降至 ~50 KB，见优化项 #4）。

#### 2. 删除 `src/assets/background.webp`（节省 317 KB）

**事实**：经全项目搜索，`background.webp` 在 `src/` 目录下无任何代码引用（0 处 import/src/URL 引用）。

**操作**：直接删除文件。

**影响**：前端 dist 嵌入二进制后，直接减少二进制体积 317 KB。

#### 3. 删除 `static/` 旧前端目录

**事实**：`static/` 是 V0.609 的遗留 vanilla JS 前端，`tauri.conf.json` 的 `frontendDist` 指向 `../dist`（Vue 构建产物），`static/` 不参与任何构建流程。

**操作**：删除 `static/` 目录。如需保留作参考，可移至独立 Git 分支。

**影响**：不影响安装包体积（本来就不打包），但减少仓库体积 ~435 KB。

#### 4. 在 `.gitignore` 中排除 `src-tauri/gen/`

**事实**：`gen/schemas/`（339 KB）是 `tauri build` 自动生成的开发时文件，不应纳入版本控制。

**操作**：在 `.gitignore` 添加：

```
src-tauri/gen/
```

**影响**：不影响安装包体积，减少仓库体积 339 KB。

---

### P1 — 高收益优化

#### 5. 精简模板 JSON（节省 ~400 KB）

**事实**：模板 JSON 包含大量未使用字段，特别是：

- `EquipmentTemplateTb.json`（272 KB）：504 条数据 × 8 字段，仅用 3 字段（`item_id`, `equipment_type`, `suit_type`），未用字段含纹理路径、音乐标签等
- `WeaponTemplateTb.json`（114 KB）：93 条数据 × 27 字段，仅用 4 字段（`item_id`, `weapon_name`, `star_limit`, `refine_limit`），未用字段含 `base_property`、`rand_property`、`weapon_script_config`、`weapon_ui_model`、`unk_*` 系列等 23 个
- `AvatarFormTemplateTb.json`（304 B）：`template_loader.rs` 中完全未加载此文件

**方案 A（推荐 — 构建时预处理）**：

在 `build.rs` 中添加逻辑，读取原始模板 JSON，只提取需要的字段，生成精简版写入 `OUT_DIR`，然后 `template_loader.rs` 从精简版加载。

预估精简后大小：

| 文件 | 原始 | 精简后 | 压缩比 |
|------|------|--------|--------|
| EquipmentTemplateTb.json | 272 KB | ~15 KB | 94% |
| WeaponTemplateTb.json | 114 KB | ~5 KB | 96% |
| EquipmentSuitTemplateTb.json | 36 KB | ~3 KB | 92% |
| AvatarBaseTemplateTb.json | 25 KB | ~8 KB | 68% |
| AvatarFormTemplateTb.json | 304 B | 删除 | 100% |
| **合计** | **447 KB** | **~31 KB** | **93%** |

**方案 B（简单 — 手动替换）**：

手动创建精简版 JSON 替换原文件，保留原始文件在 `data/templates/full/` 下作参考。

**影响**：安装包中 data/ 从 454 KB 降至 ~50 KB，同时减少运行时内存占用和 JSON 解析时间。

#### 6. 精简图标资源（节省 ~180 KB）

**事实**：

- `src/assets/icon.ico`（110 KB）和 `src-tauri/icons/icon.ico`（110 KB）是同一文件（字节数相同），存在重复
- Sidebar 中 `<img src="@/assets/icon.ico">` 加载 110 KB ICO 文件仅用于 85×85 像素 Logo 显示
- `src/assets/icon.png`（87 KB）仅被 `index.html` 的 `<link rel="icon">` 引用作 favicon，87 KB 对 favicon 过大

**方案**：

1. 为 Sidebar Logo 生成专用小尺寸 PNG（如 128×128，预估 ~8 KB），替换 `icon.ico` 引用
2. 为 favicon 生成 32×32 PNG（预估 ~2 KB），替换 `icon.png`
3. 删除 `src/assets/icon.ico` 和 `src/assets/icon.png`

**影响**：前端 dist 减少约 180 KB 嵌入资源。

#### 7. CSS 去重合并（节省 ~20-30 KB）

**事实**：`theme.css`（62 KB）和 `vue-extras.css`（23 KB）存在 98 个重复定义的 CSS 类。两个文件对同一组件（如 `.btn`、`.form-input`、`.search-wrap`）给出了不同的样式值，说明是不同迭代阶段的产物。

**方案**：

1. 以 `theme.css` 为主（更完整、更新），逐类比对 `vue-extras.css` 中的差异
2. 将 `vue-extras.css` 中独有的样式（主要是 Vue Transition 动画定义）合并到 `theme.css`
3. 删除 `vue-extras.css`，从 `main.ts` 中移除 `import './styles/vue-extras.css'`

**影响**：CSS 体积从 85 KB 降至 ~55-60 KB（去重 + 压缩后效果更显著），直接减少嵌入二进制的体积。

---

### P2 — 进阶优化

#### 8. 拼音数据后端化（节省 ~17 KB 前端体积）

**事实**：`pinyin-data.ts`（17 KB）导出 `AVATAR_PINYIN`、`WEAPON_PINYIN`、`SUIT_PINYIN` 三个映射表，用于前端搜索排序。这些数据与后端 `data/` 下的名称映射存在重叠——后端已有所有 ID→名称的映射，拼音可以由后端一并提供。

**方案**：

1. 在 Rust 后端 `template_loader.rs` 中为每个 avatar/weapon/suit 生成拼音首字母
2. 通过 `get_templates` 命令将拼音数据随模板一起返回
3. 前端删除 `pinyin-data.ts`，改为从模板数据中读取拼音

**影响**：前端减少 17 KB 硬编码数据，数据源统一到后端，新增游戏内容时无需手动更新前端拼音表。

#### 9. 发布版去掉 `devtools` feature

**事实**：`Cargo.toml` 中 `tauri = { version = "2", features = ["devtools"] }` 启用了 WebView 开发者工具。在 WebView2 上，devtools 本身是系统组件，此 feature 仅控制是否允许在 release 中打开。对包体大小影响极小（仅移除少量胶水代码），但可减少攻击面。

**方案**：使用条件编译，仅在 debug 模式启用 devtools：

```toml
[target.'cfg(debug_assertions)'.dependencies]
tauri = { version = "2", features = ["devtools"] }

[dependencies]
tauri = { version = "2", features = [] }
```

或更简单地，直接移除 `devtools` feature，需要调试时临时加回。

**影响**：包体减少可忽略（<10 KB），但提升安全性。

#### 10. Rust 二进制压缩

**事实**：当前 `Cargo.toml` release profile 已配置：

```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
opt-level = "s"
```

这是 Tauri 项目的标准优化配置，已接近最优。

**可选方案 — UPX 压缩**：

UPX 可进一步压缩 Windows PE 可执行文件 40-60%。**但存在重大风险**：

- ⚠️ 杀毒软件误报率极高（UPX 压缩的 exe 几乎必然触发 Windows Defender 等的启发式检测）
- 启动速度略慢（需运行时解压）
- 可能影响代码签名

**建议**：除非有强制的包体大小要求，否则**不推荐**使用 UPX。当前 LTO + strip + opt-level="s" 已是安全与体积的最佳平衡。

---

## 三、优化效果汇总

### 对安装包体积的影响

| # | 优化项 | 安装包节省 | 难度 | 风险 |
|---|--------|-----------|------|------|
| 1 | 配置 bundle.resources | +50 KB（修复缺陷） | ⭐ | 无 |
| 2 | 删除 background.webp | -317 KB | ⭐ | 无 |
| 5 | 精简模板 JSON | -400 KB | ⭐⭐ | 低 |
| 6 | 精简图标资源 | -180 KB | ⭐⭐ | 低 |
| 7 | CSS 去重合并 | -25 KB | ⭐⭐⭐ | 中 |
| 8 | 拼音数据后端化 | -17 KB | ⭐⭐⭐ | 中 |
| 9 | 去掉 devtools | -<10 KB | ⭐ | 无 |
| **合计** | | **~890 KB** | | |

### 对仓库体积的影响

| # | 优化项 | 仓库节省 |
|---|--------|---------|
| 3 | 删除 static/ | ~435 KB |
| 4 | 排除 gen/schemas/ | ~339 KB |
| 2 | 删除 background.webp | ~317 KB |
| **合计** | | **~1091 KB** |

---

## 四、推荐执行顺序

1. **第一步（5 分钟）**：删除 `background.webp`，`.gitignore` 添加 `src-tauri/gen/`，配置 `bundle.resources`
2. **第二步（30 分钟）**：精简模板 JSON（方案 B 手动替换更快捷），替换 Sidebar 图标和 favicon
3. **第三步（1-2 小时）**：CSS 去重合并，拼音数据后端化
4. **第四步（可选）**：去掉 devtools feature，删除 `static/`

---

## 五、验证方法

每次优化后执行以下验证：

```bash
# 1. 前端类型检查
npx vue-tsc --noEmit

# 2. 前端构建（检查 dist/ 大小）
npm run build

# 3. 完整 Tauri 构建（检查最终 exe 大小）
cd src-tauri && cargo tauri build

# 4. 检查安装包大小
# NSIS 安装包位于: D:\cargo-build\release\bundle\nsis\
```

重点关注 `npm run build` 后 `dist/` 目录的总大小变化，以及最终 NSIS 安装包的 `.exe` 大小变化。
