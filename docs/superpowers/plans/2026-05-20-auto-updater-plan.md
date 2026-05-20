# 自动更新功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 应用启动时自动检查 GitHub Releases 上的新版本，TitleBar 角标通知，用户自主选择安装或跳过。

**Architecture:** 使用 tauri-plugin-updater 官方插件，前端通过 `@tauri-apps/plugin-updater` JS API 直接调用。SettingsPanel 保留手动检查按钮 + 手动下载链接作备用。

**Tech Stack:** tauri-plugin-updater (Rust), @tauri-apps/plugin-updater (JS), GitHub Releases

---

### Task 1: 安装更新插件依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `package.json`

- [ ] **Step 1: 添加 Rust 依赖**

`src-tauri/Cargo.toml` — 在 `[dependencies]` 中添加：
```toml
tauri-plugin-updater = "2"
```

- [ ] **Step 2: 在 Tauri builder 中注册插件**

`src-tauri/src/lib.rs` — 在 `use tauri::Manager;` 之后添加：
```rust
use tauri_plugin_updater::UpdaterExt;
```

在 `.plugin(tauri_plugin_shell::init())` 之前添加：
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

(完整 builder 链变为 updater → shell → dialog)

- [ ] **Step 3: 安装前端 JS 包**

运行：
```bash
npm install @tauri-apps/plugin-updater
```

- [ ] **Step 4: 验证编译**

运行：
```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
期望：无新增错误

- [ ] **Step 5: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs package.json package-lock.json
git commit -m "deps: 添加 tauri-plugin-updater 更新插件"
```

---

### Task 2: 配置 updater endpoint

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 添加 updater 和 bundle 配置**

在 `tauri.conf.json` 中添加 `plugins` 字段（与 `app`、`bundle` 同级）：

```json
"plugins": {
  "updater": {
    "endpoints": [
      "https://github.com/kunkunr/yoshunko-admin-rust/releases/latest/download/latest.json"
    ]
  }
}
```

> 注意：production 环境需要添加 `"pubkey"` 签名验证，当前先用无签名的方案跑通。

在 `bundle` 中添加 `createUpdaterArtifacts`：

```json
"bundle": {
  "active": true,
  "targets": "nsis",
  "createUpdaterArtifacts": true,
  ...
}
```

(在现有 `"bundle"` 对象中添加 `"createUpdaterArtifacts": true`，位于 `"targets": "nsis"` 之后)

- [ ] **Step 2: 验证 JSON 有效性**

```bash
"/c/Program Files/nodejs/node" -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json','utf8')); console.log('OK')"
```
期望：OK

- [ ] **Step 3: 提交**

```bash
git add src-tauri/tauri.conf.json
git commit -m "config: 配置 updater endpoint 和构建产物"
```

---

### Task 3: 前端更新逻辑

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/lib/api.ts`
- Modify: `src/App.vue`
- Modify: `src/components/layout/TitleBar.vue`
- Modify: `src/components/panels/SettingsPanel.vue`

- [ ] **Step 1: 添加 UpdateInfo 类型**

`src/lib/types.ts` — 在文件末尾添加：

```typescript
export interface UpdateInfo {
  version: string
  body: string
  date: string
}
```

- [ ] **Step 2: 创建 updater 工具**

新建 `src/composables/useUpdater.ts`：

```typescript
import { ref } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { pushToast } from '@/lib/utils'
import type { UpdateInfo } from '@/lib/types'

export const updateInfo = ref<UpdateInfo | null>(null)
export const updateAvailable = ref(false)

let pendingUpdate: Update | null = null

export async function checkUpdate(): Promise<boolean> {
  try {
    const update = await check()
    if (update) {
      pendingUpdate = update
      updateInfo.value = {
        version: update.version,
        body: update.body || '',
        date: update.date || '',
      }
      updateAvailable.value = true
      return true
    }
    return false
  } catch {
    // 静默失败——开发环境或网络不可用时不影响正常使用
    return false
  }
}

export async function installUpdate(onProgress?: (pct: number) => void) {
  if (!pendingUpdate) return
  try {
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Progress' && onProgress) {
        onProgress(Math.round((event.data.downloaded / event.data.total) * 100))
      }
    })
  } catch (e) {
    pushToast(`更新失败: ${e}`, 'error')
  }
}

