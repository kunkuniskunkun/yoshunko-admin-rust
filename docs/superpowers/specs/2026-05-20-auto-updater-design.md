# 自动更新功能设计

> 设计日期: 2026-05-20 | 版本: v0.714+

## 一、目标

为 Yoshunko Admin 添加应用内自动更新功能。应用启动时后台自动检查新版本，发现有更新后通知用户，用户自主决定是否立即安装。更新内容包括新数据模板、新功能、Bug 修复等全部应用变更。

## 二、选型

使用 **tauri-plugin-updater**（Tauri 官方更新插件），后端依托 **GitHub Releases** 作为分发服务器。

**理由**：
- 官方维护，与 Tauri v2 兼容
- 支持 Windows .msi 静默安装
- GitHub Releases 免费提供 CDN 和版本 JSON
- 不需要额外服务器、域名、数据库
- 已有一个 `.github/workflows/ci.yml`，只需新增 release 构建步骤

## 三、更新流程

```
应用启动
  → 后台静默调用 updater.check()
  → 请求 GitHub Releases 获取最新版本 JSON
  → 比对本地版本号
  → 无新版 → 不做任何提示
  → 有新版 → 更新按钮出现角标，提示"发现新版本 vX.XX"
  → 用户点击 → 弹出更新详情（版本号 + changelog 摘要）→ 选择：
       [立即更新] → 下载 .msi（显示进度）→ 静默安装 → 自动重启
       [稍后提醒] → 关闭弹窗，下次启动再次通知
  → 网络错误 → 静默失败，不做任何提示
  → 手动模式（备用）→ 设置页"手动下载"链接 → 浏览器打开 GitHub Releases
```

## 四、架构

```
┌────────────────────────────────────────┐
│  App.vue (onMounted)                    │
│  启动时异步调用 api.checkUpdate()       │
│  发现新版 → 存入 updateInfo ref         │
├────────────────────────────────────────┤
│  TitleBar.vue                           │
│  app-update-badge (角标 + 版本号)        │
│  点击 → 弹出更新详情 Modal              │
├────────────────────────────────────────┤
│  SettingsPanel.vue                      │
│  "手动检查更新" 按钮（手动触发）          │
│  "手动下载" 链接（跳转 GitHub Releases） │
├────────────────────────────────────────┤
│  lib/api.ts                             │
│  checkUpdate() / installUpdate()        │
├────────────────────────────────────────┤
│  Rust api/update.rs                     │
│  pub fn check_update() -> Value         │
│  pub fn install_update() -> Value       │
├────────────────────────────────────────┤
│  tauri-plugin-updater                   │
│  - 读取 updater 配置（tauri.conf.json）  │
│  - 查询远端 JSON endpoint               │
│  - 下载 .msi                           │
│  - 调用 Windows Installer 静默安装       │
└────────────────────────────────────────┘
```

## 五、文件改动

### 5.1 Rust 后端

**`src-tauri/Cargo.toml`** — 新增依赖：
```toml
tauri-plugin-updater = "2"
```

**`src-tauri/tauri.conf.json`** — 新增 updater 配置块：
```json
"plugins": {
  "updater": {
    "endpoints": [
      "https://github.com/<owner>/<repo>/releases/latest/download/latest.json"
    ],
    "pubkey": "<签名公钥>"
  }
}
```

公钥机制：构建时生成密钥对，私钥本地签名，公钥内置于应用。更新包必须通过签名验证才安装，防止篡改。

**`src-tauri/src/api/update.rs`** — 新增更新命令：
```rust
use tauri::State;
use serde_json::{json, Value};

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Value {
    let updater = app.updater().ok_or_else(|| "updater plugin not configured")?;
    match updater.check().await {
        Ok(Some(update)) => json!({
            "ok": true,
            "has_update": true,
            "version": update.version,
            "body": update.body,
            "date": update.date
        }),
        Ok(None) => json!({"ok": true, "has_update": false}),
        Err(e) => json!({"ok": false, "error": e.to_string()})
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Value {
    let updater = app.updater().ok_or_else(...)?;
    match updater.check().await {
        Ok(Some(update)) => {
            update.download_and_install(|progress| { ... }, || { ... }).await;
            json!({"ok": true})
        }
        _ => json!({"ok": false, "error": "no update available"})
    }
}
```

### 5.2 前端

