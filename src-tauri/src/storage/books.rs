use rusqlite::params;

use super::db::get_connection;
use crate::utils::{AppError, AppResult};

/// 书架中的书籍记录
#[derive(Debug, Clone, serde::Serialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub source_rule_name: Option<String>,
    pub chapter_list_url: Option<String>,
    pub last_chapter_id: Option<i64>,
    pub last_read_offset: Option<i64>,
    pub chapter_count: i64,
}

/// 添加书籍并保存章节列表，返回完整书记录
pub fn add_book_with_chapters(
    title: &str,
    author: Option<&str>,
    chapter_list_url: &str,
    source_rule_name: &str,
    source_rule_json: &str,
    source_id: Option<&str>,
    chapters: &[(String, String)],
) -> AppResult<Book> {
    if chapters.is_empty() {
        return Err(AppError::Database("章节列表不能为空".into()));
    }

    let conn = get_connection()?;
    let now = unix_now();

    conn.execute(
        "INSERT INTO books (title, author, chapter_list_url, source_rule_name, source_rule_json, source_id, update_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            title,
            author,
            chapter_list_url,
            source_rule_name,
            source_rule_json,
            source_id,
            now
        ],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;

    let book_id = conn.last_insert_rowid();
    super::chapters::save_chapters(book_id, chapters)?;

    get_book_by_id(book_id)
}

/// 按 ID 获取书籍
pub fn get_book_by_id(id: i64) -> AppResult<Book> {
    let conn = get_connection()?;
    conn.query_row(
        "SELECT b.id, b.title, b.author, b.cover_url, b.source_rule_name,
                b.chapter_list_url, b.last_chapter_id, b.last_read_offset,
                (SELECT COUNT(*) FROM chapters c WHERE c.book_id = b.id) AS chapter_count
         FROM books b WHERE b.id = ?1",
        params![id],
        map_book_row,
    )
    .map_err(|e| AppError::Database(format!("书籍 id={id} 不存在: {e}")))
}

/// 获取书源 JSON（阅读时后端内部使用）
pub fn get_source_rule_json(book_id: i64) -> AppResult<String> {
    let conn = get_connection()?;
    conn.query_row(
        "SELECT source_rule_json FROM books WHERE id = ?1",
        params![book_id],
        |row| row.get(0),
    )
    .map_err(|e| AppError::Database(format!("读取书源规则失败: {e}")))
}

/// 获取所有书架书籍
pub fn list_books() -> AppResult<Vec<Book>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.title, b.author, b.cover_url, b.source_rule_name,
                    b.chapter_list_url, b.last_chapter_id, b.last_read_offset,
                    (SELECT COUNT(*) FROM chapters c WHERE c.book_id = b.id) AS chapter_count
             FROM books b ORDER BY b.update_time DESC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let books = stmt
        .query_map([], map_book_row)
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(books)
}

/// 删除书籍及其章节
pub fn delete_book(id: i64) -> AppResult<()> {
    super::content_cache::delete_book_cache(id).ok();
    let conn = get_connection()?;
    conn.execute("DELETE FROM chapters WHERE book_id = ?1", params![id])
        .map_err(|e| AppError::Database(e.to_string()))?;
    conn.execute("DELETE FROM books WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

/// 更新阅读进度
pub fn update_book_progress(book_id: i64, chapter_id: i64, offset: i64) -> AppResult<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE books SET last_chapter_id = ?1, last_read_offset = ?2, update_time = ?3
         WHERE id = ?4",
        params![chapter_id, offset, unix_now(), book_id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn map_book_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        title: row.get(1)?,
        author: row.get(2)?,
        cover_url: row.get(3)?,
        source_rule_name: row.get(4)?,
        chapter_list_url: row.get(5)?,
        last_chapter_id: row.get(6)?,
        last_read_offset: row.get(7)?,
        chapter_count: row.get(8)?,
    })
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
