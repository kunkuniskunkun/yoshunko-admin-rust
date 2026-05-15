# Yoshunko Admin Rust 版修改日志

> 基于 Python 版 (yoshunko-admin-python) 完全重写的 Tauri v2 原生桌面应用。版本号与 Python 版同步。

---

## V0.629 (2026-05-15)

### 驱动盘业务完全修复 — 主属性/副属性选项与基础值

**Bug 修复**
- 修复主属性选项错误：4号位移除穿透率（23103），5号位改为穿透率+元素伤害加成（移除暴击率/暴击伤害/异常精通/能量自动回复），6号位改为异常掌控+冲击力+能量自动回复（移除其他所有）
- 修复主属性选项缺少 base_value 字段：现在返回正确的 base_value，前端不再使用硬编码的错误数值
- 修复副属性选项缺少 base_value 字段：现在返回正确的 base_value（如暴击率=240、暴击伤害=480等）
- 修复创建驱动盘时副属性 base_value 未自动填充：选择副属性后自动设置正确的 base_value
- 修复编辑驱动盘时副属性 add_value 默认值错误：新选择的副属性默认 add_value=1（无强化），而非 0

**数据对齐**
- 主属性选项完全对齐 Python 版 MAIN_STAT_OPTIONS
- 副属性选项完全对齐 Python 版 SUB_STAT_OPTIONS
- 前端 MAIN_STAT_BASE_VALUES 完全对齐 Python 版数值

---

## V0.628 (2026-05-15)

### 式舆防卫战布局重构 + 导航栏图标优化 + 快速启动名称调整

**UI 改进**
- 式舆防卫战页改为全覆盖样式（settings-panel），四个入口各自成列排列
- 导航栏五个选项的图标添加 hover 动画：放大15% + 旋转8度，与设置/主题切换按钮风格一致
- 暗色主题时导航栏上方的 Logo 图标不再跟随主题转换颜色

**名称调整**
- 快速启动页的"Yidhari Client"改为"Client"

---

## V0.627 (2026-05-15)

### 式舆防卫战布局 + 技能名修复 + 驱动盘副词条修正 + 玩家信息页重构

**UI 改进**
- 式舆防卫战页面：卡片改为2列并排布局，增大卡片尺寸（min-height: 120px），间距从10px增至16px
- 导航栏图标放大8%（20→22）
- 玩家信息页改为全覆盖样式（settings-panel），分段显示：基本信息、角色展示、数据管理

**Bug 修复**
- 修复角色编辑页技能名全部显示为"普攻"：ZON 枚举类型（如 `.common_attack`）未被正确解析为字符串，修改 `ZonValue::as_str()` 支持枚举类型
- 修复驱动盘副词条包含"穿透率"：穿透率（23103）只能作为主属性，副属性应为穿透值（23203），已从 sub_stat_options 移除穿透率

**数据说明**
- 音擎仓库中的"未知"项：部分音擎 ID（如 13018、14004）在 weapon_names_zh.json 中已有中文名称，若仍显示为"Weapon_ID"格式，可能是用户数据中的 ID 不在模板范围内

---

## V0.626 (2026-05-15)

### 关闭按钮修复 + 副属性显示调整 + Toast 位置优化

**Bug 修复**
- 修复标题栏关闭按钮（X）无法关闭程序：移除 App.vue 的 `onCloseRequested` 监听器，改为在 TitleBar 中直接处理关闭逻辑（检查 dirty 状态 → 显示对话框 → 关闭窗口）

**UI 改进**
- Toast 弹窗位置从 `top: 16px` 调整为 `top: 50px`，避免遮挡标题栏下方内容
- 副属性"属性"列与"强化次数"列间距从 8px 增大到 12px
- 副属性"强化次数"列 flex 从 4 增大到 5，进一步右移对齐

**显示逻辑调整**
- 副属性强化次数显示值-1：存储值包含基础词条，显示时减1以匹配游戏内"+N"的显示方式（如存储值3显示为2）
- 创建驱动盘的强化次数显示同步调整
- 强化总和计算同步调整（基于显示值求和）

---

## V0.625 (2026-05-15)

### 关闭按钮修复 + 副属性列对齐

