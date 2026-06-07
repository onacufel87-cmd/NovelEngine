use rusqlite::params;

use super::content_cache::{read_chapter_content, write_chapter_content};
use super::db::get_connection;
use crate::utils::{AppError, AppResult};

/// 章节记录
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterRecord {
    pub id: i64,
    pub book_id: i64,
    pub title: String,
    pub url: String,
    /// 兼容旧库：仅当未落盘时可能有值
    pub content: Option<String>,
    pub is_read: bool,
}

/// 批量保存章节列表
pub fn save_chapters(book_id: i64, chapters: &[(String, String)]) -> AppResult<()> {
    let conn = get_connection()?;
    for (title, url) in chapters {
        conn.execute(
            "INSERT INTO chapters (book_id, title, url) VALUES (?1, ?2, ?3)",
            params![book_id, title, url],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(())
}

/// 保存章节并预填正文（本地导入：正文写入文件系统）
pub fn save_chapters_with_content(
    book_id: i64,
    chapters: &[(String, String, String)],
) -> AppResult<()> {
    let conn = get_connection()?;
    for (title, url, content) in chapters {
        conn.execute(
            "INSERT INTO chapters (book_id, title, url, content, content_on_disk) VALUES (?1, ?2, ?3, NULL, 0)",
            params![book_id, title, url],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        let chapter_id: i64 = conn.last_insert_rowid();
        write_chapter_content(book_id, chapter_id, content)?;
        conn.execute(
            "UPDATE chapters SET content_on_disk = 1, content = NULL WHERE id = ?1",
            params![chapter_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(())
}

/// 获取某本书的章节列表
pub fn list_chapters(book_id: i64) -> AppResult<Vec<ChapterRecord>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, title, url, content, is_read
             FROM chapters WHERE book_id = ?1 ORDER BY id",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let chapters = stmt
        .query_map(params![book_id], map_chapter_row)
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chapters)
}

/// 按 ID 获取单个章节
pub fn get_chapter(chapter_id: i64) -> AppResult<ChapterRecord> {
    let conn = get_connection()?;
    conn.query_row(
        "SELECT id, book_id, title, url, content, is_read
         FROM chapters WHERE id = ?1",
        params![chapter_id],
        map_chapter_row,
    )
    .map_err(|e| AppError::Database(format!("章节 id={chapter_id} 不存在: {e}")))
}

/// 读取章节正文：优先文件缓存，其次 SQLite 旧字段
pub fn load_chapter_content(chapter: &ChapterRecord) -> Option<String> {
    if let Some(text) = read_chapter_content(chapter.book_id, chapter.id) {
        if !text.is_empty() {
            return Some(text);
        }
    }
    chapter.content.clone().filter(|c| !c.is_empty())
}

/// 缓存章节正文到文件系统（不再写入 SQLite BLOB 字段）
pub fn cache_content(chapter_id: i64, content: &str) -> AppResult<()> {
    let chapter = get_chapter(chapter_id)?;
    write_chapter_content(chapter.book_id, chapter_id, content)?;

    let conn = get_connection()?;
    conn.execute(
        "UPDATE chapters SET content = NULL, content_on_disk = 1 WHERE id = ?1",
        params![chapter_id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn map_chapter_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChapterRecord> {
    Ok(ChapterRecord {
        id: row.get(0)?,
        book_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        content: row.get(4)?,
        is_read: row.get(5)?,
    })
}
