# 工程方案：GitHub 发布与维护

本文档描述「小说引擎」开源仓库的目标结构、发布流程与裁剪原则，便于长期维护。

## 1. 目标

| 角色 | 体验 |
|------|------|
| **最终用户** | 从 GitHub Releases 下载 `*-setup.exe`，双击安装，开始菜单 + 桌面快捷方式，WebView2 由安装包处理 |
| **贡献者** | `git clone` → `npm install` → `npm run tauri dev`；CI 自动跑测试与构建 |
| **维护者** | 打 tag `v0.1.0` → Actions 产出 Release 草稿 → 核对后发布 |

## 2. 技术栈（精简后）

```
┌─────────────────────────────────────────┐
│  Vue 3 + Pinia + Vite（前端 UI）         │
├─────────────────────────────────────────┤
│  Tauri 2（桌面壳 + IPC）                 │
├─────────────────────────────────────────┤
│  Rust：spider / crawler / storage       │
│  SQLite + 文件缓存 · zhconv 简繁        │
└─────────────────────────────────────────┘
```

**已移除的冗余：**

- `opencc-js` / `src/utils/opencc.js`（简繁已在 Rust `zhconv`）
- `codemirror` 重复包、`@codemirror/language`（未使用）
- `@tauri-apps/plugin-opener`（前端无调用）
- `legacy` 命令、`crawler/preload`、`crawler/downloader`（无引用）
- `public/test-*`（移至 `dev-fixtures/`，仅开发模式提供）
- 模板 SVG（`tauri.svg`、`vite.svg`、`logo.svg`）

## 3. 目录结构

```
小说引擎/
├── .github/workflows/
│   ├── ci.yml              # push/PR：测试 + 构建
│   └── release.yml         # tag v*：NSIS + exe 上传 Release
├── docs/
│   ├── INSTALL.md          # 用户安装说明
│   └── ENGINEERING.md      # 本文件
├── dev-fixtures/           # 开发用 HTML/JSON 测试页（不进 release）
├── public/                 # 仅 app-icon.png 等生产静态资源
├── rules/example_rule.json   # 书源模板（可提交）
├── scripts/                # 图标处理、桌面快捷方式等
├── src/                    # Vue 前端
├── src-tauri/
│   ├── tauri.conf.json     # NSIS + WebView2 embedBootstrapper
│   └── windows/
│       └── installer-hooks.nsh  # 安装后创建桌面快捷方式
├── 一键打包.bat / 一键运行.bat
├── 创建桌面快捷方式.bat
├── .nvmrc                  # Node 20
└── rust-toolchain.toml     # stable
```

## 4. Windows 安装包配置

`src-tauri/tauri.conf.json` 关键项：

| 配置 | 值 | 作用 |
|------|-----|------|
| `bundle.targets` | `["nsis"]` | 仅 NSIS，避免 MSI 等重复产物 |
| `webviewInstallMode.type` | `embedBootstrapper` | 安装时嵌入 WebView2 引导程序 |
| `webviewInstallMode.silent` | `false` | 缺失 WebView2 时显示安装进度 |
| `nsis.languages` | `SimpChinese`, `English` | 安装向导双语 |
| `nsis.startMenuFolder` | `小说引擎` | 开始菜单分组 |
| `installer-hooks.nsh` | POSTINSTALL | 创建桌面 `.lnk` |

便携版 `novel-reader-core.exe`：`src-tauri/src/utils/webview2.rs` 在启动前检测注册表，缺失则弹窗并退出。

## 5. CI / Release

### CI（`.github/workflows/ci.yml`）

- 触发：`push` / `pull_request` → `main` / `master`
- 步骤：`npm ci` → `cargo test` → `npm run build` → `npm run tauri build`

### Release（`.github/workflows/release.yml`）

- 触发：`push` tag `v*`（如 `v0.1.0`）
- 使用 `tauri-apps/tauri-action@v0` 上传：
  - `*-setup.exe`（推荐）
  - `novel-reader-core.exe`（便携）
- 默认 `releaseDraft: true`，维护者检查后点 Publish

**发布命令示例：**

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 6. 本地开发环境（仅打包机器需要）

| 依赖 | 说明 |
|------|------|
| Node.js 18+ | 见 `.nvmrc`（20） |
| npm | `npm install` |
| Rust stable | `rustup.rs` + `rust-toolchain.toml` |
| Windows MSVC | Visual Studio Build Tools |
| WebView2 | 开发机通常已有 |

```bash
npm install
npm run tauri dev      # 开发
npm run tauri build    # 或双击 一键打包.bat
npm run test:rust      # Rust 单元测试
```

产物路径：

- 便携：`src-tauri/target/release/novel-reader-core.exe`
- 安装包：`src-tauri/target/release/bundle/nsis/*-setup.exe`

## 7. 上传 GitHub 前检查清单

- [x] 仓库地址：`https://github.com/onacufel87-cmd/NovelEngine`
- [ ] `git init` → 添加 `.gitignore`（已含 `node_modules`、`target`、`*.db`）
- [ ] 确认 `rules/*.json` 除 `example_rule.json` 外不被提交
- [ ] 首次 Release 前本地跑通 `npm run tauri build`
- [ ] 在 GitHub 创建仓库并 push；打 tag 触发 Release Action

## 8. 版本与变更

- 版本号：`package.json`、`src-tauri/Cargo.toml`、`tauri.conf.json` 保持一致
- 建议后续增加 `CHANGELOG.md`，Release 时粘贴变更摘要

## 9. 刻意不做的（控制范围）

- 不内置 Android / iOS / Linux 打包矩阵（当前仅 Windows NSIS）
- 不托管第三方书源 JSON
- 不在仓库内放 `dist/`、`target/`、用户 `books.db`
- 不引入额外状态管理或 UI 框架

---

维护原则：**一条发布路径（NSIS）、一套依赖（Tauri + Vue + Rust）、开发夹具与生产资源分离。**