**`src/lib/api.ts`** — 新增封装：
```typescript
export const api = {
  // ... existing ...
  checkUpdate: () => invoke<UpdateResult>('check_update'),
  installUpdate: () => invoke<InstallResult>('install_update'),
}
```

**`src/App.vue`** — onMounted 中启动后台检查：
```typescript
onMounted(async () => {
  // ... existing init ...
  // 后台静默检查更新
  const result = await api.checkUpdate()
  if (result.has_update) {
    updateInfo.value = result  // 触发 TitleBar 角标
  }
})
```

**`src/components/layout/TitleBar.vue`** — 新增更新角标：
```html
<!-- 版本更新角标，有新版时显示 -->
<span v-if="updateVersion" class="app-update-badge" @click="showUpdateModal = true">
  v{{ updateVersion }}
</span>
<!-- 点击弹出更新详情 Modal -->
<NModal v-model:show="showUpdateModal" title="发现新版本">
  <p>{{ updateBody }}</p>
  <div class="modal-actions">
    <NButton @click="showUpdateModal = false">稍后提醒</NButton>
    <NButton type="primary" @click="doInstall" :loading="installing">
      {{ installing ? '下载安装中...' : '立即更新' }}
    </NButton>
  </div>
</NModal>
```

**`src/components/panels/SettingsPanel.vue`** — 更新区域简化：
```html
<div class="settings-section">
  <h3>版本更新</h3>
  <p>当前版本: {{ version }}</p>
  <NButton @click="doCheckUpdate" :loading="checking">
    {{ checking ? '检查中...' : '检查更新' }}
  </NButton>
  <NButton text @click="openReleasePage">手动下载</NButton>
</div>
```

### 5.3 构建配置

**`tauri.conf.json`** — 新增 bundle 配置（签名相关）：
```json
"bundle": {
  "windows": {
    "wix": {
      "language": "zh-CN"
    }
  },
  "updater": {
    "active": true
  }
}
```

**`scripts/release.cjs`** — 增强现有 release 脚本：
在构建完成后自动生成 `latest.json`：
```javascript
// 生成更新清单 JSON
const latestJson = {
  version: pkg.version,
  notes: getChangelogEntry(changelog, displayVer),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(sigPath, 'utf8'),
      url: `https://github.com/${owner}/${repo}/releases/download/v${pkg.version}/YoshunkoAdmin_${pkg.version}_x64_zh-CN.msi`
    }
  }
};
```

### 5.4 CI

**`.github/workflows/release.yml`** — 新增 release 触发构建：
```yaml
name: Release Build
on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: npm run tauri build
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/latest.json
```

## 六、注意事项

- **签名验证**：生产环境必须启用私钥签名，防止中间人替换安装包
- **首次跳过**：如果 updater plugin 没有正确配置 endpoint（如本地开发环境），检查更新应静默失败，不给用户报错
- **权限**：安装位置在 Program Files，更新安装时需要管理员权限（MSI 会自动触发 UAC）
- **降级**：不自动更新，需用户确认后才下载安装
- **手动回退**：SettingsPanel 提供"手动下载"链接，跳转到 GitHub Releases 页面

## 七、变更范围

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | 加 `tauri-plugin-updater` |
| `src-tauri/tauri.conf.json` | 修改 | 加 updater endpoint 配置 |
| `src-tauri/src/api/update.rs` | 新建 | 更新命令 |
| `src-tauri/src/api/mod.rs` | 修改 | 注册 update 模块 |
| `src-tauri/src/lib.rs` | 修改 | 注册 updater plugin |
| `src/lib/api.ts` | 修改 | 新增 update API |
| `src/lib/types.ts` | 修改 | 新增 UpdateResult 类型 |
| `src/App.vue` | 修改 | onMounted 自动检查更新 |
| `src/components/layout/TitleBar.vue` | 修改 | 新增更新角标 + Modal |
| `src/components/panels/SettingsPanel.vue` | 修改 | 手动检查 + 手动下载链接 |
| `scripts/release.cjs` | 修改 | 增强 release 流程 |
| `.github/workflows/release.yml` | 新建 | release CI |

## 八、不做的

- **不强制更新**：用户可以选择"稍后提醒"，下次启动再通知
- **不后台下载**：用户点击"立即更新"后才开始下载
- **不做增量更新**：每次下载完整 .msi（20-30MB 级别，不大）
- **不做跨平台更新**：仅 Windows
- **不做回滚功能**：用户可从 GitHub Releases 下载旧版手动安装
