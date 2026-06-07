//! 书库路径常量

use std::path::PathBuf;
use std::sync::OnceLock;

static LIBRARY_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// 记录书库根路径
pub fn set_library_root(path: PathBuf) {
    let _ = LIBRARY_ROOT.set(path);
}

/// 书库根目录：{app_data}/library/
pub fn library_root() -> PathBuf {
    LIBRARY_ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("library"))
}

/// 索引库路径
pub fn index_db_path() -> PathBuf {
    library_root().join("books.db")
}

/// 在线书章节正文：library/texts/{book_id}/{chapter_id}.txt
pub fn texts_dir() -> PathBuf {
    library_root().join("texts")
}

/// 本地导入原件：library/imports/
pub fn imports_dir() -> PathBuf {
    library_root().join("imports")
}
