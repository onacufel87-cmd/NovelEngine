//! 章节正文文件缓存：SQLite 仅存索引，正文落盘避免数据库膨胀

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::utils::{AppError, AppResult};

static CACHE_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// 应用启动时设置正文缓存根目录（通常为 app_data/content_cache）
pub fn set_content_cache_dir(path: PathBuf) {
    let _ = CACHE_ROOT.set(path);
}

fn cache_root() -> PathBuf {
    CACHE_ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("content_cache"))
}

fn chapter_file_path(book_id: i64, chapter_id: i64) -> PathBuf {
    cache_root()
        .join(book_id.to_string())
        .join(format!("{chapter_id}.txt"))
}

/// 写入章节正文到文件系统
pub fn write_chapter_content(book_id: i64, chapter_id: i64, content: &str) -> AppResult<()> {
    let path = chapter_file_path(book_id, chapter_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::Database(format!("创建正文缓存目录失败: {e}")))?;
    }
    fs::write(&path, content)
        .map_err(|e| AppError::Database(format!("写入正文缓存失败: {e}")))?;
    Ok(())
}

/// 从文件系统读取章节正文
pub fn read_chapter_content(book_id: i64, chapter_id: i64) -> Option<String> {
    let path = chapter_file_path(book_id, chapter_id);
    fs::read_to_string(path).ok()
}

/// 删除单本书的全部正文缓存
pub fn delete_book_cache(book_id: i64) -> AppResult<()> {
    let dir = cache_root().join(book_id.to_string());
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| AppError::Database(format!("删除正文缓存失败: {e}")))?;
    }
    Ok(())
}

/// 将旧版 SQLite 内嵌正文迁移到文件（启动时可选调用）
pub fn migrate_inline_content_to_disk(
    book_id: i64,
    chapter_id: i64,
    inline: &str,
) -> AppResult<()> {
    if inline.is_empty() {
        return Ok(());
    }
    if read_chapter_content(book_id, chapter_id).is_some() {
        return Ok(());
    }
    write_chapter_content(book_id, chapter_id, inline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn write_and_read_roundtrip() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("nr-cache-test-{stamp}"));
        set_content_cache_dir(tmp.clone());

        write_chapter_content(1, 2, "测试正文").expect("write");
        let text = read_chapter_content(1, 2).expect("read");
        assert_eq!(text, "测试正文");

        delete_book_cache(1).ok();
        let _ = fs::remove_dir_all(tmp);
    }
}
