# 本地书库说明

书籍**不会**保存在项目源码目录里，而是统一放在用户数据目录下的 **`library/`** 模块中。

## 路径规则

- **默认**：由 Tauri 按「当前 Windows 用户 + 应用 ID」自动分配，例如  
  `C:\Users\张三\AppData\Roaming\com.novel.reader.core\library\`  
  
- **自定义**：可在设置 → 本地书库 →「更改书库文件夹」指定任意目录（如 `D:\Novels`），保存后重启生效。

配置保存在 `{app_data}/library_config.json`，与书库数据分离。

```
%APPDATA%\com.novel.reader.core\library\
├── books.db          # SQLite：书架、章节目录、阅读进度、书源
├── texts/            # 在线书：阅读时下载的正文
│   └── {book_id}/
│       └── {chapter_id}.txt
└── imports/          # 本地导入的 EPUB/TXT 原件（预留）
```

## 代码模块

| 路径 | 职责 |
|------|------|
| `src-tauri/src/storage/library/` | 书库根目录初始化、旧数据迁移 |
| `src-tauri/src/storage/books.rs` | 书架 CRUD |
| `src-tauri/src/storage/chapters.rs` | 章节目录 |
| `src-tauri/src/storage/content_cache.rs` | 正文读写（实际路径为 `library/texts/`） |

## 旧版数据迁移

若你之前用过本应用，数据可能在：

```
%APPDATA%\com.novel.reader.core\books.db
%APPDATA%\com.novel.reader.core\content_cache\
```

**首次启动新版本**时会自动迁移到 `library/`，无需手动操作。

## 在应用内查看路径

设置页底部可看到「本地书库路径」，或调用命令 `get_library_path`。
