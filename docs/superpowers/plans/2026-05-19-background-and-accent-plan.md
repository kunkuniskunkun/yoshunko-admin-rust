# 自定义背景图 + 主题色切换 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Yoshunko Admin 添加自定义背景图和 6 色主题切换功能

**Architecture:** 主题色通过 CSS `[data-accent]` 属性选择器覆盖 CSS 变量，localStorage 持久化。背景图通过 `position: fixed` 背景层 + 遮罩实现，config.json 持久化。

**Tech Stack:** Vue 3, TypeScript, CSS variables, Tauri v2 (Rust), `@tauri-apps/plugin-dialog`, `convertFileSrc`

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/styles/theme.css` | 修改 | 添加 6 套 `[data-accent]` CSS 变量规则 |
| `src/composables/useTheme.ts` | 修改 | 扩展主题色管理（accent ref + localStorage） |
| `src/composables/useBackground.ts` | 新建 | 背景图状态管理（bgUrl, bgOpacity, setBackground） |
| `src/App.vue` | 修改 | 添加 `.bg-layer` + `.bg-overlay`，启动时加载背景和主题色 |
| `src/components/panels/SettingsPanel.vue` | 修改 | 添加主题色选择器 + 背景图设置 UI |
| `src-tauri/src/api/config.rs` | 修改 | 新增 `set_background` command |
| `src-tauri/src/lib.rs` | 修改 | 注册 `set_background` handler |
| `src/lib/api.ts` | 修改 | 新增 `setBackground` wrapper |
| `src/lib/types.ts` | 修改 | Config 类型新增 `background` 字段 |

---

### Task 1: theme.css — 添加 6 套主题色变量

**Files:**
- Modify: `src/styles/theme.css:31-41`（在 `:root` 的 accent 变量之后）

在 `:root` 中现有的 `--accent-secondary-bg` 之后（约 L41），添加 5 套新主题色的 `[data-accent]` 规则。默认蓝色保持在 `:root` 中不加属性选择器。

- [ ] **Step 1: 在 theme.css 的 `:root` 末尾（`--accent-secondary-bg` 之后）添加 5 套 data-accent 规则**

```css
/* ─── Accent color themes ────────────────────────────── */

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

[data-accent="purple"] {
  --accent: #8b6cc1;
  --accent-dark: #7054a0;
  --accent-glow: rgba(139, 108, 193, 0.35);
  --accent-bg: rgba(139, 108, 193, 0.07);
  --accent-secondary: #b090d8;
  --accent-secondary-dark: #9070c0;
  --accent-secondary-glow: rgba(176, 144, 216, 0.3);
  --accent-secondary-bg: rgba(176, 144, 216, 0.06);
}

[data-accent="red"] {
  --accent: #e06060;
  --accent-dark: #c04848;
  --accent-glow: rgba(224, 96, 96, 0.35);
  --accent-bg: rgba(224, 96, 96, 0.07);
  --accent-secondary: #e88888;
  --accent-secondary-dark: #d06868;
  --accent-secondary-glow: rgba(232, 136, 136, 0.3);
  --accent-secondary-bg: rgba(232, 136, 136, 0.06);
}

[data-accent="orange"] {
  --accent: #e09050;
  --accent-dark: #c07838;
  --accent-glow: rgba(224, 144, 80, 0.35);
  --accent-bg: rgba(224, 144, 80, 0.07);
  --accent-secondary: #e8b080;
  --accent-secondary-dark: #d09860;
  --accent-secondary-glow: rgba(232, 176, 128, 0.3);
  --accent-secondary-bg: rgba(232, 176, 128, 0.06);
}

[data-accent="pink"] {
  --accent: #d07090;
  --accent-dark: #b05878;
  --accent-glow: rgba(208, 112, 144, 0.35);
  --accent-bg: rgba(208, 112, 144, 0.07);
  --accent-secondary: #e098b0;
  --accent-secondary-dark: #c87898;
  --accent-secondary-glow: rgba(224, 152, 176, 0.3);
  --accent-secondary-bg: rgba(224, 152, 176, 0.06);
}
```

- [ ] **Step 2: 验证**

运行: `npm run build`
预期: 编译通过（纯 CSS 新增，不影响现有逻辑）

- [ ] **Step 3: 提交**

```bash
git add src/styles/theme.css
git commit -m "feat: 添加 6 套主题色 CSS 变量（蓝绿紫红橙粉）"
```

---

### Task 2: useTheme.ts — 扩展主题色管理

**Files:**
- Modify: `src/composables/useTheme.ts`

在现有 `currentTheme` / `initTheme` / `toggleTheme` / `setTheme` 旁边，添加主题色的 ref、init、set 函数。

- [ ] **Step 1: 在 useTheme.ts 末尾添加主题色管理代码**

在文件末尾追加：

```typescript
// ─── Accent Color ─────────────────────────────────────