**Bug 修复**
- 修复标题栏关闭按钮（X）无法关闭程序：dialog 插件缺少 capabilities 权限导致 `ask()` 抛异常，`preventDefault()` 后窗口永远无法关闭。添加 `dialog:default` + `dialog:allow-ask` 权限，并加 try-catch 兜底

**UI 改进**
- 副属性"属性"列加宽（flex: 3）、"强化次数"列对齐（flex: 4），header 与 row gap 统一为 8px

---

## V0.624 (2026-05-15)

### 副属性编辑器布局 + 面板切换滚动修复

**UI 改进**
- 驱动盘编辑页副属性：缩短"属性"列、加宽"强化次数"列、增大加粗强化次数字体（14px/600 → 16px/700）

**Bug 修复**
- 修复切换面板时滚动位置继承：从角色仓库滚动到中间后切到音擎仓库会停在相同位置，现在切换面板自动回到顶部

---

## V0.623 (2026-05-15)

### 未保存提醒 + GitHub Actions CI

**新功能**
- 关闭窗口时若有未保存更改，弹出确认对话框（Tauri dialog plugin）
- 三个编辑面板（角色/音擎/驱动盘）跟踪编辑状态，保存后自动清除
- GitHub Actions CI：推送 `v*` tag 时自动构建 Windows 安装包并创建 Release

**依赖变更**
- 新增 `tauri-plugin-dialog` 2.x（Rust + JS）
- 新增 `.github/workflows/build.yml`

---

## V0.622 (2026-05-15)

### 快速启动面板 UX 改进

**Bug 修复**
- 配置路径时自动去除首尾双引号（如粘贴 `"D:\path\to\exe"` 会自动保存为 `D:\path\to\exe`）

**UI 改进**
- 三件套（HoyoSDK、KCPSHIM、Yidhari）按钮统一显示"配置"（原为已配置时显示"编辑"）
- 未配置项：红色左侧边条 + 红色状态点
- 已配置项：绿色左侧边条 + 绿色状态点

---

## V0.621 (2026-05-15)

### 快速启动修复 — Windows API 重写

**Bug 修复**
- 修复所有快速启动业务无法正常运行：`launch_program` 和 `launch_yoshunko` 添加 `CREATE_NEW_CONSOLE` 标志，子进程获得独立控制台窗口
- `launch_program_admin` 从 PowerShell `Start-Process -Verb RunAs` 改为直接调用 `ShellExecuteW` Win32 API（`runas` 动词），对齐 Python 版实现
- `launch_yoshunko` 改进 WSL 路径解析：统一使用 `/` 分隔符，通过 `wsl.localhost` 定位发行版名称

**依赖变更**
- 新增 `windows-sys` 0.59（Win32 Foundation + Shell + WindowsAndMessaging）

---

## V0.620 (2026-05-15)

### 性能优化 — 后端缓存 + 前端动画 + 构建优化

**后端优化**
- DataManager 新增内存缓存层：`read_zon_obj` 先查 HashMap 缓存，命中直接返回；`write_zon` 写入后同步更新缓存；`delete_equip` 后清除缓存条目
- 消除 `write_zon` 中的 `data.clone()`：新增 `serialize_zon_object` 直接序列化 `&BTreeMap`，避免每次保存时完整克隆数据树
- `get_templates` 结果缓存到 `OnceLock`：模板数据运行期间不变，首次计算后存入，后续调用直接返回
- `with_manager` 改为传递 `&mut DataManager` 以支持缓存写入

**前端优化**
- 交错动画从 JS RAF 改为纯 CSS `animation-delay`：删除 `useStaggeredAnimation` 中的 `setTimeout` + 强制回流 + `requestAnimationFrame` 逻辑，改用 CSS `@keyframes cardEnter` + `.staggered-anim` 类，浏览器合成器线程处理动画
- 修复 Sidebar watcher 的无效 `deep: true`：`shallowRef` 只跟踪 `.value` 引用替换，`deep: true` 是多余的深度遍历
- Vite `manualChunks` 配置：分离 `pinyin-data` (11KB) 和 `naive-ui` (337KB) 为独立 chunk，不常变的内容可长期缓存

