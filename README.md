# 小说引擎 · Novel Reader Core

> 通用书源解析引擎 + 纯净桌面阅读器  
> Tauri 2 · Rust · Vue 3 · SQLite

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/onacufel87-cmd/NovelEngine?label=release)](https://github.com/onacufel87-cmd/NovelEngine/releases/latest)

**小说引擎** 是一款 Windows 桌面阅读器，内置 Rust 书源解析引擎。支持书架管理、在线阅读、本地 EPUB/TXT 导入、JSON 书源规则与自动检测接入。数据全部保存在本机，无需账号。

---

## 下载安装

**普通用户无需 clone 仓库。**

👉 **[下载最新版](https://github.com/onacufel87-cmd/NovelEngine/releases/latest)**

| 文件 | 说明 |
|------|------|
| `小说引擎_*_x64-setup.exe` | **推荐**：NSIS 安装包，自动处理 WebView2，含开始菜单与桌面快捷方式 |
| `novel-reader-core.exe` | 便携版，需系统已安装 WebView2 |

**安装步骤**

1. 打开 [Releases](https://github.com/onacufel87-cmd/NovelEngine/releases/latest)，在 **Assets** 下载 `*-setup.exe`
2. 双击运行安装向导（简体中文 / English）
3. 安装完成后，从开始菜单或桌面启动「小说引擎」

**系统要求**：Windows 10 / 11（64 位）

> Release 尚未发布时页面可能为空，可在 [Actions](https://github.com/onacufel87-cmd/NovelEngine/actions) 查看构建进度，维护者 Publish 后再下载。

安装细节与常见问题见 **[docs/INSTALL.md](docs/INSTALL.md)**。

---

## 主要功能

### 阅读体验

- 书架管理、章节目录、阅读进度与滚动位置记忆
- 顶栏 + 底栏常驻导航（上一章 / 目录 / 下一章）
- 8 种护眼主题，支持跟随系统深色模式
- 简繁转换、正文清洗、拼音占位符还原
- 划词笔记，全书笔记汇总
- 导出 TXT（仅已缓存章节，后台执行不卡界面）

### 书源与解析

- **JSON 规则驱动**：CSS 选择器定义搜索、目录、正文、分页（模板见 [rules/example_rule.json](rules/example_rule.json)）
- **内置 4 个合法公版源**：中文维基文库、Open Library、Project Gutenberg、Standard Ebooks
- **远程订阅**：填写书源仓库 URL，一键同步更新
- **自动检测**：粘贴目录页 + 正文页 URL，启发式推断选择器
- **健康检测**：ping 书源可用性与响应速度

### 本地书库

- EPUB / TXT 本地导入
- SQLite 索引 + 文件系统正文缓存
- 可自定义书库文件夹（设置 → 本地书库），默认位于用户 AppData

---

## 快速上手

```
下载安装 → 打开应用 → 发现页搜索 / 导入本地书 → 加入书架 → 开始阅读
```

**书源在哪？** 官方仓库**不含**第三方书源。应用内可启用内置公版源，或自行导入 JSON 规则、订阅社区书源列表 URL。

---

## 数据存储

书籍与设置保存在用户数据目录，**与安装路径无关**，重装不丢数据：

```
%APPDATA%\com.novel.reader.core\
├── library_config.json       # 自定义书库路径（可选）
└── library\
    ├── books.db              # 书架、章节、进度、书源
    ├── texts/                # 在线书正文缓存
    └── imports/              # 本地导入原件
```

导出 TXT 默认保存至 `文档\NovelReaderCore\`。  
可在设置页查看当前书库路径，或阅读 **[docs/STORAGE.md](docs/STORAGE.md)**。

---

## 技术架构

```
┌─────────────────────────────────────────┐
│  Vue 3 + Pinia + Vite（UI / 阅读 / 书源）│
├─────────────────────────────────────────┤
│  Tauri 2（桌面壳 + IPC）                 │
├─────────────────────────────────────────┤
│  Rust：spider 解析 · crawler 搜索       │
│  SQLite + 文件缓存 · zhconv 简繁        │
└─────────────────────────────────────────┘
```

| 模块 | 路径 | 职责 |
|------|------|------|
| 前端 | `src/` | 书架、阅读、发现、书源管理、设置 |
| 解析引擎 | `src-tauri/src/spider/` | 抓取、HTML 解析、清洗、简繁、拼音还原 |
| 存储 | `src-tauri/src/storage/` | SQLite、书库路径、正文缓存 |
| 命令层 | `src-tauri/src/commands/` | Tauri invoke 接口 |

---

## 本地开发

**依赖**

| 工具 | 版本 |
|------|------|
| Node.js | 18+（见 `.nvmrc`，推荐 20） |
| Rust | stable（见 `rust-toolchain.toml`） |
| 平台 | Windows + MSVC |

**命令**

```bash
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 打包 NSIS + 便携 exe
npm run test:rust      # Rust 单元测试
```

Windows 也可双击 **`一键运行.bat`** / **`一键打包.bat`**。

**产物路径**

- 便携版：`src-tauri/target/release/novel-reader-core.exe`
- 安装包：`src-tauri/target/release/bundle/nsis/*-setup.exe`

**发布 Release**

```bash
git tag v0.1.0
git push origin v0.1.0
```

打 tag 后 GitHub Actions 自动构建并上传 Release 草稿，维护者核对后 Publish。

更多工程细节见 **[docs/ENGINEERING.md](docs/ENGINEERING.md)**。

---

## 项目结构

```
小说引擎/
├── src/                      # Vue 前端
├── src-tauri/                # Rust 后端 + Tauri 配置
│   ├── src/spider/           # 书源解析引擎
│   ├── src/storage/          # 书库与 SQLite
│   └── resources/            # 内置公版书源
├── rules/example_rule.json   # 书源规则模板
├── docs/                     # 安装、工程、存储说明
├── dev-fixtures/             # 开发测试页（不进 Release）
└── .github/workflows/        # CI + Release 自动化
```

---

## 免责声明

本软件为**通用解析框架**，官方仓库不含任何第三方盗版书源，仅供合法公版资源（如 Project Gutenberg、维基文库）与技术研究使用。**用户自行导入书源时须遵守当地法律法规**，作者不对用户行为负责。

---

## 文档

| 文档 | 内容 |
|------|------|
| [docs/INSTALL.md](docs/INSTALL.md) | 安装、卸载、常见问题 |
| [docs/STORAGE.md](docs/STORAGE.md) | 书库路径、数据迁移 |
| [docs/ENGINEERING.md](docs/ENGINEERING.md) | CI/Release、目录结构、维护原则 |
| [rules/example_rule.json](rules/example_rule.json) | 书源 JSON 规则模板 |

---

## License

[MIT](LICENSE)
