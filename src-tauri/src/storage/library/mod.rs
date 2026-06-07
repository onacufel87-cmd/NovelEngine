//! 本地书库：统一下载/导入书籍的存储位置
//!
//! ```text
//! 默认：{当前用户 AppData}/com.novel.reader.core/library/
//! 可选：用户在设置中指定的任意文件夹（见 library_config.json）
//! ```

mod config;
mod info;
mod paths;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::utils::{AppError, AppResult};

use super::content_cache::set_content_cache_dir;
use super::db::{init_db_at, set_db_path};

pub use config::{clear_custom_root, default_library_root, resolve_library_root, set_custom_root};
pub use info::{build_path_info, LibraryPathInfo};
pub use paths::{imports_dir, index_db_path, library_root, texts_dir};

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 应用数据目录（Tauri 按用户+应用标识自动分配）
pub fn app_data_dir() -> Option<PathBuf> {
    APP_DATA_DIR.get().cloned()
}

/// 初始化书库，并迁移旧版 app_data 根目录下的数据
pub fn init(app_data_dir: &Path) -> AppResult<PathBuf> {
    let _ = APP_DATA_DIR.set(app_data_dir.to_path_buf());

    let root = config::resolve_library_root(app_data_dir);
    fs::create_dir_all(root.join("texts")).map_err(|e| {
        AppError::Database(format!("创建书库 texts 目录失败: {e}"))
    })?;
    fs::create_dir_all(root.join("imports")).map_err(|e| {
        AppError::Database(format!("创建书库 imports 目录失败: {e}"))
    })?;

    // 仅对默认位置做旧版迁移
    if root == config::default_library_root(app_data_dir) {
        migrate_legacy_layout(app_data_dir, &root)?;
    }

    let db_path = root.join("books.db");
    set_db_path(db_path.clone());
    init_db_at(&db_path)?;
    set_content_cache_dir(root.join("texts"));
    paths::set_library_root(root.clone());

    Ok(root)
}

pub fn path_info() -> Option<LibraryPathInfo> {
    let app_data = APP_DATA_DIR.get()?;
    Some(build_path_info(app_data, &library_root()))
}

/// 从 app_data/books.db、app_data/content_cache/ 迁移到 library/
fn migrate_legacy_layout(app_data: &Path, library: &Path) -> AppResult<()> {
    let legacy_db = app_data.join("books.db");
    let new_db = library.join("books.db");
    if legacy_db.exists() && !new_db.exists() {
        fs::rename(&legacy_db, &new_db).map_err(|e| {
            AppError::Database(format!("迁移书架数据库失败: {e}"))
        })?;
        for suffix in ["-wal", "-shm"] {
            let side = app_data.join(format!("books.db{suffix}"));
            if side.exists() {
                let _ = fs::rename(&side, library.join(format!("books.db{suffix}")));
            }
        }
    }

    let legacy_cache = app_data.join("content_cache");
    let new_texts = library.join("texts");
    if legacy_cache.is_dir() {
        if !new_texts.exists() {
            fs::rename(&legacy_cache, &new_texts).map_err(|e| {
                AppError::Database(format!("迁移正文缓存失败: {e}"))
            })?;
        } else {
            merge_book_cache_dirs(&legacy_cache, &new_texts)?;
            let _ = fs::remove_dir_all(&legacy_cache);
        }
    }

    Ok(())
}

fn merge_book_cache_dirs(from: &Path, to: &Path) -> AppResult<()> {
    let entries = fs::read_dir(from)
        .map_err(|e| AppError::Database(format!("读取旧正文缓存失败: {e}")))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dest = to.join(entry.file_name());
        if dest.exists() {
            continue;
        }
        fs::rename(&path, &dest).map_err(|e| {
            AppError::Database(format!("合并正文目录失败: {e}"))
        })?;
    }
    Ok(())
}