**布局修复**
- 修复标题栏占满半屏：`position: fixed` + `calc(100vh - 36px)` 绕开 naive-ui wrapper div 的 flex 高度链问题

**Bug 修复**
- 修复快速启动全部业务不可用：`launch_program` 改为后端根据 key 查找路径并设置 cwd；`get_launch_config` 返回值包装 `{"config": ...}` 匹配前端；`launch_yoshunko` 从 `state_dir` 提取 WSL 路径不再硬编码；修复 `kcpsim` → `kcpshim` 拼写
- 修复角色/音擎仓库偶尔不显示卡片：uid 切换时先清空缓存防止旧数据短路；`onActivated` 先 `await refreshCache()` 再 `nextTick` 触发动画
- 删除驱动盘副属性编辑中的"基础值"列，仅保留属性选择和强化次数

---

## V0.619 (2026-05-15)

### 关键布局修复 — 滚动 + 侧边栏 + 驱动盘编辑器

**Bug 修复**
- 修复三个仓库页面无法滚动：naive-ui wrapper div 阻断了 flex 高度链，添加 CSS 规则使 5 层 wrapper 参与 flex 布局
- 修复侧边栏底部按钮（主题切换/设置）在角色仓库和音擎仓库不显示：移除 `.sidebar` 的 `overflow-y: auto`，改由 `.sidebar-nav` 单独处理滚动
- 修复驱动盘编辑器和创建页强化次数数字显示不清：stepper input 从 inline 40px 改为使用 `.input-stepper` 类 (50px)
- 创建驱动盘时主属性基础值自动填充：添加 ZZZ 驱动盘主属性默认基础值查找表，选择主属性/槽位后自动填入

---

## V0.618 (2026-05-14)

### CSS 合并修复 + 包体优化 #8-9 + 构建脚本改进

**Bug 修复**
- 修复 CSS 合并缺失 `flex-direction: column` 导致标题栏窗口控制按钮跑到左侧（`.app-layout` 规则）
- 修复 sidebar 底部按钮（设置/主题切换）被裁剪不可见：`.sidebar-nav` 补上 `flex: 1; overflow-y: auto;`
- 修复快速启动卡片侧边颜色不显示：添加 `launch-card--ready` 条件类
- 修复切换面板后搜索框保留上次输入：离开面板时重置 `searchQuery`
- 消除 CSS 构建警告：`@media` 内 CSS 变量声明包裹 `:root`

**包体优化**
- \#8 拼音数据后端化 — 跳过（复杂度高，仅省 17 KB，性价比不足）
- \#9 发布版移除 `devtools` feature，减少攻击面

**构建脚本**
- 重写 `build-run.ps1`：自动杀进程、创建目录、显示版本号和耗时、自动加入 Node.js PATH
- 修复闪退问题：移除 `$ErrorActionPreference="Stop"`，改用 try/catch
- 删除旧版 `build.ps1`，统一使用 `build-run.ps1`

---

## V0.617 (2026-05-15)

### 8 项 Bug 修复 + 4 项追加修复 + 回归 Python 版 UI

