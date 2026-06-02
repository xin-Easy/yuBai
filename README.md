<p align="center">
  <img src="https://raw.githubusercontent.com/xin-Easy/yuBai/main/src-tauri/icons/logo.png" width="200" alt="yubai logo" />
</p>

<h1 align="center">🌐 yubai</h1>

<p align="center">
  <strong>面向多账号隔离、代理绑定和本地浏览器环境管理的桌面工作台</strong>
</p>

<p align="center">
  <a href="#快速开始">
    <img src="https://img.shields.io/badge/release-v0.1.0-blue" alt="release" />
  </a>
  <a href="#技术栈">
    <img src="https://img.shields.io/badge/Core-Rust-orange" alt="core" />
  </a>
  <a href="#技术栈">
    <img src="https://img.shields.io/badge/Language-TypeScript-blue" alt="language" />
  </a>
  <img src="https://img.shields.io/badge/License-MIT-green" alt="license" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Vue-3.x-42b883?logo=vue.js" alt="Vue 3" />
  <img src="https://img.shields.io/badge/Tauri-2.x-24c8db?logo=tauri" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/TypeScript-5.x-3178c6?logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Element_Plus-2.x-409eff?logo=elementplus" alt="Element Plus" />
  <img src="https://img.shields.io/badge/UnoCSS-4x-black?logo=unocss" alt="UnoCSS" />
</p>

---

**yubai** 是一个基于 **Tauri 2 + Rust + Vue 3** 的本地桌面应用，用来集中管理浏览器实例、浏览器内核、代理池、默认书签、自动化运行时和本地 Launch API。项目重点放在**本地数据可控**、**实例环境隔离**、**代理链路可检测**，以及桌面端日常操作的效率。

无需复杂的服务端部署，只需优雅的桌面客户端，即可实现多账号浏览器实例管理、代理绑定与检测、指纹参数配置、自动化脚本调度。

<p align="center">
  <a href="#快速开始">🚀 快速开始</a> ·
  <a href="#功能特性">✨ 核心特性</a> ·
  <a href="#技术栈">🛠 技术栈</a> ·
  <a href="#项目结构">📁 项目结构</a> ·
  <a href="#配置说明">⚙️ 配置说明</a>
</p>

---

## 功能特性

> 🧩 **核心引擎说明：** 本项目（yubai）是 Tauri 2 的可视化操作界面。

核心采集与自动化能力由 **Rust 后端** 强力驱动。

👉 [点击前往 Rust 核心仓库](src-tauri/src) 了解底层实现与 API 文档。

### 浏览器管理

| 能力 | 说明 |
|------|------|
| 🖥️ **实例管理** | 创建、编辑、启动、停止浏览器实例，支持标签、分组、关键词配置 |
| 🔧 **内核管理** | 扫描本机内核、注册系统浏览器、校验路径、下载可用内核 |
| 👆 **指纹参数** | 为实例维护指纹参数、启动参数、默认启动页和会话恢复配置 |
| 📸 **快照与 Cookie** | 支持浏览器快照创建/恢复/删除，Cookie 读取、清理和导出 |

### 代理体系

| 能力 | 说明 |
|------|------|
| 🌍 **代理池** | 代理导入、Clash 订阅拉取、订阅刷新、批量测速、IP 健康检查 |
| 🔗 **实例绑定** | 代理节点与浏览器实例的绑定关系查看与管理 |
| 🔌 **浏览器扩展** | 内置 Proxy Checker 扩展，实时检测网络出口 IP |

### 自动化与 API

| 能力 | 说明 |
|------|------|
| ⚡ **自动化脚本** | 提供运行时状态、脚本列表、运行记录和安装入口 |
| 🚀 **Launch API** | 内置本地 HTTP API 服务，外部调用启动指定浏览器环境 |
| 💾 **数据存储** | SQLite 保存实例、代理、内核、书签和分组数据 |

---

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面框架 | **Tauri 2** — 轻量级跨平台桌面壳 |
| 后端 | **Rust 2021**, Diesel ORM, SQLite, Axum, Tokio 异步运行时 |
| 前端 | **Vue 3**, TypeScript, Vue Router, Pinia 状态管理 |
| UI | **Element Plus**, **UnoCSS**, SCSS |
| 构建工具 | **Vite**, **pnpm** |

---

## 项目结构