export function openReleasePage() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('open_release_page')
}
```

- [ ] **Step 3: App.vue onMounted 添加后台检查**

在 `src/App.vue` 的 `<script setup>` 中：

添加导入：
```typescript
import { checkUpdate } from '@/composables/useUpdater'
```

在 `onMounted` 末尾（`catch` 之后）添加：
```typescript
// 后台检查更新
checkUpdate()
```

完整的 onMounted 变为：
```typescript
onMounted(async () => {
  initAccent()
  try {
    const config = await api.getConfig()
    if (config.background?.path) {
      await setBackground(config.background.path, config.background.opacity)
    }
  } catch (e) { console.error('[App] getConfig failed:', e) }
  // 后台检查更新（静默，有新版也不弹窗，只设 updateInfo 触发角标）
  checkUpdate()
})
```

- [ ] **Step 4: TitleBar 添加更新角标和 Modal**

`src/components/layout/TitleBar.vue` — 替换整个文件：

```vue
<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { api } from '@/lib/api'
import { ref, onMounted } from 'vue'
import { updateAvailable, updateInfo, installUpdate } from '@/composables/useUpdater'
import { NModal, NButton } from 'naive-ui'

const appWindow = getCurrentWindow()
const version = ref('---')
const showModal = ref(false)
const installing = ref(false)
const progress = ref(0)

function minimize() { appWindow.minimize() }
function toggleMax() { appWindow.toggleMaximize() }
function close() { appWindow.close() }

async function doInstall() {
  installing.value = true
  await installUpdate((pct) => { progress.value = pct })
  // downloadAndInstall 成功后会重启应用，不会回到这里
}

onMounted(async () => {
  try { const data = await api.getVersion(); version.value = data.version } catch { version.value = '---' }
})
</script>

<template>
  <div data-tauri-drag-region class="title-bar">
    <div class="title-bar__left" data-tauri-drag-region>
      <span class="title-bar__brand">Yoshunko Admin</span>
      <span class="title-bar__version">{{ version }}</span>
      <span
        v-if="updateAvailable"
        class="title-bar__update-badge"
        @click.stop="showModal = true"
      >
        新版本 v{{ updateInfo?.version }}
      </span>
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

  <NModal v-model:show="showModal" title="发现新版本" :mask-closable="false">
    <div class="update-modal">
      <p>新版本: <strong>v{{ updateInfo?.version }}</strong></p>
      <pre class="update-notes">{{ updateInfo?.body || '无更新说明' }}</pre>
      <p v-if="installing">下载进度: {{ progress }}%</p>
      <div class="update-modal__actions">
        <NButton @click="showModal = false">稍后提醒</NButton>
        <NButton type="primary" @click="doInstall" :loading="installing">
          {{ installing ? '下载安装中...' : '立即更新' }}
        </NButton>
      </div>
    </div>
  </NModal>
</template>
```

- [ ] **Step 5: SettingsPanel 添加手动检查和下载**

在 `src/components/panels/SettingsPanel.vue` 的 template 中，`关于` section 之前添加：

```html
<!-- 更新 -->
<div class="section-title">更新</div>
<div class="settings-row">
  <span class="settings-label">自动检查</span>
  <span class="settings-value">启动时自动检查新版本</span>
</div>
<div class="settings-row">
  <NButton size="small" @click="handleCheckUpdate" :loading="checking">
    {{ checking ? '检查中...' : '手动检查更新' }}
  </NButton>
  <NButton size="small" text @click="openReleasePage">手动下载</NButton>
</div>
```

在 script 中添加：
```typescript
import { checkUpdate, updateAvailable, updateInfo, openReleasePage } from '@/composables/useUpdater'

const checking = ref(false)
async function handleCheckUpdate() {
  checking.value = true
  const found = await checkUpdate()
  checking.value = false
  if (found) {
    pushToast(`发现新版本 v${updateInfo.value?.version}`, 'success')
  } else {
    pushToast('已是最新版本', 'info')
  }
}
```

- [ ] **Step 6: 验证类型检查**

```bash
npx vue-tsc --noEmit
```
期望：无新增类型错误

- [ ] **Step 7: 提交**

```bash
git add src/lib/types.ts src/composables/useUpdater.ts src/App.vue src/components/layout/TitleBar.vue src/components/panels/SettingsPanel.vue
git commit -m "feat: 启动自动检查更新 + TitleBar 角标 + 手动更新"
```

---

### Task 4: CSS 更新角标样式

**Files:**
- Modify: `src/styles/theme.css`

- [ ] **Step 1: 添加角标和 Modal 样式**

在 `theme.css` 末尾添加：

```css
/* ─── Update Badge ─────────────────────────────── */
.title-bar__update-badge {
  background: var(--accent);
  color: #fff;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  cursor: pointer;
  transition: filter 0.2s;
  margin-left: 10px;
}
.title-bar__update-badge:hover {
  filter: brightness(1.15);
}

