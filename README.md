# Yoshunko Admin

Tauri v2 桌面应用 — ZZZ 游戏存档数据管理工具

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6.svg)](#)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC131.svg)](https://tauri.app)

## 功能

角色、音擎、驱动盘、防卫战的面板化编辑管理，支持键盘快捷键操作和自动更新。

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

1. 启动应用后，在设置页面配置**状态目录**（包含 `player/` 子目录的游戏存档目录）
2. 应用将自动加载玩家列表和游戏数据
3. 从侧边栏选择面板开始编辑

## 数据格式

使用 **ZON**（Zig Object Notation）格式读写游戏存档，键名与结构与游戏客户端保持一致。

所有写操作采用原子写入（`.tmp` + `rename`），保留最近 5 份备份于 `.backup/` 目录，支持 `Ctrl+Z` 撤回。

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

本项目是独立开发的开源工具，与 HoYoverse / miHoYo 无关。不包含游戏客户端、服务器代码或加密密钥。

## License

MIT
