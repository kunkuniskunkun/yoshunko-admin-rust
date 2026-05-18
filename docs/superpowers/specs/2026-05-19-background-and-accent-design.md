# 自定义背景图 + 主题色切换 设计文档

> 2026-05-19

---

## 概述

为 Yoshunko Admin 添加两项个性化功能：
1. **自定义背景图**：用户选择本地图片作为整个窗口的背景，带半透明遮罩保证可读性
2. **主题色切换**：预设 5-6 种主色调，一键切换按钮/高亮/选中态的颜色

---

## 一、自定义背景图

### 1.1 布局结构

在 `App.vue` 的 `.app-layout` 内部、所有内容之前添加背景层：

```
.app-layout
  ├── .bg-layer (position: fixed, z-index: -1, background-image)
  ├── .bg-overlay (position: fixed, z-index: -1, 半透明遮罩)
  ├── TitleBar
  └── .app-body
       ├── Sidebar
       └── MainContent
```

- `.bg-layer`：`background-size: cover; background-position: center;` 填满窗口
- `.bg-overlay`：`background: var(--bg-void); opacity: 可调;` 保证文字可读

### 1.2 图片加载

使用 Tauri 的 `convertFileSrc()` 将本地文件路径转为可加载 URL：

```typescript
import { convertFileSrc } from '@tauri-apps/api/core'
const url = convertFileSrc(selectedPath)
// → asset://localhost/C:/Users/xxx/Pictures/wallpaper.jpg
```

### 1.3 配置持久化

在 `config.json` 中新增 `background` 字段：

```json
{
  "state_dir": "...",
  "launch": { ... },
  "background": {
    "path": "C:\\Users\\xxx\\Pictures\\wallpaper.jpg",
    "opacity": 0.85
  }
}
```

- `path`：图片文件的绝对路径（文件不存在时自动忽略，回退默认背景）
- `opacity`：遮罩透明度 0.3~0.95（0.3=几乎全透明，0.95=接近纯色）

启动时读取 → 加载背景 → 应用遮罩透明度。

### 1.4 设置面板 UI

在 SettingsPanel 中添加"背景图"设置区域：

```
┌─────────────────────────────────────────┐
│ 背景图                                   │
│                                         │
│  [选择图片]  [清除背景]                   │
│                                         │
│  遮罩透明度  ■■■■■■■□□□  0.70           │
│                                         │
│  当前: C:\Users\xxx\Pictures\bg.jpg     │
└─────────────────────────────────────────┘
```

- **选择图片**：调用 `@tauri-apps/plugin-dialog` 的 `open()` 过滤 png/jpg/webp
- **清除背景**：移除背景图，恢复默认纯色
- **遮罩透明度**：滑块实时调节，拖动时即时预览
- **当前路径**：显示已选图片的文件名（截断长路径）

### 1.5 后端 API

复用现有的 config 读写机制。前端读取时 `api.getConfig()` 返回的 config 对象中已包含 `background` 字段。

写入需要在 Rust 后端新增一个 command：

```rust
#[tauri::command]
pub fn set_background(state: State<AppState>, path: String, opacity: f64) -> Value {
    // 读取现有 config → 更新 background 字段 → atomic_write_config
}
```

### 1.6 边界处理

- **文件被删除/移动**：启动时检测路径是否存在，不存在则忽略，显示默认背景
- **超大图片**：CSS `background-size: cover` 自动缩放，浏览器引擎处理，无需额外优化
- **路径包含中文/空格**：`convertFileSrc` 自动处理 URL 编码
- **暗色/亮色模式切换**：遮罩颜色跟随 `var(--bg-void)`，自动适配

---

## 二、主题色切换

### 2.1 预设配色方案

每套配色定义完整的 `--accent-*` 变量组：