export type AccentColor = 'blue' | 'green' | 'purple' | 'red' | 'orange' | 'pink'

export const ACCENT_COLORS: { key: AccentColor; label: string; hex: string }[] = [
  { key: 'blue',   label: '海蓝', hex: '#4a9fd8' },
  { key: 'green',  label: '翠绿', hex: '#4caf7d' },
  { key: 'purple', label: '藤紫', hex: '#8b6cc1' },
  { key: 'red',    label: '珊瑚红', hex: '#e06060' },
  { key: 'orange', label: '琥珀橙', hex: '#e09050' },
  { key: 'pink',   label: '樱粉', hex: '#d07090' },
]

export const currentAccent = ref<AccentColor>('blue')

export function initAccent() {
  try {
    const saved = localStorage.getItem('yos-accent')
    if (saved && ACCENT_COLORS.some(c => c.key === saved)) {
      currentAccent.value = saved as AccentColor
    }
  } catch {}
  applyAccent(currentAccent.value)
}

export function setAccent(color: AccentColor) {
  currentAccent.value = color
  applyAccent(color)
  try { localStorage.setItem('yos-accent', color) } catch {}
}

function applyAccent(color: AccentColor) {
  if (color === 'blue') {
    document.documentElement.removeAttribute('data-accent')
  } else {
    document.documentElement.setAttribute('data-accent', color)
  }
}
```

- [ ] **Step 2: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/composables/useTheme.ts
git commit -m "feat: 添加主题色管理（currentAccent + setAccent + localStorage）"
```

---

### Task 3: SettingsPanel — 添加主题色选择器

**Files:**
- Modify: `src/components/panels/SettingsPanel.vue:1-6`（import）
- Modify: `src/components/panels/SettingsPanel.vue:172-182`（模板，在"界面偏好"区域后添加）

- [ ] **Step 1: 更新 import**

将 L6 的 import 改为：

```typescript
import { currentTheme, setTheme, currentAccent, setAccent, ACCENT_COLORS } from '@/composables/useTheme'
```

- [ ] **Step 2: 在模板的"界面偏好" section 末尾（`</div>` 关闭 `form-row` 之后）添加主题色选择器**

在 `<!-- Appearance -->` 区域的 `</div>` 关闭 `form-row` 之后、`<!-- Logs -->` 之前，插入：

```html
          <div class="form-field" style="margin-top: 12px;">
            <label class="form-label">主题色</label>
            <div class="accent-picker">
              <button
                v-for="color in ACCENT_COLORS"
                :key="color.key"
                class="accent-dot"
                :class="{ active: currentAccent === color.key }"
                :style="{ '--dot-color': color.hex }"
                :title="color.label"
                @click="setAccent(color.key)"
              />
            </div>
          </div>
```

- [ ] **Step 3: 在 theme.css 中添加主题色选择器的样式**

在 theme.css 末尾追加：

```css
/* ─── Accent picker ─────────────────────────────────── */

.accent-picker {
  display: flex;
  gap: 10px;
  margin-top: 6px;
}

.accent-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  background: var(--dot-color);
  cursor: pointer;
  transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
}

.accent-dot:hover {
  transform: scale(1.15);
}

.accent-dot.active {
  border-color: var(--text);
  box-shadow: 0 0 0 3px var(--dot-color);
  transform: scale(1.1);
}
```

- [ ] **Step 4: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src/components/panels/SettingsPanel.vue src/styles/theme.css
git commit -m "feat: SettingsPanel 添加主题色选择器 UI"
```

---

### Task 4: App.vue — 启动时加载主题色

**Files:**
- Modify: `src/App.vue:1-8`（script）
- Modify: `src/App.vue:59`（template）

- [ ] **Step 1: 更新 App.vue 的 import 和 onMounted**

将 L2 的 import 改为：

```typescript
import { computed, onMounted } from 'vue'
import { darkTheme, lightTheme } from 'naive-ui'
import { currentTheme, initAccent } from '@/composables/useTheme'
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
import { toasts, removeToast, confirmState, closeConfirm } from '@/lib/utils'
```

在 `<script setup>` 中 `themeOverrides` 之后添加：

```typescript
onMounted(() => {
  initAccent()
})
```

- [ ] **Step 2: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/App.vue
git commit -m "feat: App.vue 启动时加载主题色"
```

