# Yoshunko Admin Rust 版 — 深度优化建议（三次修订版）

> 基于对项目全部源码的逐行审阅，每条建议均经源码交叉验证。
> 日期：2026-05-15 · 当前版本：V0.630
> 三次修订说明：基于 V0.630 代码变更全面复核，修正 5 处过时声明，新增 3 项已修复问题标注，补充新发现

---

## 符号说明

| 标记 | 含义 |
|------|------|
| ✅ 已验证 | 事实声明已对照源码确认 |
| ⚠️ 需注意 | 实施时可能影响现有行为，需额外测试 |
| 🟢 无风险 | 纯增量修改，不影响现有逻辑和 UI |
| 🟡 低风险 | 修改内部实现，外部行为不变，需回归测试 |
| 🔴 高风险 | 可能改变现有行为或 UI 表现，需仔细评估 |
| 🔵 高契合 | 与项目实际需求高度匹配，建议优先实施 |
| ⚪ 低契合 | 对当前项目收益有限，属于过度工程或远期考虑 |

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

### 1.1 DataManager 缓存策略改进

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `DataManager.cache` 类型为 `HashMap<PathBuf, BTreeMap<String, ZonValue>>`（[data_manager.rs:15](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/data_manager.rs#L15)）
- `read_zon_obj` 缓存命中时返回 `Some(cached.clone())`（[data_manager.rs:194](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/data_manager.rs#L194)）
- `write_zon` 写入后执行 `self.cache.insert(path, data.clone())`（[data_manager.rs:231](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/data_manager.rs#L231)）
- `with_manager` 每次调用都 `state.data_manager.lock()` 锁整个 DataManager（[api.rs:52](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L52)）

**问题**：
- 缓存命中时 clone 整个 BTreeMap，对于单个 ZON 文件（通常几十个字段）开销很小
- 缓存没有失效机制，如果外部程序修改了 ZON 文件，缓存与磁盘不同步
- ⚠️ 但实际上，当前应用是单用户桌面工具，外部修改文件的场景极少，且 `with_manager` 的 Mutex 在单线程 IPC 场景下不会产生锁争用

**建议**：
- 引入 `Arc<BTreeMap<String, ZonValue>>` 替代裸 `BTreeMap`，clone 只增加引用计数 → 🟢 无风险 · ⚪ 低契合：单用户桌面工具中 clone 开销可忽略，引入 Arc 增加复杂度但无实际性能收益
- 为缓存条目添加 mtime 校验，读取时比对文件修改时间 → 🟡 需确保 mtime 比对在 Windows 上可靠（NTFS 精度问题）· 🔵 高契合：外部修改检测对调试场景有实际价值
- `DashMap` 替代 `HashMap + Mutex` → 🟡 当前单线程 IPC 场景下收益有限 · ⚪ 低契合：引入新依赖，单线程无锁争用

### 1.2 ZON 解析器健壮性

**优先级**：中 · **风险**：🟡 低风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `tokenize` 函数执行 `let chars: Vec<char> = text.chars().collect()` 一次性分配（[zon.rs:64](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/zon.rs#L64)）
- 解析错误统一返回 `Result<_, String>`，无结构化错误类型（[zon.rs:63](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/zon.rs#L63)）
- `parse_brace` 通过 lookahead 区分数组 vs 结构体（[zon.rs:284-296](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/zon.rs#L284)）
- 递归解析无深度限制（`parse_value` → `parse_brace` → `parse_value` 无限递归）

**问题**：
- ZON 文件通常很小（每个角色/音擎几 KB），`chars().collect()` 的内存开销在实际场景中可忽略
- 无递归深度限制理论上可被恶意输入利用，但 ZON 文件只来自本地磁盘，攻击面极小
- 错误类型为 String 不影响功能，只影响调试体验

**建议**：
- 添加递归深度限制（如 max_depth = 64），超限返回错误 → 🟢 无风险 · 🔵 高契合：防御性编程，零成本
- 定义 `ZonError` 枚举实现 `std::error::Error` → 🟢 无风险 · ⚪ 低契合：改动面大，当前 String 错误类型对桌面工具够用
- 大文件场景优化（peekable 迭代器）→ 🟡 · ⚪ 低契合：ZON 文件通常几 KB，无需优化

### 1.3 API 层输入验证增强

**优先级**：中 · **风险**：🟡 低风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证，V0.630 更新）：
- `update_player_basic` 接受 `BTreeMap<String, ZonValue>` 并直接传给 `dm.update_basic_info`，无字段白名单（[api.rs:375-389](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L375)）
- ✅ **V0.630 已改进**：新增验证常量（`MIN_LEVEL`/`MAX_LEVEL`/`MIN_STAR`/`MAX_STAR`/`MIN_REFINE`/`MAX_REFINE`/`MIN_RANK`/`MAX_RANK`/`MIN_PASSIVE`/`MAX_PASSIVE`/`MIN_EQUIP_LEVEL`/`MAX_EQUIP_LEVEL`/`MAX_EQUIP_STAR`）和 `check_range` 函数（[api.rs:25-45](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L25)），avatar/weapon/equip 的 level/star/rank/refine/passive 等字段已有范围验证
- `nickname` 后端仍无任何验证（无长度限制、无非空检查、无字符过滤），前端有 `!editNickname.value.trim()` 非空检查但无 maxlength（[PlayerPanel.vue:136](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/panels/PlayerPanel.vue#L136)）
- `set_state_dir` 只检查 `player/` 子目录存在，不验证路径是否含 `..` 或是否为绝对路径（[api.rs:104-130](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L104)）
- `create_equip` / `update_equip` 的 `properties` 数组无长度限制
- `update_player_basic` 中 `exp` 和 `avatar_id` 的验证错误消息仍为英文（`"exp must be >= 0"`, `"avatar_id must be >= 0"`），与其他中文错误消息不一致
- ✅ **V0.630 已改进**：`update_avatar`/`update_weapon`/`update_equip`/`update_hadal_zone` 现在有 merge-with-existing 逻辑（先读取现有数据，覆盖更新字段，写入完整对象），避免了前端未发送的字段被丢失

**问题**：
- 前端可传入任意 key 写入 ZON 文件，可能写入游戏不识别的字段
- ⚠️ 但当前前端是唯一客户端，且 `update_player_basic` 传入的 key 都是硬编码的，不存在恶意输入场景
- `set_state_dir` 的路径由 SetupPanel 的目录选择器或手动输入提供

**建议**：
- 为 update 命令添加字段白名单，拒绝未知 key → 🟡 需确保白名单覆盖所有合法字段 · 🔵 高契合：防止写入游戏不识别的字段
- `nickname` 后端添加非空 + 长度限制 ≤ 64 字符 → 🟢 无风险 · 🔵 高契合
- `set_state_dir` 验证路径不含 `..` 且为绝对路径 → 🟢 无风险 · 🔵 高契合
- `properties` 数组长度限制 → 🟡 需确认游戏实际允许的最大长度 · 🔵 高契合
- 统一英文错误消息为中文 → 🟢 无风险 · 🔵 高契合
- ~~update 命令未保留未发送字段~~ → ✅ V0.630 已通过 merge-with-existing 修复

### 1.4 配置文件写入竞态

**优先级**：高 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证，V0.630 更新）：
- `set_state_dir`（[api.rs:104-130](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L104)）和 `set_launch_path`（[api.rs:731-754](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L731)）各自独立读取 → 修改 → 写入 `config.json`
- `get_launch_config`（[api.rs:722-728](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L722)）读取时不加锁

**问题**：
- 理论上并发调用可能丢失修改，但实际上这两个操作都是用户手动触发，不会并发
- ⚠️ **确认 Bug（仍存在）**：`set_state_dir` 会覆盖整个 config（只写 `state_dir` 和 `version`），如果之前有 `launch` 配置会被丢失
- ⚠️ **版本硬编码（部分修复）**：✅ V0.630 新增 `get_version` 命令（[api.rs:82-101](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L82)）从 `tauri.conf.json` 动态读取版本号。但 `get_config`（[api.rs:75](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L75)）和 `set_state_dir`（[api.rs:114](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L114)）仍硬编码 `"V0.700"`，与实际版本 V0.630 不一致

**建议**：
- `set_state_dir` 应保留现有 config 中的 `launch` 字段 → 🟡 修复现有 bug · 🔵 高契合
- `set_state_dir` 应读取现有 config 并只更新 `state_dir` 字段 → 🟢 无风险 · 🔵 高契合
- 统一配置读写到带锁的 `ConfigManager` → 🟢 无风险 · ⚪ 低契合：当前无并发场景，优先级低
- ✅ `get_version` 已动态读取版本号，`get_config` 和 `set_state_dir` 中的硬编码 `"V0.700"` 应改为调用 `get_version` 的格式化逻辑 → 🟢 无风险 · 🔵 高契合

### 1.5 审计日志改进

**优先级**：低 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `self.player_dir.join("..").join("audit.log")`（[data_manager.rs:273](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/data_manager.rs#L273)）
- 日志格式为纯文本 `[timestamp] WRITE path timezone`
- 轮换策略仅基于文件大小（>1MB），无时间维度

**建议**：
- 使用 `Path::parent()` 替代 `join("..")` → 🟢 无风险，语义更清晰
- 日志格式改为 JSON Lines → 🟡 改变日志格式，需确认无外部工具依赖当前格式
- 添加时间维度轮换 → 🟢 无风险

---

## 2. 前端 Vue 优化

### 2.1 三面板代码重复消除

**优先级**：高 · **风险**：🔴 高风险 · **契合**：🟡 中契合 · **验证**：✅

**现状**（已验证）：
- `AvatarsPanel.vue`（394 行）、`WeaponsPanel.vue`（253 行）、`EquipsPanel.vue`（670 行，含创建流程）存在以下重复模式：
  - `refreshCache()` — 检查 dirty → 调 API → 更新 cache → 设 loading=false
  - `selectXxx()` — 卡片点击动画 + 设置选中 + 加载编辑器 + slide-in
  - `backToGallery()` — 重置视图 + 选中项 + staggered animation
  - `saveXxx()` — 调 API → 检查 ok → toast → markClean → markCacheDirty → backToGallery
  - `onMounted` / `onActivated` / `watch(panel)` — 生命周期逻辑
  - `groupedXxx` computed — 分组 + 排序 + 分配 stagger index

**问题**：
- 重复代码约 200 行（不含 EquipsPanel 独有的创建流程），但三个面板的模板部分也有大量结构相似性（卡片网格、编辑器布局），实际重复远超 200 行
- 修改一处逻辑需同步修改三处

**建议**：
- 提取 `usePanelGallery<T>(options)` composable → 🔴 高风险：重构涉及三个面板的核心逻辑，需全面回归测试每个面板的每个操作流程
- 提取 `useCardAnimation()` composable → 🟡 低风险：仅封装动画逻辑
- ⚠️ EquipsPanel 有创建流程（3 步向导）和删除功能，与其他两个面板差异较大，composable 需设计好扩展点

### 2.2 `(item as any)._i` 类型安全

**优先级**：中 · **风险**：🟡 低风险 · **契合**：🟡 中契合 · **验证**：✅

**现状**（已验证）：
- 三个面板的 `groupedXxx` computed 中均使用 `(item as any)._i = idx++`（AvatarsPanel:98, WeaponsPanel:57, EquipsPanel:101）
- 模板中使用 `:style="{ '--i': (a as any)._i }"`

**问题**：
- `as any` 绕过类型检查
- 修改了从 API 返回的响应式对象

**建议**：
- 定义 `StaggeredItem<T>` 包装类型 → 🟡 需修改三个面板的 computed 和模板，但外部行为不变
- 使用 `WeakMap` 存储索引 → 🟡 更安全但实现更复杂
- ⚠️ 不建议在模板中用 `$index` + 偏移计算，因为分组内索引需要跨组连续

### 2.3 硬编码过滤 ID

**优先级**：中 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `AvatarsPanel.vue:43` — `avatar_id !== 2071 && avatar_id !== 2121`
- `WeaponsPanel.vue:25` — `w.id < 12000 || w.id > 12999`

**问题**：
- 魔数含义不明
- 游戏版本更新后可能需要调整

**建议**：
- 提取为命名常量并添加注释 → 🟢 无风险，纯重构
- 考虑移到后端过滤 → 🟡 需修改 API 返回格式，前端需适配

### 2.4 HadalPanel / PlayerPanel 缺少 `onActivated`

**优先级**：高 · **风险**：🟡 低风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `HadalPanel.vue` 只 import 了 `onMounted`（[HadalPanel.vue:2](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/panels/HadalPanel.vue#L2)），无 `onActivated`
- `PlayerPanel.vue` 只 import 了 `onMounted`（[PlayerPanel.vue:2](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/panels/PlayerPanel.vue#L2)），无 `onActivated`
- `MainContent.vue` 使用 `<KeepAlive>`（[MainContent.vue:97](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/layout/MainContent.vue#L97)）

**问题**：
- 切换 uid 后再切回这两个面板，数据不会刷新——这是**确认的 Bug**
- `MainContent.vue` 的 `watch(uid)` 会清空 avatarCache/weaponCache/equipCache 并设 `cacheDirty = true`，但 HadalPanel 和 PlayerPanel 不使用这三个缓存，它们各自独立调 API 且不检查 `cacheDirty`
- HadalPanel 和 PlayerPanel 的数据与 uid 直接关联（如 `api.getHadalZone(uid)`），uid 变化后旧数据属于不同玩家

**建议**：
- 添加 `onActivated` 钩子重新加载数据 → 🟡 低风险 · 🔵 高契合：修复确认 Bug，但需注意每次切回都会重新请求 API。推荐实现：监听 `uid` 变化时才重新加载，而非每次 `onActivated` 都刷新
- 与其他三个面板保持一致的生命周期模式

### 2.5 导入功能不完整

**优先级**：中 · **风险**：🟡 低风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `PlayerPanel.vue` 的 `importData()` 只导入 `json.info`（[PlayerPanel.vue:106](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/panels/PlayerPanel.vue#L106)）
- `exportData()` 导出完整数据：info + avatars + weapons + equips + hadal_zone（[PlayerPanel.vue:61-77](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/components/panels/PlayerPanel.vue#L61)）

**问题**：
- 导出文件包含完整数据，但导入只恢复 info 部分
- 用户可能期望导入是导出的逆操作
- ⚠️ 但当前仅导入 info 是**保守但安全**的策略：完整导入需逐个调用 update API，若中间某步失败会导致数据部分恢复、部分未恢复的不一致状态

**建议**：
- 完整实现导入 → 🟡 需逐个调用 update API，需添加进度提示和错误处理
- 后端新增 `batch_update` 命令 → 🟡 需修改 Rust 后端和前端 API 层

### 2.6 `document.querySelector` 直接 DOM 操作

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `AvatarsPanel.vue:126` — `document.querySelector('.main-content')`
- `WeaponsPanel.vue:103` — `document.querySelector('.main-content')`
- `EquipsPanel.vue:156` — `document.querySelector('.main-content')`
- `useStaggeredAnimation.ts:7` — `document.querySelector('.main-content')`

**问题**：
- 硬编码 CSS 选择器，重构时容易遗漏
- ⚠️ 但 `.main-content` 是 `MainContent.vue` 中的 `ref="mainRef"` 元素，当前不会改名

**建议**：
- 通过 `provide/inject` 传递 DOM 引用 → 🟡 需修改 MainContent 和所有面板，但行为不变
- 或将 `editor-slide-in` 改为纯 CSS 方案 → 🟡 需重新设计动画触发机制

### 2.7 `setTimeout` 未清理

**优先级**：低 · **风险**：🟢 无风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `AvatarsPanel.vue:119` — `setTimeout(() => { el.style.transform = 'scale(1)' }, 120)`
- `WeaponsPanel.vue:96` — 同上
- `EquipsPanel.vue:149` — 同上
- `GameCard.vue:26` — 已正确处理 `onUnmounted(() => { if (pressTimer) clearTimeout(pressTimer) })`

**问题**：
- 如果用户在 120ms 内切换面板，回调可能在已卸载组件的 DOM 元素上执行
- ⚠️ 但实际上 120ms 极短，用户几乎不可能在此时差内切换面板，且 `el` 引用的 DOM 元素即使已卸载，设置 `style.transform` 也不会报错

**建议**：
- 提取为 `usePressAnimation()` composable 并正确清理 timer → 🟢 无风险

---

## 3. CSS / 设计系统优化

### 3.1 theme.css 体量控制

**优先级**：低 · **风险**：🔴 高风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `theme.css` 共 1729 行
- 包含设计系统变量、组件样式、布局样式、动画、响应式、暗色主题覆盖、合并段
- 合并段起始于第 1588 行 `/* Merged from vue-extras.css */`

**建议**：
- 拆分为多个 CSS 文件 → 🔴 高风险 · ⚪ 低契合：CSS 拆分可能改变级联顺序，导致样式覆盖关系变化。1729 行对当前项目规模可管理，拆分风险大于收益
- ⚠️ 如果拆分，必须确保 `@import` 顺序与当前单文件中的声明顺序一致
- 建议先做不改变级联顺序的拆分（如提取 `@keyframes` 到独立文件），再逐步拆分组件样式

### 3.2 CSS 变量一致性

**优先级**：低 · **风险**：🟡 低风险 · **契合**：🟡 中契合 · **验证**：✅（含修正）

**现状**（已验证）：
- `--transition-bounce` 定义于 [theme.css:106](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/styles/theme.css#L106)，全项目搜索 **0 次使用** → 确认未使用 ✅
- `--z-sticky` / `--z-dropdown` 定义于 [theme.css:158-159](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/styles/theme.css#L158)，全项目搜索 **0 次使用** → 确认未使用 ✅
- ~~`--font-display` 未使用~~ → **修正**：`--font-display` 使用了 3 次（theme.css:278/424/1144），只是值等于 `--font`（[theme.css:99](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/styles/theme.css#L99) `--font-display: var(--font)`）
- `--text-xs` (10px) 使用 2 次，`--text-sm` (11px) 使用 5 次 → 两者都有实际使用
- `.rarity-s` 使用 `background: #c49a2e`（[theme.css:502](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/styles/theme.css#L502)），而 `--rarity-s: #f0c04e` → **修正**：`#c49a2e` 是深金色（徽章背景），`#f0c04e` 是亮金色（边框/发光），这是**有意区分**而非遗漏。border 使用 `rgba(240,192,78,0.5)` 即 `--rarity-s` 的半透明版本

**建议**：
- 删除 `--transition-bounce`、`--z-sticky`、`--z-dropdown` → 🟢 无风险
- `--font-display` 保留但考虑未来赋予差异化值 → 🟢 无风险
- ⚠️ 不建议合并 `--text-xs` 和 `--text-sm`，两者都有实际使用场景
- ⚠️ 不建议将 `.rarity-s` 的 `#c49a2e` 改为 `var(--rarity-s)`，这是有意区分的深色版本

### 3.3 响应式断点补充

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- 最小窗口宽度 960px（[tauri.conf.json:18](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/tauri.conf.json#L18)）
- CSS 中有 1100px 和 960px 两个断点

**建议**：
- 添加超宽屏断点 → 🟢 无风险
- 侧边栏折叠为图标模式 → 🟡 需修改 Sidebar 组件和布局 CSS

---

## 4. 架构与数据流优化

### 4.1 状态管理集中化

**优先级**：中 · **风险**：🔴 高风险 · **契合**：🟡 中契合 · **验证**：✅

**现状**（已验证）：
- `useAppState.ts` 导出 31 个独立 ref/computed/函数，面板组件直接 import 使用
- `dirty` 是全局的（[useAppState.ts:40](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/composables/useAppState.ts#L40)），但实际应该是每面板独立的
- `cacheDirty` 是全局的（[useAppState.ts:14](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/composables/useAppState.ts#L14)），三个面板的缓存共享此标记

**问题**：
- `cacheDirty` 全局共享意味着：角色面板标记 dirty 后，切到音擎面板也会重新加载，即使音擎缓存并未失效
- `dirty` 全局共享意味着：在角色编辑器修改数据后，切换到音擎面板时 `dirty` 仍为 true（但音擎面板的 `watch(panel)` 已重置了视图，未保存的修改实际上已丢失，`dirty` 标记已不准确）
- ⚠️ 但 `cacheDirty` 全局共享在部分场景下是**期望行为**：保存角色数据后（`markCacheDirty()`），切到音擎面板刷新可确保关联数据一致性（如角色的 `cur_weapon_uid` 可能影响音擎显示）

**建议**：
- 将 `cacheDirty` 拆分为 `avatarCacheDirty` / `weaponCacheDirty` / `equipCacheDirty` → 🟡 低风险，需修改所有使用 `cacheDirty` 的地方
- 将 `dirty` 拆分为每面板独立 → 🟡 低风险，需修改 `useKeyboard.ts` 的保存逻辑
- ⚠️ 不建议大规模重构为多个 store，当前单文件模式虽不完美但可维护

### 4.2 IPC 调用批量化

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `MainContent.vue:48` 的 `loadCounts` 使用 `Promise.all` 并行发 3 个 IPC 调用
- `PlayerPanel.vue:61` 的 `exportData` 使用 `Promise.all` 并行发 5 个 IPC 调用

**问题**：
- `Promise.all` 已经是并行的，IPC 开销主要在序列化/反序列化
- ⚠️ 实际上 3 个并行 IPC 调用的延迟约等于 1 个最慢的调用，批量化收益有限

**建议**：
- 后端新增 `get_all_counts` 命令 → 🟡 需修改 Rust 后端，但可减少序列化开销
- 后端新增 `export_player` 命令 → 🟡 同上
- 优先级低，当前并行调用已足够高效

### 4.3 错误处理统一

**优先级**：低 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- 前端错误处理模式统一为 `toast(e instanceof Error ? e.message : 'xxx失败', 'error')`
- 后端错误消息中英混杂：`'状态目录未配置'`（中文）vs `'Failed to write config'`（英文）vs `'exp must be >= 0'`（英文）

**建议**：
- 后端错误消息统一为中文 → 🟢 无风险，面向中文用户
- 提取 `asyncWrap(fn, errorMsg)` 工具函数 → 🟢 无风险

### 4.4 撤销栈实现

**优先级**：低 · **风险**：🟡 低风险 · **契合**：🟡 中契合 · **验证**：✅

**现状**（已验证）：
- `pushUndo` / `popUndo` 已定义（[useAppState.ts:85-92](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/composables/useAppState.ts#L85)）
- 三个编辑面板**均未调用** `pushUndo` → Ctrl+Z 只会 pop 空栈，无实际效果
- `useKeyboard.ts:45-49` 注册了 Ctrl+Z 并调用 `popUndo()`

**建议**：
- 在每个面板的 `saveXxx` 之前调用 `pushUndo` → 🟡 需仔细设计快照内容，确保 `restore()` 能正确恢复编辑器状态
- 撤销栈改为面板独立 → 🟡 需修改 `useKeyboard.ts` 的 undo 逻辑

---

## 5. 安全加固

### 5.1 CSP 策略

**优先级**：中 · **风险**：🟡 低风险 · **契合**：🟡 中契合 · **验证**：✅

**现状**（已验证）：
- `tauri.conf.json:27` — `"csp": null`，完全禁用内容安全策略

**问题**：
- WebView 可以加载任意外部资源
- ⚠️ 但 Tauri v2 的 WebView 运行在本地 `tauri://` 协议下，不像浏览器那样加载不可信网页，XSS 攻击面有限
- 游戏数据中的 nickname 等字段通过 Vue 模板渲染（`{{ }}` 自动转义），不会执行 HTML/JS

**建议**：
- 设置合理的 CSP → 🟡 需测试确保不破坏现有功能：
  ```json
  "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'"
  ```
- ⚠️ `unsafe-inline` 是 Tailwind CSS 所需，禁止 `unsafe-eval` 和外部脚本
- ⚠️ 设置 CSP 后需全面测试，特别是快速启动功能（可能用到 `shell:allow-open`）

### 5.2 路径遍历防护

**优先级**：高 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `set_state_dir`（[api.rs:104](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/api.rs#L104)）只检查 `player/` 子目录存在，不验证路径

**建议**：
- 验证路径不含 `..` → 🟢 无风险
- 验证路径是绝对路径 → 🟢 无风险
- ⚠️ 不建议验证路径不在系统目录下，因为用户可能有意将数据放在非标准位置

### 5.3 输入消毒

**优先级**：中 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `nickname` 等字符串字段直接写入 ZON 文件
- ZON 序列化器对字符串中的引号和换行符有转义处理（[zon.rs:370](file:///d:/3.0.1/tools/yoshunko-admin-rust/src-tauri/src/zon.rs#L370)）

**建议**：
- 字符串长度限制 → 🟢 无风险
- 过滤控制字符 → 🟢 无风险

### 5.4 capabilities 权限最小化

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `default.json` 包含 `shell:default` + `shell:allow-open`
- `shell:allow-open` 用于快速启动功能（`launch_program`、`launch_program_admin`、`launch_yoshunko`）

**建议**：
- ⚠️ `shell:allow-open` 是快速启动功能所必需的，不能简单移除
- 可考虑使用更细粒度的 shell 权限 → 🟡 需确认 Tauri v2 是否支持

---

## 6. 构建与部署优化

### 6.1 前端 bundle 分析

**优先级**：低 · **风险**：🟢 无风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**：Vite 配置了 `manualChunks` 分离 pinyin-data 和 naive-ui。

**建议**：
- 添加 `rollup-plugin-visualizer` → 🟢 无风险
- ⚠️ naive-ui 通过 `unplugin-vue-components` + `NaiveUiResolver` 已实现按需引入，tree-shaking 应已生效

### 6.2 Rust 编译优化

**优先级**：低 · **风险**：🟡 低风险 · **契合**：⚪ 低契合 · **验证**：✅

**现状**（已验证）：
- `Cargo.toml` release profile：`strip = true`、`lto = true`、`codegen-units = 1`、`opt-level = "s"`

**建议**：
- `opt-level = "z"` → 🟡 可能略微增加编译时间，体积收益有限
- `panic = "abort"` → 🟡 ⚠️ 会改变 panic 行为：当前 `unwrap()` 等 panic 会打印调用栈并 abort，添加后直接 abort 无调用栈。对于桌面工具，保留调用栈有助于调试用户反馈的崩溃问题，**不建议添加**

### 6.3 CI/CD 增强

**优先级**：低 · **风险**：🟢 无风险 · **契合**：⚪ 低契合 · **验证**：✅

**建议**：
- 添加 `cargo clippy` 和 `vue-tsc --noEmit` 到 CI → 🟢 无风险

---

## 7. 可访问性与 UX 细节

### 7.1 键盘导航完善

**优先级**：中 · **风险**：🟢 无风险 · **契合**：🔵 高契合 · **验证**：✅

**现状**（已验证）：
- `useKeyboard.ts` 注册了全局快捷键，但**没有实现 Ctrl+F**（[useKeyboard.ts](file:///d:/3.0.1/tools/yoshunko-admin-rust/src/composables/useKeyboard.ts) 全文无 `f` 键或 `search` 相关处理）
- `ShortcutsPanel.vue:23` 列出了 `Ctrl+F: 聚焦当前面板搜索框`，但实际未实现
- 卡片有 `tabindex="0"` 和 `@keydown.enter` / `@keydown.space`
- 创建驱动盘模态框使用 `<Teleport to="body">` + `@click.self`，无焦点陷阱

**建议**：
- 实现 Ctrl+F 聚焦搜索框 → 🟢 无风险
- 添加 `.game-card:focus-visible` 样式 → 🟢 无风险
- 模态框焦点陷阱 → 🟡 需确认当前模态框是否已有键盘可访问性

### 7.2 加载状态优化

**优先级**：低 · **风险**：🟢 无风险 · **契合**：🟡 中契合 · **验证**：✅

**建议**：
- 保存操作添加 loading 状态防止重复点击 → 🟢 无风险

### 7.3 空状态引导

**优先级**：低 · **风险**：🟢 无风险 · **契合**：🟡 中契合 · **验证**：✅

**建议**：
- 提供操作建议 → 🟢 无风险

---

## 8. 优化路线图建议

### 阶段一：Bug 修复与安全加固（1-2 天）

| 编号 | 项目 | 风险 | 契合度 | 说明 |
|------|------|------|--------|------|
| 1.4 | set_state_dir 保留 launch 配置 + 版本号去硬编码 | 🟡 | 🔵 | 确认 Bug（仍存在），数据丢失风险 |
| 2.4 | HadalPanel/PlayerPanel onActivated | 🟡 | 🔵 | 确认 Bug（仍存在），uid 切换后数据不刷新 |
| 5.2 | 路径遍历防护 | 🟢 | 🔵 | 纯增量验证逻辑 |
| 5.3 | 输入长度限制 + nickname 非空 | 🟢 | 🔵 | 纯增量 |
| 1.2 | ZON 递归深度限制 | 🟢 | 🔵 | 纯增量防护 |
| 2.3 | 硬编码 ID 提取为常量 | 🟢 | 🔵 | 纯重构 |
| 7.1 | 实现 Ctrl+F | 🟢 | 🔵 | ShortcutsPanel 已声明但未实现 |
| 4.3 | 后端错误消息统一中文 | 🟢 | 🔵 | 纯文本替换（含 `exp must be >= 0` 等） |
| 1.5 | 审计日志 join("..") 改 parent() | 🟢 | 🔵 | 纯重构 |

### 阶段二：体验改善与功能补全（2-3 天）

| 编号 | 项目 | 风险 | 契合度 | 说明 |
|------|------|------|--------|------|
| 2.5 | 导入功能完善 | 🟡 | 🔵 | 需处理部分失败场景 |
| 4.1 | cacheDirty 拆分 | 🟡 | 🟡 | 需修改多个文件，但当前全局共享部分场景为期望行为 |
| 5.1 | 启用 CSP | 🟡 | 🟡 | 需全面测试，攻击面有限 |
| 1.1 | 缓存 mtime 校验 | 🟡 | 🔵 | 对调试场景有实际价值 |
| 1.3 | update 命令字段白名单 | 🟡 | 🔵 | 需确保白名单完整 |

### 阶段三：代码质量提升（3-5 天，需充分回归测试）

| 编号 | 项目 | 风险 | 契合度 | 说明 |
|------|------|------|--------|------|
| 2.1 | 三面板代码重复消除 | 🔴 | 🟡 | 核心逻辑重构，需逐面板回归；EquipsPanel 差异大，composable 设计需谨慎 |
| 3.1 | theme.css 拆分 | 🔴 | ⚪ | CSS 级联顺序敏感，1729 行对当前规模可管理 |
| 2.2 | `(item as any)._i` 类型安全 | 🟡 | 🟡 | 模板和 computed 修改 |
| 4.4 | 撤销栈实际接入 | 🟡 | 🟡 | 需设计快照策略 |

### V0.630 已修复的问题

| 编号 | 原问题 | 修复方式 |
|------|--------|----------|
| 1.3 | 后端无输入范围验证 | 新增 `check_range` + 13 个验证常量 |
| 1.3 | update 命令未保留未发送字段 | 新增 merge-with-existing 逻辑 |
| 1.4 | 版本号完全硬编码 | 新增 `get_version` 命令动态读取（`get_config`/`set_state_dir` 仍硬编码） |

### 暂不建议实施

| 编号 | 项目 | 原因 |
|------|------|------|
| 1.1 | Arc 缓存 / DashMap | 单用户桌面工具，clone 开销可忽略，引入新依赖无实际收益 |
| 1.2 | ZonError 枚举 | 改动面大，当前 String 错误类型对桌面工具够用 |
| 4.2 | IPC 调用批量化 | Promise.all 已并行，批量化收益极低 |
| 6.2 | opt-level = "z" | 体积收益有限，编译时间增加 |

---

## 附录 A：事实修正记录

### 初版修正（6 处）

| 编号 | 初版声明 | 修正后 | 依据 |
|------|----------|--------|------|
| 3.2 | `--font-display` 定义了但未使用 | 实际使用了 3 次（theme.css:278/424/1144），值等于 `--font` | `grep -c "var(--font-display)"` = 3 |
| 3.2 | `.rarity-s` 使用 `#c49a2e` 未使用 `--rarity-s` | `#c49a2e` 是深金色（徽章背景），`--rarity-s: #f0c04e` 是亮金色，有意区分 | border 使用 `rgba(240,192,78,0.5)` 即 `--rarity-s` 半透明 |
| 3.2 | `--text-xs` 和 `--text-sm` 差距过小常混用 | 两者分别使用 2 次和 5 次，有各自使用场景 | `grep -c` 验证 |
| 3.1 | theme.css 1650 行 | 实际 1729 行 | PowerShell `(Get-Content).Count` |
| 6.2 | 建议添加 `panic = "abort"` | 不建议添加，会丢失 panic 调用栈，影响用户崩溃反馈调试 | 桌面工具需保留调试信息 |
| 4.2 | 导出功能 5 个调用串行等待 | 实际使用 `Promise.all` 并行等待 | PlayerPanel.vue:61 |

### 二次修订修正（7 处）

| 编号 | 修订版声明 | 修正后 | 依据 |
|------|-----------|--------|------|
| 3.1 | theme.css 1710 行 | 实际 1729 行 | PowerShell `(Get-Content).Count` 再次确认 |
| 2.1 | AvatarsPanel.vue 254 行 | 实际 394 行（含 template） | PowerShell 计数 |
| 2.1 | EquipsPanel.vue 430 行 | 实际 670 行（含 template + 创建向导） | PowerShell 计数 |
| 附录B | Rust 源文件 5 个 | 实际 6 个（含 main.rs） | Glob 确认 |
| 附录B | api.rs 917 行 | 实际 916 行 | PowerShell 计数 |
| 附录B | Vue 组件 20 个 | 实际 21 个 | Glob 确认 |
| 1.4 | 未提及版本硬编码问题 | `get_config` 和 `set_state_dir` 硬编码 `"V0.700"`，与实际版本 V0.629 不一致 | api.rs:75 和 api.rs:114 |

### 三次修订修正（5 处）

| 编号 | 二次修订声明 | 修正后 | 依据 |
|------|-------------|--------|------|
| 全文 | 版本 V0.629 | V0.630 | tauri.conf.json `0.630.0` |
| 1.3 | 后端无验证常量 | V0.630 新增 `check_range` + 13 个验证常量 | api.rs:25-45 |
| 1.3 | update 命令未保留未发送字段 | V0.630 已通过 merge-with-existing 修复 | api.rs:485/563/642/706 |
| 1.4 | 版本号完全硬编码 | V0.630 新增 `get_version` 动态读取，但 `get_config`/`set_state_dir` 仍硬编码 | api.rs:82-101 |
| 附录B | api.rs 916 行 / IPC 命令 31 | api.rs 977 行 / IPC 命令 32 | PowerShell 计数 / lib.rs 注册列表 |

## 附录 B：代码度量

| 指标 | 数值 | 验证方式 |
|------|------|----------|
| Rust 源文件 | 6 | LS 确认（含 main.rs 6 行入口） |
| Rust 总行数 | ~1,993 | api.rs 977 + data_manager.rs 289 + zon.rs 393 + template_loader.rs 222 + lib.rs 106 + main.rs 6 |
| Vue 组件 | 21 | Glob 确认 |
| CSS 总行数 | 1,729 | PowerShell 计数 |
| IPC 命令数 | 32 | lib.rs 注册列表（V0.630 新增 `get_version`） |
| 未使用 CSS 变量 | 3 | `--transition-bounce`、`--z-sticky`、`--z-dropdown` |
| 硬编码魔数 | 2 处 | avatar_id 2071/2121、weapon id 12000-12999 |
| 硬编码版本号 | 2 处 | `get_config` 和 `set_state_dir` 中的 `"V0.700"`（`get_version` 已动态读取） |