| 名称 | --accent | --accent-dark | 效果预览 |
|------|----------|---------------|---------|
| 海蓝 (默认) | `#4a9fd8` | `#3a80b0` | 当前颜色 |
| 翠绿 | `#4caf7d` | `#3a8f65` | 清新自然 |
| 藤紫 | `#8b6cc1` | `#7054a0` | 优雅神秘 |
| 珊瑚红 | `#e06060` | `#c04848` | 热情醒目 |
| 琥珀橙 | `#e09050` | `#c07838` | 温暖活力 |
| 樱粉 | `#d07090` | `#b05878` | 柔和甜美 |

每套配色的完整变量：

```css
[data-accent="green"] {
  --accent: #4caf7d;
  --accent-dark: #3a8f65;
  --accent-glow: rgba(76, 175, 125, 0.35);
  --accent-bg: rgba(76, 175, 125, 0.07);
  --accent-secondary: #7ecda8;
  --accent-secondary-dark: #5ab88a;
  --accent-secondary-glow: rgba(126, 205, 168, 0.3);
  --accent-secondary-bg: rgba(126, 205, 168, 0.06);
}
```

`--accent-glow` 和 `--accent-bg` 的 rgba 值从主色自动计算，无需手动指定。

### 2.2 实现方式

在 `<html>` 元素上设置 `data-accent` 属性：

```typescript
document.documentElement.setAttribute('data-accent', 'green')
```

CSS 中用属性选择器覆盖变量：

```css
[data-accent="green"] { --accent: #4caf7d; ... }
[data-accent="purple"] { --accent: #8b6cc1; ... }
/* 默认（不设 data-accent 或 data-accent="blue"）使用 :root 中的值 */
```

### 2.3 持久化

保存到 `localStorage`（纯前端状态，不需要后端参与）：

```typescript
localStorage.setItem('accent', 'green')  // 保存
const accent = localStorage.getItem('accent') || 'blue'  // 读取
```

启动时在 `App.vue` 的 `onMounted` 中读取并应用。

### 2.4 设置面板 UI

在 SettingsPanel 中添加"主题色"设置区域：

```
┌─────────────────────────────────────────┐
│ 主题色                                   │
│                                         │
│  ●  ●  ●  ●  ●  ●                      │
│  蓝  绿  紫  红  橙  粉                  │
│                                         │
└─────────────────────────────────────────┘
```

- 6 个圆形色块，点击即切换
- 当前选中的色块加白色边框 + 放大效果
- 切换即时生效，无需保存按钮

---

## 三、涉及文件

| 文件 | 改动 |
|------|------|
| `App.vue` | 添加 `.bg-layer` + `.bg-overlay` div，onMounted 读取背景配置和主题色 |
| `SettingsPanel.vue` | 添加"背景图"和"主题色"设置区域 |
| `theme.css` | 添加 `[data-accent="xxx"]` 变量覆盖规则 |
| `api/mod.rs` | 新增 `set_background` command（或在 config.rs） |
| `lib/api.ts` | 新增 `setBackground` wrapper |
| `lib/types.ts` | Config 类型新增 `background` 字段 |

---

## 四、实施顺序

1. 主题色切换（纯前端，~1.5h）
   - theme.css 添加 5 套 data-accent 变量
   - useTheme.ts 扩展主题色逻辑
   - SettingsPanel 添加色块选择器
2. 自定义背景图（前后端，~2.5h）
   - App.vue 添加背景层
   - Rust 后端新增 set_background command
   - SettingsPanel 添加背景图设置区域
   - config.json 读写集成

---

## 五、边界与风险

| 风险 | 缓解 |
|------|------|
| 背景图文件被删除 | 启动时检测路径存在性，不存在则忽略 |
| 遮罩透明度过低导致文字不可读 | 滑块范围限制在 0.3~0.95，最低 0.3 |
| 暗色模式下主题色可读性 | 每套配色同时定义 light/dark 两组值（或保持统一，因为现有 --accent 在 dark mode 下已有 --accent-dark 变体） |
| convertFileSrc 在 dev 模式下的协议差异 | Tauri 自动处理 asset:// 协议，dev 和 release 一致 |
