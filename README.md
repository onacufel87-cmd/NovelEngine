# 小说引擎 · Novel Reader Core

> **通用书源解析引擎 + 纯净桌面阅读器**  
> Tauri 2 · Rust · Vue 3 · SQLite

---

## 下载即用（推荐）

**普通用户无需 clone 仓库。**

1. 打开 [GitHub Releases](https://github.com/onacufel87-cmd/NovelEngine/releases)
2. 下载 **`小说引擎_*_x64-setup.exe`**
3. 双击安装 → 开始菜单 / 桌面会出现「小说引擎」

详细说明与常见问题见 **[docs/INSTALL.md](docs/INSTALL.md)**。

| 文件 | 说明 |
|------|------|
| `*-setup.exe` | **推荐**：自动处理 WebView2，含快捷方式 |
| `novel-reader-core.exe` | 便携版，需系统已有 WebView2 |

**系统要求**：Windows 10 / 11（64 位）

---

## Disclaimer

本软件为通用解析框架，**官方仓库不含任何第三方盗版书源**。仅供合法公版资源（如 Project Gutenberg、维基文库）与技术研究使用。用户自行导入书源时须遵守当地法律法规。

---

## 功能概览

- 书架、阅读进度、EPUB/TXT 导入
- 8 种护眼主题、简繁转换（Rust `zhconv`）、正文清洗与拼音还原
- JSON 书源规则、仓库 URL 订阅、启发式自动检测
- 4 个内置合法公版源；SQLite 索引 + 文件系统正文缓存

---

## 开发者

```bash
npm install
npm run tauri dev      # 开发
npm run tauri build    # 打包 NSIS + exe
npm run test:rust      # cargo test
```

Windows 可双击 **`一键打包.bat`** / **`一键运行.bat`**。

| 依赖 | 版本 |
|------|------|
| Node.js | 18+（见 `.nvmrc`） |
| Rust | stable（见 `rust-toolchain.toml`） |
| 平台 | Windows + MSVC |

架构、CI/Release、裁剪说明见 **[docs/ENGINEERING.md](docs/ENGINEERING.md)**。

书源规则模板：[rules/example_rule.json](rules/example_rule.json)

---

## License

MIT — 详见 [LICENSE](./LICENSE)
