use rusqlite::params;

use super::chapters::{get_chapter, load_chapter_content};
use super::db::get_connection;
use crate::utils::{AppError, AppResult};

/// 阅读标注记录
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteRecord {
    pub id: i64,
    pub book_id: i64,
    pub chapter_id: i64,
    pub note_type: String,
    pub start_offset: Option<i64>,
    pub end_offset: Option<i64>,
    pub quote: Option<String>,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
    pub body: String,
    pub color: String,
    pub content_hash: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 笔记列表项（含书名、章节名，供评论页展示）
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteListItem {
    pub note: NoteRecord,
    pub book_title: String,
    pub chapter_title: String,
}

/// 按书分组的笔记
#[derive(Debug, Clone, serde::Serialize)]
pub struct NotesByBook {
    pub book_id: i64,
    pub book_title: String,
    pub notes: Vec<NoteListItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateNotePayload {
    pub book_id: i64,
    pub chapter_id: i64,
    pub start_offset: i64,
    pub end_offset: i64,
    pub quote: String,
    pub context_before: String,
    pub context_after: String,
    pub content_hash: String,
    pub body: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateNotePayload {
    pub id: i64,
    pub body: String,
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteRecord> {
    Ok(NoteRecord {
        id: row.get(0)?,
        book_id: row.get(1)?,
        chapter_id: row.get(2)?,
        note_type: row.get(3)?,
        start_offset: row.get(4)?,
        end_offset: row.get(5)?,
        quote: row.get(6)?,
        context_before: row.get(7)?,
        context_after: row.get(8)?,
        body: row.get(9)?,
        color: row.get(10)?,
        content_hash: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

const NOTE_SELECT: &str = "SELECT id, book_id, chapter_id, note_type, start_offset, end_offset,
    quote, context_before, context_after, body, color, content_hash, created_at, updated_at";

/// 按划词锚点查找已有评论（幂等创建用）
fn find_note_by_anchor(
    conn: &rusqlite::Connection,
    book_id: i64,
    chapter_id: i64,
    start_offset: i64,
    end_offset: i64,
) -> AppResult<Option<NoteRecord>> {
    match conn.query_row(
        &format!(
            "{NOTE_SELECT} FROM notes
             WHERE book_id = ?1 AND chapter_id = ?2
               AND start_offset = ?3 AND end_offset = ?4"
        ),
        params![book_id, chapter_id, start_offset, end_offset],
        map_note_row,
    ) {
        Ok(note) => Ok(Some(note)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}

/// 根据章节正文校正划词偏移，避免前后端索引不一致导致漏字
fn reconcile_anchor_offsets(
    chapter_text: &str,
    start: i64,
    end: i64,
    quote: &str,
    context_before: &str,
    context_after: &str,
) -> (i64, i64, String) {
    if quote.is_empty() {
        return (start, end, quote.to_string());
    }

    let start_usize = start.max(0) as usize;
    let end_usize = end.max(start + 1) as usize;

    // 索引切片与 quote 一致则无需校正
    if end_usize <= chapter_text.len() && start_usize < end_usize {
        if chapter_text.get(start_usize..end_usize) == Some(quote) {
            return (start, end, quote.to_string());
        }
    }

    let search_radius = 150usize;
    let from = start_usize.saturating_sub(search_radius);
    let to = (end_usize + search_radius).min(chapter_text.len());
    if from < to {
        if let Some(local) = chapter_text[from..to].find(quote) {
            let idx = from + local;
            return (idx as i64, (idx + quote.len()) as i64, quote.to_string());
        }
    }

    // 上下文锚点匹配
    if !context_before.is_empty() || !context_after.is_empty() {
        let needle = format!("{context_before}{quote}{context_after}");
        if let Some(pos) = chapter_text.find(&needle) {
            let s = pos + context_before.len();
            return (s as i64, (s + quote.len()) as i64, quote.to_string());
        }
    }

    // 全文搜索 quote
    if let Some(idx) = chapter_text.find(quote) {
        return (idx as i64, (idx + quote.len()) as i64, quote.to_string());
    }

    (start, end, quote.to_string())
}

/// 创建划词高亮笔记（可先高亮、后补正文；同锚点幂等）
pub fn create_note(payload: &CreateNotePayload) -> AppResult<NoteRecord> {
    if payload.start_offset >= payload.end_offset {
        return Err(AppError::Database("标注范围无效".into()));
    }

    let conn = get_connection()?;

    // 加载章节正文，校正偏移后再入库
    let (start_offset, end_offset, quote) = if let Ok(chapter) = get_chapter(payload.chapter_id) {
        if let Some(text) = load_chapter_content(&chapter) {
            let (s, e, q) = reconcile_anchor_offsets(
                &text,
                payload.start_offset,
                payload.end_offset,
                &payload.quote,
                &payload.context_before,
                &payload.context_after,
            );
            (s, e, q)
        } else {
            (
                payload.start_offset,
                payload.end_offset,
                payload.quote.clone(),
            )
        }
    } else {
        (
            payload.start_offset,
            payload.end_offset,
            payload.quote.clone(),
        )
    };

    // 已存在相同划词范围则直接返回
    if let Some(existing) = find_note_by_anchor(
        &conn,
        payload.book_id,
        payload.chapter_id,
        start_offset,
        end_offset,
    )? {
        return Ok(existing);
    }

    let now = unix_now();
    let body = payload.body.clone().unwrap_or_default();
    let color = payload
        .color
        .clone()
        .unwrap_or_else(|| "#D4A373".to_string());

    match conn.execute(
        "INSERT INTO notes (
            book_id, chapter_id, note_type, start_offset, end_offset,
            quote, context_before, context_after, body, color, content_hash,
            created_at, updated_at
        ) VALUES (?1, ?2, 'highlight', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            payload.book_id,
            payload.chapter_id,
            start_offset,
            end_offset,
            quote,
            payload.context_before,
            payload.context_after,
            body,
            color,
            payload.content_hash,
            now,
            now,
        ],
    ) {
        Ok(_) => get_note_by_id(conn.last_insert_rowid()),
        Err(e) if e.to_string().contains("UNIQUE") => find_note_by_anchor(
            &conn,
            payload.book_id,
            payload.chapter_id,
            start_offset,
            end_offset,
        )?
        .ok_or_else(|| AppError::Database("重复划词记录写入失败".into())),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}

/// 更新笔记正文
pub fn update_note(payload: &UpdateNotePayload) -> AppResult<NoteRecord> {
    let conn = get_connection()?;
    let now = unix_now();
    let rows = conn
        .execute(
            "UPDATE notes SET body = ?1, updated_at = ?2 WHERE id = ?3",
            params![payload.body, now, payload.id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    if rows == 0 {
        return Err(AppError::Database(format!("笔记 id={} 不存在", payload.id)));
    }

    get_note_by_id(payload.id)
}

/// 删除笔记
pub fn delete_note(note_id: i64) -> AppResult<()> {
    let conn = get_connection()?;
    let rows = conn
        .execute("DELETE FROM notes WHERE id = ?1", params![note_id])
        .map_err(|e| AppError::Database(e.to_string()))?;
    if rows == 0 {
        return Err(AppError::Database(format!("笔记 id={note_id} 不存在")));
    }
    Ok(())
}

pub fn get_note_by_id(note_id: i64) -> AppResult<NoteRecord> {
    let conn = get_connection()?;
    conn.query_row(
        &format!("{NOTE_SELECT} FROM notes WHERE id = ?1"),
        params![note_id],
        map_note_row,
    )
    .map_err(|e| AppError::Database(format!("笔记 id={note_id} 不存在: {e}")))
}

/// 某章全部标注（阅读页渲染高亮）
pub fn list_notes_by_chapter(book_id: i64, chapter_id: i64) -> AppResult<Vec<NoteRecord>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(&format!(
            "{NOTE_SELECT} FROM notes
             WHERE book_id = ?1 AND chapter_id = ?2
             ORDER BY start_offset ASC, id ASC"
        ))
        .map_err(|e| AppError::Database(e.to_string()))?;

    let notes = stmt
        .query_map(params![book_id, chapter_id], map_note_row)
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// 全部笔记（含书名章节名）
pub fn list_all_notes_with_meta() -> AppResult<Vec<NoteListItem>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.book_id, n.chapter_id, n.note_type, n.start_offset, n.end_offset,
                    n.quote, n.context_before, n.context_after, n.body, n.color, n.content_hash,
                    n.created_at, n.updated_at,
                    b.title AS book_title, c.title AS chapter_title
             FROM notes n
             JOIN books b ON b.id = n.book_id
             JOIN chapters c ON c.id = n.chapter_id
             ORDER BY n.updated_at DESC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let items = stmt
        .query_map([], |row| {
            Ok(NoteListItem {
                note: NoteRecord {
                    id: row.get(0)?,
                    book_id: row.get(1)?,
                    chapter_id: row.get(2)?,
                    note_type: row.get(3)?,
                    start_offset: row.get(4)?,
                    end_offset: row.get(5)?,
                    quote: row.get(6)?,
                    context_before: row.get(7)?,
                    context_after: row.get(8)?,
                    body: row.get(9)?,
                    color: row.get(10)?,
                    content_hash: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                },
                book_title: row.get(14)?,
                chapter_title: row.get(15)?,
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// 按书分组笔记
pub fn list_notes_grouped_by_book() -> AppResult<Vec<NotesByBook>> {
    let items = list_all_notes_with_meta()?;
    let mut groups: Vec<NotesByBook> = Vec::new();

    for item in items {
        if let Some(group) = groups.iter_mut().find(|g| g.book_id == item.note.book_id) {
            group.notes.push(item);
        } else {
            groups.push(NotesByBook {
                book_id: item.note.book_id,
                book_title: item.book_title.clone(),
                notes: vec![item],
            });
        }
    }

    Ok(groups)
}
