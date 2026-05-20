# Yoshunko Admin

Tauri v2 桌面应用 — ZZZ 游戏数据管理工具

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6.svg)](#)

## 功能

- 🎮 **角色管理** — 等级、影画、技能、潜能激发
- 🗡️ **音擎管理** — 等级、星级突破、精炼、复制/删除
- 💎 **驱动盘管理** — 创建、编辑、主/副属性配置、套装选择
- 🛡️ **防卫战管理** — Zone ID 切换、最新期号查询
- 👤 **玩家信息** — 昵称、等级、展示角色编辑
- 🚀 **快速启动** — 一键启动/停止游戏服务
- 🌙 **主题切换** — 浅色/深色一键切换 + 6 种主题色 + 自定义背景图
- ⌨️ **键盘快捷键** — `Ctrl+S` 保存 / `Ctrl+Z` 撤回 / `Ctrl+F` 搜索 / 数字键导航
- 🔄 **自动更新** — 启动时自动检查新版本，一键安装

## 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Tauri v2 |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript |
| UI | Naive UI + Tailwind CSS 4 |
| 数据格式 | ZON (Zig Object Notation) |

## 开发环境

- Windows 10/11
- Node.js 20+
- Rust stable

```bash
git clone https://github.com/kunkuniskunkun/yoshunko-admin-rust.git
cd yoshunko-admin-rust
npm ci
npm run tauri dev
```

构建：

```bash
npm run build
cd src-tauri && cargo tauri build
```

## 免责声明

本项目是独立开发的开源工具，与 HoYoverse / miHoYo 无关。不包含游戏客户端、服务器代码或加密密钥。

## License

MIT