```
yubai/
├── public/
│   └── icon.png                  # 应用图标（AI 抠图生成）
├── src/                         # Vue 3 前端
│   ├── api/                     # Tauri command 调用封装
│   ├── layouts/                 # 应用壳、顶栏、侧边栏
│   ├── pages/                   # 控制台、实例、代理、内核、设置等页面
│   ├── router/                  # 路由与导航配置
│   ├── store/                   # Pinia 状态
│   ├── styles/                  # 全局样式
│   └── types/                   # 前端类型定义
├── src-tauri/                   # Tauri / Rust 后端
│   ├── resources/extensions/    # 浏览器扩展资源（proxy-checker）
│   ├── icons/                   # 多平台图标（PNG / ICO / ICNS）
│   ├── migrations/              # SQLite 数据库迁移
│   ├── src/
│   │   ├── app/                 # AppState、服务组装、运行态组合根
│   │   ├── commands/            # 暴露给前端的 Tauri commands
│   │   ├── services/            # 应用业务编排层
│   │   ├── infra/               # 数据库、文件、网络、进程等底层实现入口
│   │   ├── db/                  # 现阶段保留的数据库 store 和 schema 实现
│   │   ├── core/                # 旧实现承载层，逐步迁移到 infra
│   │   ├── domain/              # 配置、路径、领域模型
│   │   └── state.rs             # 对 app/state.rs 的兼容导出
│   └── tauri.conf.json          # Tauri 配置
├── docs/                        # 设计、迁移和本地开发文档
├── config.yaml                  # 默认本地配置
└── package.json                 # 前端脚本与依赖
```

## 快速开始

### 环境要求

- **Node.js** 20+
- **pnpm** 11.x
- **Rust stable**
- Tauri 2 所需的系统依赖（WebView、编译工具链）

### 安装与运行

```bash
# 安装依赖
pnpm install

# 启动前端开发服务
pnpm dev

# 启动桌面开发模式（前端 + Tauri）
pnpm tauri:dev

# 构建生产版本
pnpm build && pnpm tauri:build

# 仅检查 Rust 后端编译
pnpm rust:check

# 查看 Tauri 环境信息
pnpm tauri:info
```

## 配置说明

默认配置文件为 `config.yaml`。应用运行时会在本地应用数据目录创建实际配置、数据库、日志和浏览器实例数据。

常用目录约定：

| 路径 | 用途 |
|------|------|
| `data/app.db` | SQLite 数据库 |
| `data/profiles` | 浏览器实例用户数据目录 |
| `data/app.log` | 应用日志 |
| `src-tauri/resources/extensions/proxy-checker` | 代理检测浏览器扩展资源 |

## 应用页面

| 页面 | 说明 |
|------|------|
| 📊 **控制台** | 查看实例、代理、内核、自动化和运行时状态总览 |
| 🖥️ **实例列表** | 集中管理浏览器实例、代理绑定和运行状态 |
| ⚡ **自动化脚本** | 查看自动化运行时、脚本数量和最近执行记录 |
| 🔧 **内核管理** | 扫描、注册、下载和校验浏览器内核 |
| 🌍 **代理池配置** | 导入、测试、刷新和维护代理节点 |
| 🔖 **默认书签** | 维护浏览器实例初始化使用的书签 |
| 🏷️ **标签管理** | 管理实例分组和标签 |
| ⚙️ **系统设置** | 调整目录、默认参数、代理模式和浏览器启动行为 |
| 📋 **日志查看** | 查看和清理本地应用日志 |

## 开发约定

- 前端通过 `src/api/*` 调用 Rust 后端的 **Tauri commands**
- Rust command 统一在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册
- Tauri 启动组装放在 `src-tauri/src/app/bootstrap.rs`
- Rust 后端正在从 `core + db + state` 结构迁移到 `app + commands + services + infra + domain`
- 数据模型优先保持 **camelCase JSON contract**，便于前端直接消费
- 长耗时任务使用 **async command** 或 `tokio::task::spawn_blocking`，避免阻塞桌面窗口
- 数据库结构变更应同步更新 `src-tauri/migrations` 和 `src-tauri/src/db/schema.rs`

## 文档

- [本地开发命令](docs/local-commands.md)
- [代理桥接运行时](docs/proxy-bridge-runtime.md)
- [迁移方案](docs/ant-browser-to-yubai-migration.md)
- [迁移状态](docs/migration-status-update.md)
- [剩余迁移工作](docs/remaining-migration-work.md)

## 状态

项目仍在快速迭代阶段。当前已包含：桌面壳、SQLite 数据模型、核心页面、浏览器实例管理、代理池、内核管理、自动化状态和 Launch API 等模块；部分文档中心、自动化脚本执行和迁移收尾工作仍在持续完善。

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=xin-Easy/yubai&type=Timeline)](https://star-history.com/#xin-Easy/yubai&Date)
