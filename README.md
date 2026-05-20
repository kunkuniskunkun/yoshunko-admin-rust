# Yoshunko Admin

Tauri v2 桌面应用 — ZZZ 游戏存档数据管理工具

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6.svg)](#)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC131.svg)](https://tauri.app)

## 功能

- 🎮 **角色管理** — 等级、影画、技能等级、潜能激发
- 🗡️ **音擎管理** — 等级、星级突破、精炼、复制与删除
- 💎 **驱动盘管理** — 创建、编辑、主/副属性、套装选择
- 🛡️ **防卫战管理** — Zone ID 切换、最新期号查询
- 👤 **玩家信息** — 昵称、等级、展示角色编辑
- 🚀 **快速启动** — 一键启动/停止游戏服务
- 🌙 **个性化主题** — 浅色/深色切换、6 种主题色、自定义背景图
- ⌨️ **键盘快捷键** — `Ctrl+S` 保存 · `Ctrl+Z` 撤回 · `Ctrl+F` 搜索 · 数字键切换面板
- 🔄 **自动更新** — 启动时自动检查新版本，一键安装

## 快速开始

### 系统要求

- Windows 10 / 11
- [Node.js](https://nodejs.org) 20+
- [Rust](https://www.rust-lang.org/tools/install) stable

### 从源码构建

```bash
git clone https://github.com/kunkuniskunkun/yoshunko-admin-rust.git
cd yoshunko-admin-rust
npm ci
npm run tauri dev        # 开发模式
```

```bash
npm run build            # 构建前端
cd src-tauri && cargo tauri build   # 打包 .msi 安装包
```

### 首次使用

1. 启动应用后，在设置页面配置**状态目录**（即包含 `player/` 子目录的游戏存档目录）
2. 应用将自动加载玩家列表和游戏数据
3. 从侧边栏选择面板即可开始编辑

## 数据格式

本工具使用 **ZON**（Zig Object Notation）格式读写游戏存档。存档数据的键名与结构与游戏客户端保持一致，可以直接编辑而不破坏存档完整性。

所有写操作均采用原子写入（`.tmp` + `rename`），并保留最近 5 份备份于 `.backup/` 目录中。支持 `Ctrl+Z` 撤回。

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri v2 |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript |
| UI 组件 | Naive UI |
| 样式 | Tailwind CSS 4 |
| 构建工具 | Vite 6 |

## 免责声明

本项目是独立开发的开源工具，与 **HoYoverse / miHoYo** 无关。

- 不包含游戏客户端、服务器代码或加密密钥
- 仅编辑本地游戏存档文件，不涉及网络传输
- 使用者自行承担数据备份和合规责任

## License

MIT
