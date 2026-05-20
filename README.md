# Yoshunko Admin

桌面工具 — 用来编辑绝区零的游戏存档

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6.svg)](#)

## 这个工具能做什么

如果你在玩绝区零的私服，这个工具可以帮你编辑存档：修改角色等级和技能、添加音擎和驱动盘、切换防卫战关卡等等。所有操作都在界面上点点就行，不用手动改文件。

目前支持的最新客户端版本：**ZZZ3.0.3Beta**

## 怎么安装

### 普通用户（推荐）

1. 打开 [Releases 页面](https://github.com/kunkuniskunkun/yoshunko-admin-rust/releases)
2. 下载最新版本的 `.exe` 安装包
3. 双击运行，一路点下一步即可
4. 以后有新版本会提示你更新，点一下就能升级

### 开发者（从代码构建）

需要安装 [Node.js](https://nodejs.org) 20+ 和 [Rust](https://www.rust-lang.org/tools/install)。

```bash
git clone https://github.com/kunkuniskunkun/yoshunko-admin-rust.git
cd yoshunko-admin-rust
npm ci
npm run tauri dev
```

## 怎么使用

1. 打开软件后，先点左侧的「设置」
2. 在「状态目录」那里，选择你放游戏存档的文件夹（就是里面有个 `player` 子目录的那个）
3. 配好后，左侧会列出所有玩家，选一个就能编辑了
4. 数字键 1-7 可以快速切换面板，`Ctrl+S` 保存，`Ctrl+Z` 撤回

## 安全吗

- 每次保存会自动备份（保留最近 5 份），改错了可以找回来
- 撤回到上一步操作前
- 这个工具只读写你电脑上的存档文件，不上传任何数据到网络

## 免责声明

本项目与 HoYoverse / miHoYo 无关，不包含游戏客户端或服务器代码。

## License

MIT
