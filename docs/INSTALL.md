# 安装与使用（普通用户）

> 无需安装 Node.js / Rust。从 [GitHub Releases](https://github.com/onacufel87-cmd/NovelEngine/releases) 下载即可。

## 系统要求

| 项目 | 要求 |
|------|------|
| 操作系统 | Windows 10 / 11（64 位） |
| WebView2 | Win10/11 通常已内置；极老系统需单独安装 |
| 磁盘 | 约 50 MB（安装版）；便携版约 15 MB |

## 推荐：安装版（`*-setup.exe`）

1. 在 Releases 页面下载 `小说引擎_*_x64-setup.exe`（或类似名称）。
2. 双击运行安装向导（支持简体中文 / English）。
3. 若系统缺少 **WebView2**，安装程序会**自动下载并安装**（有进度提示，需联网）。
4. 安装完成后：
   - **开始菜单** → `小说引擎` 文件夹 → `小说引擎`
   - **桌面** → `小说引擎` 快捷方式

卸载：Windows「设置 → 应用 → 已安装的应用」中卸载「小说引擎」。

## 备选：便携版（`novel-reader-core.exe`）

1. 下载 Release 中的 `novel-reader-core.exe`。
2. 放到任意目录，双击运行。
3. 若缺少 WebView2，程序会**弹窗提示**并给出官方下载链接。
4. 可选：运行仓库中的 **`创建桌面快捷方式.bat`**（需先本地打包，或传入 exe 路径）。

## 数据保存在哪？

书架、设置、正文缓存均在：

```
%APPDATA%\com.novel.reader.core\
├── books.db           # SQLite 索引
└── content_cache\     # 章节正文文件
```

与 exe / 安装目录无关，重装或移动程序不会丢数据。

## 常见问题

**Q：安装时没有提示，打开白屏或闪退？**  
A：多半是 WebView2 未就绪。请手动安装 [WebView2 运行时](https://go.microsoft.com/fwlink/p/?LinkId=2124703) 后重试。

**Q：杀毒软件报毒？**  
A：未签名的开源 Tauri 应用可能被误报，可在 Releases 核对 SHA256 后添加信任。

**Q：书源在哪？**  
A：官方仓库**不含**第三方书源。可在应用内订阅社区书源列表 URL，或自行导入 JSON（见 `rules/example_rule.json`）。

---

开发者构建说明见 [ENGINEERING.md](./ENGINEERING.md)。
