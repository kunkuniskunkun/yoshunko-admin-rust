# Yoshunko Admin Rust 版 — 深度优化建议

> 基于对项目全部源码（Rust 后端 5 文件 / Vue 前端 20+ 文件 / CSS 1600+ 行）的逐行审阅，按优先级和影响面分类整理。  
> 日期：2026-05-15 · 当前版本：V0.627

---

## 目录

1. [后端 Rust 优化](#1-后端-rust-优化)
2. [前端 Vue 优化](#2-前端-vue-优化)
3. [CSS / 设计系统优化](#3-css--设计系统优化)
4. [架构与数据流优化](#4-架构与数据流优化)
5. [安全加固](#5-安全加固)
6. [构建与部署优化](#6-构建与部署优化)
7. [可访问性与 UX 细节](#7-可访问性与-ux-细节)
8. [优化路线图建议](#8-优化路线图建议)

---

## 1. 后端 Rust 优化

### 1.1 DataManager 缓存策略改进 [高优先级]

**现状**：`DataManager.cache` 是 `HashMap<PathBuf, BTreeMap<String, ZonValue>>`，读取时缓存命中直接返回 clone，写入时也 clone 入缓存。对于大型存档（100+ 角色、200+ 音擎），每次 `get_avatars` 调用会触发 N 次 `BTreeMap::clone`。

**问题**：
- `read_zon_obj` 返回 `Some(cached.clone())` — 每次列表查询都完整克隆所有已缓存对象
- `write_zon` 中 `self.cache.insert(path, data.clone())` — 保存时再克隆一份
- 缓存没有 TTL / 失效机制，长时间运行后缓存可能与磁盘不同步（外部程序修改文件）

**建议**：
- 引入 `Arc<BTreeMap<String, ZonValue>>` 替代裸 `BTreeMap`，clone 只增加引用计数
- 为缓存条目添加 `mtime` 时间戳，读取时比对文件修改时间决定是否需要重新解析
- 考虑 `DashMap` 替代 `HashMap + Mutex`，减少锁争用（当前 `with_manager` 每次调用都锁整个 DataManager）

### 1.2 ZON 解析器健壮性 [高优先级]

**现状**：`zon.rs` 的 tokenizer 是手写的逐字符解析器，存在以下边界情况：

**问题**：
- 大文件（>1MB）时 `chars().collect::<Vec<char>>()` 一次性分配完整字符数组
- 解析错误只返回 `String` 而非结构化错误类型，无法区分语法错误 / IO 错误 / 类型错误
- `parse_brace` 中通过 lookahead 区分数组 vs 结构体，逻辑脆弱
- 序列化器对嵌套深度没有限制，恶意输入可能导致栈溢出

**建议**：
- 大文件场景改为 `peekable` 迭代器或 `&str` 切片遍历，避免全量 `collect`
- 定义 `ZonError` 枚举（`Syntax { pos, msg }` / `Io` / `Type`），实现 `std::error::Error`
- 添加递归深度限制（如 max_depth = 64），超限返回错误
- 为关键解析路径添加 fuzz 测试

### 1.3 API 层输入验证增强 [中优先级]

**现状**：`api.rs` 的验证集中在数值范围（`check_range`），但缺少以下验证：

**问题**：
- `update_player_basic` 的 `nickname` 无长度限制，可写入超长字符串
- `create_equip` / `update_equip` 的 `properties` 数组无长度限制
- `BTreeMap<String, ZonValue>` 作为命令参数，前端可传入任意 key，后端无白名单过滤
- `set_state_dir` 的路径参数未做路径遍历检查（如 `../../etc/passwd`）

**建议**：
- 为每个 update 命令定义允许的字段白名单，拒绝未知 key
- 添加字符串长度限制（nickname ≤ 64 字符）
- 添加数组长度限制（properties ≤ 1, sub_properties ≤ 4）
- `set_state_dir` 验证路径不包含 `..` 且为绝对路径

### 1.4 配置文件写入竞态 [中优先级]

**现状**：`set_state_dir` 和 `set_launch_path` 都采用 `.tmp + rename` 原子写入，但：

**问题**：
- 两个命令独立读取 → 修改 → 写入 `config.json`，并发调用可能丢失其中一个的修改
- `get_launch_config` 读取时没有加锁，可能与 `set_launch_path` 并发

**建议**：
- 将配置读写统一到一个带锁的 `ConfigManager`，类似 `DataManager` 的模式
- 或使用 `fs2` 文件锁确保跨进程安全

### 1.5 审计日志改进 [低优先级]

**现状**：`audit_log` 写入 `{state_dir}/../audit.log`，即 `audit.log` 在 `state_dir` 的父目录。

**问题**：
- `self.player_dir.join("..").join("audit.log")` 使用 `..` 路径，语义不清晰
- 日志格式为纯文本，难以程序化解析
- 轮换策略仅基于文件大小（1MB），无时间维度

**建议**：
- 使用 `Path::parent()` 替代 `join("..")`
- 日志格式改为 JSON Lines（每行一个 JSON 对象），便于后续分析
- 添加时间维度轮换（如保留最近 7 天）

---

## 2. 前端 Vue 优化

### 2.1 三面板代码重复消除 [高优先级]

**现状**：`AvatarsPanel.vue`、`WeaponsPanel.vue`、`EquipsPanel.vue` 存在大量结构重复：

**重复模式**：
- `refreshCache()` — 逻辑几乎相同（检查 dirty → 调 API → 更新 cache → 设 loading=false）
- `selectXxx()` — 卡片点击动画 + 设置选中 + 加载编辑器 + slide-in 动画
- `backToGallery()` — 重置视图 + 选中项 + staggered animation
- `saveXxx()` — 调 API → 检查 ok → toast → markClean → markCacheDirty → backToGallery
- `onMounted` / `onActivated` / `watch(panel)` — 生命周期逻辑完全相同
- `groupedXxx` computed — 分组 + 排序 + 分配 stagger index

**建议**：
- 提取 `usePanelGallery<T>(options)` composable，封装：
  - `loading`, `editorData`, `editorLoading` 状态
  - `refreshCache()`, `selectItem()`, `backToGallery()`, `saveItem()` 方法
  - `onMounted` / `onActivated` / `watch(panel)` 生命周期
  - `filteredItems` / `groupedItems` computed
- 提取 `useCardAnimation()` composable，封装卡片点击缩放动画
- 预计减少 ~300 行重复代码

### 2.2 `(item as any)._i` 类型安全 [中优先级]

**现状**：三个面板都在列表项上设置 `(item as any)._i = idx++` 用于交错动画索引。

**问题**：
- `as any` 绕过类型检查，容易误用
- 修改了从 API 返回的响应式对象，可能导致意外行为

**建议**：
- 定义 `StaggeredItem<T>` 包装类型：`{ data: T; staggerIndex: number }`
- 或使用 `WeakMap<object, number>` 存储索引映射，不污染原始数据
- 或在模板中使用 `$index` + 分组偏移计算

### 2.3 `filteredAvatars` 硬编码过滤 [中优先级]

**现状**：`AvatarsPanel.vue` 第 43 行硬编码过滤 `avatar_id !== 2071 && avatar_id !== 2121`，`WeaponsPanel.vue` 第 25 行硬编码 `w.id < 12000 || w.id > 12999`。

**问题**：
- 魔数含义不明，维护者无法理解为什么排除这些 ID
- 游戏版本更新后可能需要调整，但无法搜索定位

**建议**：
- 提取为命名常量：`const HIDDEN_AVATAR_IDS = [2071, 2121]`，`const WEAPON_ID_NPC_RANGE = [12000, 12999]`
- 添加注释说明排除原因（如"NPC 专用角色"、"NPC 专用音擎"）
- 考虑将过滤逻辑移到后端 `get_avatars` / `get_weapons`，前端无需关心

### 2.4 HadalPanel / PlayerPanel 缺少 `onActivated` [中优先级]

**现状**：`HadalPanel.vue` 和 `PlayerPanel.vue` 只在 `onMounted` 中加载数据，没有 `onActivated` 钩子。

**问题**：
- 由于 `MainContent.vue` 使用了 `<KeepAlive>`，切换面板再切回来不会重新触发 `onMounted`
- 如果用户在另一个面板修改了数据（如切换了 uid），再切回这两个面板时数据不会刷新

**建议**：
- 添加 `onActivated` 钩子，重新加载当前 uid 的数据
- 与 AvatarsPanel / WeaponsPanel / EquipsPanel 保持一致的生命周期模式

### 2.5 导入功能不完整 [中优先级]

**现状**：`PlayerPanel.vue` 的 `importData()` 只导入了 `info`（玩家基本信息），没有导入角色/音擎/驱动盘/式舆数据。

**问题**：
- 导出文件包含完整数据（info + avatars + weapons + equips + hadal_zone），但导入只恢复 info
- 用户可能期望导入是导出的逆操作

**建议**：
- 完整实现导入：遍历 avatars/weapons/equips 逐个调用 update API
- 添加进度提示（导入 N/M 条数据）
- 考虑后端新增 `batch_update` 命令，一次 IPC 调用完成批量导入

### 2.6 `document.querySelector` 直接 DOM 操作 [低优先级]

**现状**：多处使用 `document.querySelector('.main-content')` 获取 DOM 元素：

- `AvatarsPanel.vue` 第 127 行
- `WeaponsPanel.vue` 第 103 行
- `EquipsPanel.vue` 第 155 行
- `useStaggeredAnimation.ts` 第 8 行

**问题**：
- 硬编码 CSS 选择器，重构时容易遗漏
- 在 Vue 组件中直接操作外部 DOM 违反组件封装原则

**建议**：
- 通过 `ref` + `defineExpose` 或 `provide/inject` 传递 DOM 引用
- 或将动画逻辑封装为 CSS-only 方案（当前已部分实现 `.staggered-anim`，但 `editor-slide-in` 仍需 JS 触发）

### 2.7 `setTimeout` 未清理 [低优先级]

**现状**：三面板的 `selectXxx` 函数中都有 `setTimeout(() => { el.style.transform = 'scale(1)' }, 120)`，但未在组件卸载时清理。

**问题**：
- 如果用户在 120ms 内切换面板，回调可能在已卸载组件的 DOM 元素上执行
- `GameCard.vue` 已正确处理（`onUnmounted` 清理 timer），但面板内联代码没有

**建议**：
- 统一使用 `GameCard.vue` 的 `onPress` 方法处理卡片点击动画
- 或提取为 `usePressAnimation()` composable

---

## 3. CSS / 设计系统优化

### 3.1 theme.css 体量控制 [高优先级]

**现状**：`theme.css` 已达 1650 行，包含设计系统变量、组件样式、布局样式、动画、响应式、暗色主题覆盖、合并段。

**问题**：
- 单文件维护成本高，修改一个组件样式需要在大文件中搜索
- 合并段（第 1584 行起 `/* Merged from vue-extras.css */`）仍残留部分重复规则
- 部分样式同时使用 CSS 变量和硬编码值（如 `.rarity-s` 的 `background: #c49a2e` 未使用 `--rarity-s`）

**建议**：
- 按关注点拆分为多个 CSS 文件：
  - `tokens.css` — CSS 变量（颜色/间距/字号/阴影/动画）
  - `base.css` — Reset / 排版 / 滚动条
  - `layout.css` — 标题栏 / 侧边栏 / 主内容区
  - `components.css` — 卡片 / 按钮 / 表单 / 模态框 / Toast
  - `panels.css` — 面板特定样式（角色/音擎/驱动盘/快速启动等）
  - `dark.css` — 暗色主题覆盖
  - `animations.css` — 所有 @keyframes 和动画类
- 消除硬编码颜色值，统一使用 CSS 变量
- 删除合并段中确认无用的规则

### 3.2 CSS 变量一致性 [中优先级]

**现状**：部分变量定义了但未使用，部分样式绕过变量直接硬编码。

**问题**：
- `--transition-bounce` 定义了但从未使用
- `--z-sticky` / `--z-dropdown` 定义了但未使用
- `--text-xs` (10px) 和 `--text-sm` (11px) 差距过小，实际使用中常混用
- `.rarity-s` 使用 `background: #c49a2e` 而非 `var(--rarity-s)`
- `--font-display` 和 `--font` 值相同，冗余

**建议**：
- 审计所有 CSS 变量使用情况，删除未使用的
- 合并 `--text-xs` 和 `--text-sm`（10px 和 11px 视觉差异极小）
- 所有颜色引用统一使用 CSS 变量
- 删除 `--font-display` 或赋予其差异化值

### 3.3 响应式断点补充 [低优先级]

**现状**：只有 2 个响应式断点（1100px / 960px），且 960px 是最小窗口宽度。

**问题**：
- 在 960px-1100px 区间，卡片网格可能只有 2 列，空间利用率低
- 没有针对超宽屏幕（>1600px）的优化，卡片网格可能过宽

**建议**：
- 添加 1400px+ 断点，优化超宽屏布局（如侧边栏加宽、卡片网格 4-5 列）
- 考虑侧边栏在窄屏时自动折叠为图标模式

---

## 4. 架构与数据流优化

### 4.1 状态管理集中化 [高优先级]

**现状**：`useAppState.ts` 导出 20+ 个独立 ref/computed，面板组件直接 import 使用。

**问题**：
- 状态分散，没有统一的读写接口，任何组件都能直接修改任何状态
- `dirty` 是全局的，但实际应该是每面板独立的
- `cacheDirty` 是全局的，但三个面板的缓存独立
- 搜索查询 `searchQuery` 是 reactive 对象但只有 3 个字段

**建议**：
- 将状态分为几个逻辑域：
  - `useAppConfig()` — configured, uid, templates
  - `useNavigation()` — panel, view states
  - `useAvatarStore()` — avatarCache, selectedAvatarId, avatarDirty
  - `useWeaponStore()` — weaponCache, selectedWeaponUid, weaponDirty
  - `useEquipStore()` — equipCache, selectedEquipUid, equipDirty
- 每个 store 提供 `refresh()` / `save()` / `select()` / `back()` 方法
- dirty 状态改为每面板独立，关闭窗口时只检查当前面板

### 4.2 IPC 调用批量化 [中优先级]

**现状**：`MainContent.vue` 的 `loadCounts` 一次发 3 个 IPC 调用（getAvatars + getWeapons + getEquips），`PlayerPanel` 的 `exportData` 一次发 5 个。

**问题**：
- 每次切换 uid 都触发 3 个独立 IPC 调用
- IPC 调用有固定开销（序列化/反序列化/线程切换）
- 导出功能 5 个调用串行等待

**建议**：
- 后端新增 `get_all_counts` 命令，一次返回三类数据的列表摘要
- 后端新增 `export_player` 命令，一次返回完整玩家数据
- 减少前端 IPC 调用次数，降低延迟

### 4.3 错误处理统一 [中优先级]

**现状**：每个 async 函数都有独立的 try/catch + `toast(e instanceof Error ? e.message : 'xxx失败', 'error')`。

**问题**：
- 错误处理代码重复率极高
- 部分错误消息是中文（`'加载失败'`），部分是英文（`'Failed to write config'`），不一致
- 后端返回的错误消息也是中英混杂

**建议**：
- 提取 `useAsyncAction()` composable：自动处理 loading / error / toast
- 或提取 `asyncWrap(fn, errorMsg)` 工具函数
- 统一错误消息语言（建议全部中文，面向中文用户）
- 后端错误消息也统一为中文

### 4.4 撤销栈粒度 [低优先级]

**现状**：`pushUndo` / `popUndo` 支持快照式撤销，但当前只有框架没有实际使用。

**问题**：
- 三个编辑面板都没有调用 `pushUndo`
- `Ctrl+Z` 快捷键已注册但实际无效果
- 撤销栈是全局的，但编辑操作是面板独立的

**建议**：
- 在每个面板的 `saveXxx` 之前调用 `pushUndo`，保存当前编辑状态
- 或实现字段级撤销（每个 Stepper 的 beforeUpdate 保存旧值）
- 撤销栈改为面板独立

---

## 5. 安全加固

### 5.1 CSP 策略 [高优先级]

**现状**：`tauri.conf.json` 中 `"csp": null`，完全禁用了内容安全策略。

**问题**：
- WebView 可以加载任意外部资源（脚本、样式、图片、字体）
- 如果游戏数据中包含恶意 URL（如在 nickname 中注入 `<script>`），可能触发 XSS

**建议**：
- 设置合理的 CSP：
  ```json
  "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'"
  ```
- `unsafe-inline` 是 Tailwind CSS 所需，但应禁止 `unsafe-eval` 和外部脚本

### 5.2 路径遍历防护 [高优先级]

**现状**：`set_state_dir` 接受用户输入的任意路径，只检查 `player/` 子目录是否存在。

**问题**：
- 用户可能输入系统敏感路径（如 `C:\Windows\System32`）
- `DataManager` 会在此路径下创建/修改/删除文件

**建议**：
- 验证路径不包含 `..`
- 验证路径是绝对路径
- 验证路径不在系统目录下（如 `C:\Windows`、`C:\Program Files`）
- 考虑使用 Tauri 的 `dialog::open` 让用户选择目录，而非手动输入

### 5.3 输入消毒 [中优先级]

**现状**：`nickname` 等字符串字段直接写入 ZON 文件，无任何过滤。

**问题**：
- 虽然 ZON 是自定义格式不会被浏览器解析，但超长字符串可能导致 ZON 文件异常
- 特殊字符（如引号、换行符）可能导致 ZON 序列化/反序列化异常

**建议**：
- 字符串字段长度限制（nickname ≤ 64 字符）
- 过滤控制字符（\x00-\x1F）
- 验证 ZON 序列化后再反序列化结果一致

### 5.4 capabilities 权限最小化 [低优先级]

**现状**：`default.json` 中 `shell:default` + `shell:allow-open` 权限较宽。

**建议**：
- 审计是否真正需要 `shell:allow-open`
- 如果只在快速启动中使用，考虑限制为特定命令

---

## 6. 构建与部署优化

### 6.1 前端 bundle 分析 [中优先级]

**现状**：Vite 配置了 `manualChunks` 分离 `pinyin-data` (11KB) 和 `naive-ui` (337KB)。

**问题**：
- 没有实际的 bundle 体积监控
- naive-ui 只使用了约 10 个组件，但引入了完整包

**建议**：
- 添加 `rollup-plugin-visualizer` 构建时生成 bundle 分析报告
- 评估 naive-ui tree-shaking 效果（`unplugin-vue-components` + `NaiveUiResolver` 应已自动按需引入）
- 如果 tree-shaking 不理想，考虑手动引入组件替代 resolver

### 6.2 Rust 编译优化 [低优先级]

**现状**：`Cargo.toml` release profile 已配置 `lto = true`、`codegen-units = 1`、`opt-level = "s"`。

**建议**：
- 考虑 `opt-level = "z"` 进一步减小体积（可能牺牲少量性能）
- 添加 `panic = "abort"` 减小二进制体积（无需 unwind）
- 评估是否需要 `strip = true`（已配置）

### 6.3 CI/CD 增强 [低优先级]

**现状**：`.github/workflows/build.yml` 只在推送 `v*` tag 时构建。

**建议**：
- 添加 PR 构建检查（只构建不发布）
- 添加 `cargo clippy` 和 `vue-tsc --noEmit` 到 CI
- 考虑添加自动版本号递增（基于 conventional commits）

---

## 7. 可访问性与 UX 细节

### 7.1 键盘导航完善 [中优先级]

**现状**：
- `useKeyboard.ts` 注册了全局快捷键
- 卡片有 `tabindex="0"` 和 `@keydown.enter` / `@keydown.space`
- 但缺少焦点可见性样式和焦点陷阱

**问题**：
- 模态框（创建驱动盘）没有焦点陷阱，Tab 键可以跳出模态框
- 卡片焦点没有可见的焦点环（`:focus-visible` 未定义）
- 搜索框没有 `Ctrl+F` 快捷键聚焦（ShortcutsPanel 中列出了但未实现）

**建议**：
- 模态框添加焦点陷阱（Naive UI 的 `n-modal` 已内置）
- 添加 `.game-card:focus-visible` 样式（`outline: 2px solid var(--accent)`）
- 实现 `Ctrl+F` 聚焦搜索框

### 7.2 加载状态优化 [低优先级]

**现状**：首次加载显示 `SkeletonGrid`，但编辑器加载只显示一个小 spinner。

**建议**：
- 编辑器加载时显示骨架屏而非 spinner，视觉连续性更好
- 保存操作添加 loading 状态，防止重复点击

### 7.3 空状态引导 [低优先级]

**现状**：空状态只显示"没有找到匹配的 X"。

**建议**：
- 提供操作建议（如"尝试修改搜索关键词"或"点击右上角创建驱动盘"）
- 首次使用时显示引导提示

---

## 8. 优化路线图建议

按影响面和实施难度排序，建议分三个阶段：

### 阶段一：安全与稳定性（1-2 天）

| 编号 | 项目 | 优先级 |
|------|------|--------|
| 5.1 | 启用 CSP 策略 | 🔴 高 |
| 5.2 | 路径遍历防护 | 🔴 高 |
| 1.3 | API 输入验证增强 | 🟡 中 |
| 1.2 | ZON 解析器递归深度限制 | 🟡 中 |
| 5.3 | 输入消毒 | 🟡 中 |

### 阶段二：代码质量与可维护性（3-5 天）

| 编号 | 项目 | 优先级 |
|------|------|--------|
| 2.1 | 三面板代码重复消除 | 🔴 高 |
| 4.1 | 状态管理集中化 | 🔴 高 |
| 3.1 | theme.css 拆分 | 🔴 高 |
| 1.1 | DataManager 缓存改进 | 🟡 中 |
| 4.3 | 错误处理统一 | 🟡 中 |
| 2.4 | HadalPanel/PlayerPanel onActivated | 🟡 中 |
| 2.5 | 导入功能完善 | 🟡 中 |
| 2.2 | `(item as any)._i` 类型安全 | 🟡 中 |

### 阶段三：体验打磨（2-3 天）

| 编号 | 项目 | 优先级 |
|------|------|--------|
| 4.2 | IPC 调用批量化 | 🟡 中 |
| 7.1 | 键盘导航完善 | 🟡 中 |
| 3.2 | CSS 变量一致性 | 🟢 低 |
| 2.6 | DOM 操作改为 ref | 🟢 低 |
| 2.7 | setTimeout 清理 | 🟢 低 |
| 4.4 | 撤销栈实现 | 🟢 低 |
| 6.1 | Bundle 分析 | 🟢 低 |

---

## 附录：代码度量

| 指标 | 数值 |
|------|------|
| Rust 源文件 | 5 |
| Rust 总行数 | ~1,920 |
| Vue 组件 | 20 |
| Vue 总行数 | ~2,800 |
| CSS 总行数 | ~1,650 |
| IPC 命令数 | 31 |
| 前端 API 封装 | 27 |
| TypeScript 类型定义 | ~244 行 |
| 重复代码估计 | ~300 行（三面板） |
| CSS 变量数 | ~120 |
| 硬编码魔数 | ~8 处 |