.update-modal {
  min-width: 320px;
}
.update-notes {
  font-size: 13px;
  color: var(--text-secondary);
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
  margin: 12px 0;
  padding: 8px;
  background: var(--bg-hover);
  border-radius: 6px;
}
.update-modal__actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/styles/theme.css
git commit -m "style: 更新角标和 Modal 样式"
```

---

### Task 5: Rust 端打开的 Release 页面命令

**Files:**
- Modify: `src-tauri/src/api/config.rs`

`config.rs` 中添加（仅需 `open` crate 已在依赖中）：

在 `config.rs` 末尾添加：

```rust
#[tauri::command]
pub fn open_release_page() -> Value {
    let repo = "https://github.com/kunkunr/yoshunko-admin-rust/releases";
    if let Err(e) = std::process::Command::new("cmd")
        .args(["/c", "start", repo])
        .spawn()
    {
        return json!({"ok": false, "error": e.to_string()});
    }
    json!({"ok": true})
}
```

同时在 `lib.rs` 的 `generate_handler!` 列表中添加：
```rust
api::open_release_page,
```

- [ ] **Step 2: 验证编译**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
期望：无新增错误

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/api/config.rs src-tauri/src/lib.rs
git commit -m "feat: 添加打开 Release 页面命令"
```

---

### Task 6: 增强 release 脚本生成 latest.json

**Files:**
- Modify: `scripts/release.cjs`

- [ ] **Step 1: 添加 latest.json 生成逻辑**

在 `scripts/release.cjs` 的 "创建 tag" 步骤之前，添加生成 `latest.json` 的步骤：

```javascript
// 3.5 Generate updater manifest
const latestJson = {
  version: `v${frontVer}`,
  notes: (() => {
    try {
      const changelog = fs.readFileSync(`${ROOT}/CHANGELOG.md`, 'utf8');
      const match = changelog.match(new RegExp(`## ${displayVer}[\\s\\S]*?(?=## V|$)`));
      return match ? match[0].trim() : `Release ${displayVer}`;
    } catch { return `Release ${displayVer}`; }
  })(),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: "",
      url: `https://github.com/kunkunr/yoshunko-admin-rust/releases/download/v${frontVer}/Yoshunko_Admin_${frontVer}_x64-setup.exe`
    }
  }
};

const latestPath = `${ROOT}/src-tauri/target/release/latest.json`;
fs.mkdirSync(require('path').dirname(latestPath), { recursive: true });
fs.writeFileSync(latestPath, JSON.stringify(latestJson, null, 2));
ok('latest.json generated');
```

> 注意：MSI 文件名格式需与 tauri build 输出一致。上线前构建一次验证确切文件名。

- [ ] **Step 2: 验证 dry-run**

```bash
node scripts/release.cjs --dry-run
```
期望：显示 latest.json 生成路径

- [ ] **Step 3: 提交**

```bash
git add scripts/release.cjs
git commit -m "feat: release 脚本生成 updater latest.json"
```

---

### Task 7: CI Release 构建

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: 创建 release workflow**

`.github/workflows/release.yml`:

```yaml
name: Release Build

on:
  push:
    tags: ['v*']

jobs:
  build-and-release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install frontend dependencies
        run: npm ci

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: npm run tauri build

      - name: Generate latest.json
        run: node scripts/release.cjs --ci

      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/nsis/*.exe
            src-tauri/target/release/latest.json
```

- [ ] **Step 2: 提交**

```bash
git add .github/workflows/release.yml
git commit -m "ci: 添加 tag push 自动构建发布 workflow"
```

---

### Task 8: 端到端集成验证

- [ ] **Step 1: 编译检查**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
npx vue-tsc --noEmit
```

- [ ] **Step 2: 运行测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
```
期望：12 passed

- [ ] **Step 3: 构建验证**

```bash
npm run build
cargo build --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: 开发模式验证**

```bash
npm run tauri dev
```
手动验证：
- 应用启动不报错
- TitleBar 不显示更新角标（开发环境无 endpoint）
- 设置页"手动检查更新"按钮不崩溃
- "手动下载"链接可点击

---

### 文件变更汇总

| 文件 | 操作 | 任务 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | Task 1 |
| `src-tauri/src/lib.rs` | 修改 | Task 1, 5 |
| `package.json` | 修改 | Task 1 |
| `src-tauri/tauri.conf.json` | 修改 | Task 2 |
| `src/lib/types.ts` | 修改 | Task 3 |
| `src/composables/useUpdater.ts` | 新建 | Task 3 |
| `src/App.vue` | 修改 | Task 3 |
| `src/components/layout/TitleBar.vue` | 修改 | Task 3 |
| `src/components/panels/SettingsPanel.vue` | 修改 | Task 3 |
| `src/styles/theme.css` | 修改 | Task 4 |
| `src-tauri/src/api/config.rs` | 修改 | Task 5 |
| `scripts/release.cjs` | 修改 | Task 6 |
| `.github/workflows/release.yml` | 新建 | Task 7 |