**UI 全面回归 Python 版设计语言**
- theme.css 完全重写：浅色主题、Consolas 字体、干净卡片设计
- 侧边栏：大图标 (85px)、白色渐变背景、蓝色左侧 accent 条指示导航
- 标题栏：36px 高度、简洁品牌标识、版本号动态读取
- 强调色回归 Python 版蓝色 (#4a9fd8)

**Bug 修复 (8 项)**
- 修复编辑页导航切换后显示空白：`onMounted` 增加编辑器状态恢复逻辑（AvatarsPanel / WeaponsPanel / EquipsPanel）
- 修复导航栏计数全显示 0：watch cache 自动更新、移除 `defineExpose` 外部依赖（Sidebar.vue）
- 新增驱动盘创建入口：3 步创建流程 UI（选择套装 → 位置 → 配置属性）+ API 集成（EquipsPanel.vue）
- 标题栏 "Yoshunko Admin" 分两行排列（TitleBar.vue + theme.css）
- 动态读取版本号：`get_version` 从 tauri.conf.json 解析（api.rs）
- 主题切换过渡优化：遮罩先行、切换再渐变（useTheme.ts）
- 任务栏图标模糊：生成 256x256 PNG 加入 bundle（tauri.conf.json）
- 面板切换性能：`KeepAlive` 缓存组件实例，避免销毁重建（MainContent.vue）

**追加修复 (4 项)**
- 侧边栏文字 "Yoshunko Admin" 拆为两行排版（Sidebar.vue）
- 创建驱动盘：Rust 端 `clean_equip_data` 清洗 `key_name` + 过滤 null 副属性，对齐 Python `_dict_to_equip_data`（api.rs）
- 创建驱动盘乱码根因修复：`template_loader.load_templates` 使用 `suit_chinese` 中文名替代 JSON 本地化 key（template_loader.rs）
- 任务栏图标：运行时 `set_icon` 注入 256x256 PNG（lib.rs + tauri `image-png` feature）
- 创建驱动盘改为独立模态框（Teleport → body），匹配 Python 版弹窗体验（EquipsPanel.vue）
- 移除 Suspense 外包层，避免切面板重复显示 skeleton + 三面板 `onActivated` 缓存刷新
- 驱动盘编辑页副属性可编辑：下拉选择属性 + 基础值输入 + +/- 强化按钮（EquipsPanel.vue）
- 未知装备兜底显示：`suit_name` 不存在时显示 `装备_{id}` 而非 `Suit_0`（template_loader.rs）
- 修复删除驱动盘后幽灵卡片：立即从缓存移除已删除 equip，不再仅设 dirty flag（EquipsPanel.vue）
- 编辑页切走再返回重置为仓库视图：watch panel 离开时清空 editor 状态（三面板统一处理）
- 标题栏按钮：SVG 图标居中（flexbox center）+ 标题栏还原单行布局

**其他**
- Naive UI 主题补全 `primaryColorSuppl`（暗色模式适配）
- Tailwind preflight 冲突修复：meta 标签确保 Naive UI 样式优先级（main.ts）

---

## V0.615 (2026-05-14)

### UI/UX 全面重构 — GachaBase 设计语言

**设计系统重构**
- 主强调色从青绿色 (#00d4aa) 改为电光蓝 (#2EB6FF)
- 成功色从青绿色改为绿色 (#2BAD00)
- 所有面板采用玻璃拟态 (glassmorphism) 效果
- 边框使用半透明白色 (rgba(255,255,255,0.08))
- 移除所有 skew 变换效果
- 移除所有 clip-path 切角效果
- 移除所有扫光动画
- 移除水印背景纹理
- 字体栈添加 Inter

**侧边栏重构**
- 可折叠设计：默认 49px，hover 展开到 200px
- 玻璃拟态背景 + 半透明右边框
- 导航项使用 opacity 80% → 100% 过渡
- 活跃项使用蓝色背景填充（非边框指示）
- 标签文字在折叠时隐藏，hover 时滑入显示
- Logo 区域显示品牌标识，hover 展开文字

**标题栏重构**
- 玻璃拟态背景
- 品牌标识 "YOSHUNKO" + 版本号显示
- 半透明底边框

**卡片/面板重构**
- 不对称圆角：顶部 16px，底部 8px
- 玻璃拟态背景 + 半透明边框
- Hover 效果：背景变亮 + 蓝色边框 + 上移

**按钮/输入框重构**
- 移除 skew 和 clip-path 效果
- 标准 6px 圆角
- 输入框使用半透明背景

**Naive UI 主题更新**
- 所有组件主题色更新为电光蓝

---

## V0.614 (2026-05-14)

### 全面 Bug 修复 — 40 个问题修复

**高严重度修复 (5 个)**
- zon.rs: 修复 UTF-8 多字节字符导致的 tokenizer panic（中文 ZON 文件无法解析）
- api.rs: 修复 `launch_program_admin` 的命令注入漏洞（路径含特殊字符时可执行任意命令）
- api.rs: 修复 `debug_avatar_ids` 中 Mutex unwrap 导致的潜在崩溃
- MainContent.vue: 修复 `templates` 加载失败时 `configured` 被误设为 true（主界面空数据）
- AvatarsPanel/WeaponsPanel/EquipsPanel: 修复快速切换时编辑器显示错误数据的竞争条件

**中严重度修复 (19 个)**
- useTheme.ts: 添加 overlay 移除的 setTimeout 兜底，防止遮罩残留
- useStaggeredAnimation.ts: 添加 rAF 取消机制，防止快速搜索时卡片闪烁
- MainContent.vue: 修复 `onConnected` 竞态条件，await checkConfig 后再设置 configured
- MainContent.vue: 修复 `loadCounts` 静默吞掉异常，添加错误日志
- Sidebar.vue: 修复 uid 为 0 时的 falsy 判断错误
- PlayerPanel: 修复导出时 `URL.revokeObjectURL` 竞态条件
- PlayerPanel: 修复导入数据缺少 schema 校验
- PlayerPanel: 修复表单保存时缺少输入值范围校验
- PlayerPanel: 修复确认对话框中 uid 空值风险
- App.vue: 修复确认对话框不等待异步回调就关闭
- api.ts: 添加缺失的 4 个快速启动 API 封装
- api.ts: 修复 `debug_avatar_ids` 发送未使用的 uid 参数
- types.ts: 修复 `main_stat_options` 键类型从 number 改为 string
- QuickLaunchPanel: 修复 `savePath` 不持久化到后端（重启后丢失）
- QuickLaunchPanel: 修复 `launch`/`launchAll` 为 stub（后端已实现）
- api.rs: 修复 `set_launch_path` 配置写入失败不报错
- api.rs: 修复 `set_state_dir` 配置写入失败不报错
- data_manager.rs: 修复 `write_zon` 原子写入缺少 sync_all 和错误检查
- zon.rs: 修复解析器静默接受未闭合的花括号

**低严重度修复 (16 个)**
- RankDots/StarRating/SkeletonGrid: 修复 `max`/`count` 为 0 时的错误回退（`||` → `??`）
- GameCard.vue: 修复 setTimeout 组件卸载后未清理
- AvatarsPanel: 修复 `core_skill` 插入位置硬编码回退
- useKeyboard.ts: 添加 metaKey 检查，防止 Win+数字键触发面板切换
- data_manager.rs: 修复 `backup_zon` 中 unwrap 导致的潜在 panic
- data_manager.rs: 修复 `delete_equip` 静默忽略错误
- data_manager.rs: 修复 `next_uid` 整数溢出

---

## V0.613 (2026-05-14)

### UI/UX 完全重设计 — ZZZ 风格

**设计语言重构**
- 基于《绝区零》游戏 UI 设计语言完全重写视觉系统
- 新增 ZZZ 标志性平行四边形（skew）卡片效果
- 新增柠檬黄（#d4ff00）作为第二强调色
- 暗色主题设为默认主题
- 新增 "YOSHUNKO" 水印背景纹理

**颜色系统**
- 主强调色：电光蓝（#4a9fd8）→ 青绿色（#00d4aa）
- 新增第二强调色：柠檬黄（#d4ff00）
- S 稀有度：金色渐变（#ffd700 → #ffaa00）
- A 稀有度：紫色渐变（#b388ff → #9c6fd4）
- 危险色：红色（#ff2d78）
- 所有颜色变量适配暗色/亮色双主题

**组件更新**
- 卡片、按钮、导航项、输入框等添加 skewX(-4deg) 平行四边形效果
- 稀有度徽章添加反向 skew 补偿
- 导航栏添加活跃状态左侧青绿色边框
- 按钮悬停添加辉光效果（box-shadow glow）
- 所有动画从弹跳曲线改为平滑 ease 过渡

**图标系统**
- 从 CSS 几何图标迁移到 Lucide Vue Next SVG 图标
- 新增图标：Users、CircleDot、Hexagon、Triangle、User、Rocket、Settings、Sun、Moon

**Naive UI 集成**
- 更新 themeOverrides 匹配新颜色系统
- 主按钮、输入框、菜单等组件适配青绿色主题

---

## V0.612 (2026-05-13)

### 版本号重置

- 版本号从 V0.700 重置为 V0.612-r，采用新的版本号递增规则
- 新规则：底层代码修改递增修订号，满30进1；同一问题重复修改或外围修改不递增

---

## V0.700 (2026-05-13)

### 前端完全重写 — Vue 3 + Naive UI

**核心架构变更**
- 前端：从原生 HTML/CSS/JS 完全重写为 Vue 3 + TypeScript + Naive UI
- 构建：引入 Vite 6 构建系统，支持 HMR 热更新和代码自动分割
- 样式：Tailwind CSS 4 + 原有 CSS 设计系统完整迁移
- 状态管理：Vue 3 响应式系统 (composable 模式)
- 动画：Vue 内置 Transition/TransitionGroup 组件
- IPC：直接使用 @tauri-apps/api，移除 tauri-compat.js 兼容层
- 打包体积：5.8MB (原版) → 预计 ~6MB (Vue 版)

**新增功能**
- 快速启动：Rust 后端新增 5 个命令（get_launch_config、set_launch_path、launch_program、launch_program_admin、launch_yoshunko）
- 模板数据补全：get_templates 返回完整的 suit_groups、main_stat_options、sub_stat_options、stat_names、profession_names
- 驱动盘编辑器：主词条/副词条选择下拉框现在可用（数据来自后端模板）

**前端重写详情**
- 33 个 TypeScript/Vue 源文件
- 9 个面板组件（Setup、角色、音擎、驱动盘、式舆防卫战、玩家信息、快速启动、设置、快捷键）
- 8 个共享组件（Stepper、SearchBar、GameCard、SkeletonGrid、EditorPage、RankDots、StarRating、SkillGrid）
- 3 个 composable（useAppState、useTheme、useKeyboard）
- 完整的 TypeScript 类型定义（与 api.rs 返回值完全对齐）
- 所有面板自动代码分割（lazy loaded）
- 拼音搜索数据完整迁移（58 角色 + 93 音擎 + 28 套装）

**后端改动**
- `template_loader.rs`：新增 stat_names 加载
- `api.rs`：get_templates 补全 6 个字段
- `api.rs`：新增 5 个快速启动命令
- `lib.rs`：注册 5 个新命令（共 31 个）
- `tauri.conf.json`：frontendDist 改为 ../dist，添加 Vite 构建命令

**配置变更**
- 新增：package.json、tsconfig.json、vite.config.ts、index.html
- 修改：tauri.conf.json（frontendDist、withGlobalTauri、构建命令）
- 保留：static/ 目录（旧前端，迁移完成后可删除）

---

## V0.609 (2026-05-13)

### 初始版本 — 从 Python 版完整移植

**核心架构**
- 前端：复用 Python 版全部 HTML/CSS/JS，零改动
- 后端：Rust + Tauri v2，原生 WebView2 渲染
- 兼容层：`tauri-compat.js` 将 `pywebview.api` 调用映射到 Tauri IPC
- 打包体积：50MB (Python) → 5.8MB (Rust)

**已实现功能**
- 角色管理（等级、影画、技能、潜能激发、皮肤ID、英文名）
- 音擎仓库（等级、精炼、英文名）
- 驱动盘仓库（创建、编辑、删除、套装英文名）
- 式舆防卫战存档编辑
- 玩家基本信息修改
- 角色分组切换（阵营/职业）
- 拼音搜索（全拼/首字母）
- 暗色/亮色主题切换
- 快速启动面板（Yoshunko 服务端 + 三件套路径配置）
- 数据导出/导入（前端接口，后端待实现）
- 果冻动画（面板切换、卡片弹出、编辑器滑入）
- 键盘快捷键（1-7 切换面板、Ctrl+S 保存、F 打开编辑器）

**Rust 后端修复（相比初始移植）**
- ZON 解析器：`=` 符号处理、`.{` 数组/结构体区分、裸点号跳过、关键字边界检查
- 输入验证：所有 update/create 端点添加范围校验
- 数据完整性：`get_avatar` 返回完整字段（exp、rank、talent_switch_list、dressed_equip 等）
- `get_equip` 返回 properties 和 sub_properties
- `list_dir` 过滤 "next" 和 ".tmp" 文件

**已知限制**
- 导出/导入功能未实现（前端接口已有，后端返回未实现提示）
- `window_move` 未实现（Tauri 原生处理窗口拖拽）
- 数据目录需手动复制到 exe 同目录