---

### Task 5: Rust 后端 — 新增 set_background command

**Files:**
- Modify: `src-tauri/src/api/config.rs`（在 `set_state_dir` 之后添加）
- Modify: `src-tauri/src/lib.rs`（注册 handler）

- [ ] **Step 1: 更新 `get_config` 命令，返回 `background` 字段**

在 `get_config` 函数中，`let launch_config = ...` 之后添加：

```rust
    let background = config.get("background").cloned();
```

在 `json!({...})` 的末尾（`"launch_config": launch_config` 之后）添加：

```rust
        "background": background,
```

- [ ] **Step 2: 在 config.rs 的 `set_state_dir` 函数之后添加 `set_background`**

在 `set_state_dir` 函数的 `}` 之后、`auto_detect_paths` 之前，添加：

```rust
#[tauri::command]
pub fn set_background(state: State<AppState>, path: String, opacity: f64) -> Value {
    let opacity = opacity.clamp(0.3, 0.95);
    let mut config_map: serde_json::Map<String, serde_json::Value> =
        std::fs::read_to_string(&state.config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    if path.is_empty() {
        config_map.remove("background");
    } else {
        let mut bg = serde_json::Map::new();
        bg.insert("path".to_string(), json!(path));
        bg.insert("opacity".to_string(), json!(opacity));
        config_map.insert("background".to_string(), serde_json::Value::Object(bg));
    }
    if let Err(e) = atomic_write_config(&state.config_path, &config_map) {
        return json!({"ok": false, "error": e});
    }
    json!({"ok": true})
}
```

- [ ] **Step 3: 在 lib.rs 的 `generate_handler!` 中注册 `set_background`**

在 `api::auto_detect_paths,` 之后添加一行：

```
            api::set_background,
```

- [ ] **Step 4: 验证**

运行: `cd src-tauri && cargo check`
预期: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/api/config.rs src-tauri/src/lib.rs
git commit -m "feat: 新增 set_background command + get_config 返回 background"
```

---

### Task 6: api.ts + types.ts — 前端 API 集成

**Files:**
- Modify: `src/lib/types.ts:3-9`（Config 接口）
- Modify: `src/lib/api.ts`（新增 setBackground）

- [ ] **Step 1: 更新 types.ts 的 Config 接口**

将 Config 接口改为：

```typescript
export interface Config {
  configured: boolean
  config_exists: boolean
  state_dir: string
  version: string
  launch_config: Record<string, string>
  background?: { path: string; opacity: number }
}
```

- [ ] **Step 2: 在 api.ts 的 `autoDetectPaths` 之后添加 `setBackground`**

```typescript
  setBackground: (path: string, opacity: number) =>
    invoke<{ ok: boolean; error?: string }>('set_background', { path, opacity }),
```

- [ ] **Step 3: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src/lib/types.ts src/lib/api.ts
git commit -m "feat: 前端 API 集成 setBackground + Config 类型扩展"
```

---

### Task 7: App.vue — 添加背景层

**Files:**
- Modify: `src/App.vue`（script + template）

- [ ] **Step 1: 更新 App.vue script 部分**

在现有 import 中添加两行：

```typescript
import { currentTheme, initAccent } from '@/composables/useTheme'  // 已有，不变
import { bgUrl, bgOpacity, setBackground } from '@/composables/useBackground'  // 新增
import { api } from '@/lib/api'  // 新增
```

在 `themeOverrides` 之后添加 `onMounted`：

```typescript
onMounted(async () => {
  initAccent()
  try {
    const config = await api.getConfig()
    if (config.background?.path) {
      setBackground(config.background.path, config.background.opacity)
    }
  } catch {}
})
```

确保 `<script setup>` 的 import 包含 `onMounted`。

- [ ] **Step 2: 更新 App.vue template**

在 `<div class="app-layout">` 之后、`<TitleBar />` 之前，添加：

```html
              <!-- Background layer -->
              <div v-if="bgUrl" class="bg-layer" :style="{ backgroundImage: `url(${bgUrl})` }" />
              <div v-if="bgUrl" class="bg-overlay" :style="{ opacity: bgOpacity }" />
```

- [ ] **Step 3: 在 App.vue 的 `<style>` 中添加背景层样式**

如果 App.vue 没有 `<style>` 块，在 `</template>` 之后添加：

```css
<style scoped>
.bg-layer {
  position: fixed;
  inset: 0;
  z-index: -2;
  background-size: cover;
  background-position: center;
  pointer-events: none;
}

.bg-overlay {
  position: fixed;
  inset: 0;
  z-index: -1;
  background: var(--bg-void);
  pointer-events: none;
}
</style>
```

