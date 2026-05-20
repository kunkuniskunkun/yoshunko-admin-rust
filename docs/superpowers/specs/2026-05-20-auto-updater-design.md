# 自动更新功能设计

> 设计日期: 2026-05-20 | 版本: v0.714+

## 一、目标

为 Yoshunko Admin 添加应用内自动更新功能。用户点击"检查更新"按钮，自动下载并安装新版本。更新内容包括新数据模板、新功能、Bug 修复等全部应用变更。

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
用户点"检查更新"
  → Rust 端调用 updater.check()
  → 请求 GitHub Releases 获取 latest version JSON
  → 比对本地版本号
  → 有新版 → 弹出确认框 → 下载 .msi → 静默安装 → 自动重启
  → 已是最新 → 提示"当前已是最新版本"
  → 网络错误 → 提示"检查更新失败"
  → 手动模式（备用）→ 打开浏览器到 GitHub Releases 页面
```

## 四、架构

```
┌────────────────────────────────────────┐
│  SettingsPanel.vue                      │
│  ┌──────────────────────────────┐       │
│  │ "检查更新" 按钮               │       │
│  │  - 调用 api.checkUpdate()    │       │
│  │  - 显示进度 / 结果            │       │
│  └──────────────────────────────┘       │
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

**`src/components/panels/SettingsPanel.vue`** — 新增更新区域：
```html
<!-- 更新部分 -->
<div class="settings-section">
  <h3>版本更新</h3>
  <p>当前版本: {{ version }}</p>
  <NButton @click="doCheckUpdate" :loading="checking">
    {{ checking ? '检查中...' : '检查更新' }}
  </NButton>
  <div v-if="updateAvailable" class="update-info">
    <p>新版本: {{ updateVersion }}</p>
    <p>{{ updateBody }}</p>
    <NButton type="primary" @click="doInstallUpdate" :loading="installing">
      {{ installing ? '下载安装中...' : '立即更新' }}
    </NButton>
  </div>
  <div v-if="updateMessage" class="update-message">{{ updateMessage }}</div>
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
| `src/components/panels/SettingsPanel.vue` | 修改 | 新增更新按钮 |
| `scripts/release.cjs` | 修改 | 增强 release 流程 |
| `.github/workflows/release.yml` | 新建 | release CI |

## 八、不做的

- **不自动推送更新**：不后台静默下载，用户手动点击才检查
- **不做增量更新**：每次下载完整 .msi（20-30MB 级别，不大）
- **不做跨平台更新**：仅 Windows
- **不做回滚功能**：用户可从 GitHub Releases 下载旧版手动安装