- [ ] **Step 4: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src/App.vue src/composables/useBackground.ts
git commit -m "feat: App.vue 添加背景层（bg-layer + bg-overlay + 启动加载）"
```

---

### Task 8: SettingsPanel — 添加背景图设置 UI

**Files:**
- Modify: `src/components/panels/SettingsPanel.vue`（script + template）

- [ ] **Step 1: 在 SettingsPanel 的 script import 中添加背景相关**

```typescript
import { bgUrl, bgOpacity, bgPath, setBackground } from '@/composables/useBackground'
```

在 `goToShortcuts` 函数之后添加：

```typescript
// ─── Background image ──────────────────────────────────
async function selectBackground() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    multiple: false,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (!selected) return
  const path = selected as string
  try {
    const r = await api.setBackground(path, bgOpacity.value)
    if (!r.ok) { toast('保存失败: ' + r.error, 'error'); return }
    setBackground(path, bgOpacity.value)
    toast('背景图已设置', 'success')
  } catch (e: unknown) {
    toast('设置失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
}

async function clearBackground() {
  try {
    const r = await api.setBackground('', 0)
    if (!r.ok) { toast('清除失败: ' + r.error, 'error'); return }
    setBackground('', 0.85)
    toast('背景图已清除', 'success')
  } catch (e: unknown) {
    toast('清除失败: ' + (e instanceof Error ? e.message : ''), 'error')
  }
}

async function setBgOpacity(val: number) {
  const opacity = Math.round(val * 100) / 100
  bgOpacity.value = opacity
  if (bgPath.value) {
    try {
      await api.setBackground(bgPath.value, opacity)
    } catch {}
  }
}
```

- [ ] **Step 2: 在模板的"界面偏好"区域末尾（主题色选择器之后）添加背景图设置**

在主题色 `form-field` 的 `</div>` 之后、`<!-- Logs -->` 之前，插入：

```html
          <div class="form-field" style="margin-top: 16px;">
            <label class="form-label">背景图</label>
            <div class="btn-group" style="margin-bottom: 8px;">
              <button class="btn btn-ghost" @click="selectBackground">选择图片</button>
              <button class="btn btn-ghost" @click="clearBackground">清除背景</button>
            </div>
            <div v-if="bgPath" class="form-field" style="margin-top: 8px;">
              <label class="form-label">遮罩透明度</label>
              <div style="display: flex; align-items: center; gap: 12px;">
                <input
                  type="range"
                  min="0.3"
                  max="0.95"
                  step="0.05"
                  :value="bgOpacity"
                  style="flex: 1;"
                  @input="setBgOpacity(parseFloat(($event.target as HTMLInputElement).value))"
                />
                <span class="text-muted text-xs" style="min-width: 36px;">{{ Math.round(bgOpacity * 100) }}%</span>
              </div>
            </div>
          </div>
```

- [ ] **Step 3: 验证**

运行: `npm run build`
预期: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src/components/panels/SettingsPanel.vue
git commit -m "feat: SettingsPanel 添加背景图设置（选择/清除/透明度滑块）"
```

---

### Task 9: 最终验证与版本更新

- [ ] **Step 1: 全量构建验证**

```bash
cd src-tauri && cargo check
cd .. && npm run build
```

预期: 两者均通过

- [ ] **Step 2: 更新版本号**

- `src-tauri/tauri.conf.json`: `"version": "0.711.0"` → `"0.712.0"`
- `CLAUDE.md`: `v0.711-r` → `v0.712-r`

- [ ] **Step 3: 更新 CHANGELOG.md**

在 `## V0.711` 之前添加：

```markdown
## V0.712 (2026-05-19)

### 个性化功能

**自定义背景图**
- 支持选择本地图片（png/jpg/webp）作为整个窗口背景
- 半透明遮罩保证文字可读性，透明度 0.3~0.95 可调
- 配置持久化到 config.json，下次启动自动加载
- 文件被删除/移动时自动忽略，回退默认纯色背景

**主题色切换**
- 6 种预设主题色：海蓝（默认）、翠绿、藤紫、珊瑚红、琥珀橙、樱粉
- CSS `[data-accent]` 属性选择器覆盖变量，切换即时生效
- 持久化到 localStorage
- 设置面板色块选择器，当前选中项高亮
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/tauri.conf.json CLAUDE.md CHANGELOG.md
git commit -m "V0.712 — 自定义背景图 + 主题色切换"
```
