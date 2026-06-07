use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rusqlite::Connection;

use crate::utils::{AppError, AppResult};

/// 全局数据库文件路径（在 Tauri setup 中初始化）
static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// 设置数据库路径（应用启动时调用一次）
pub fn set_db_path(path: PathBuf) {
    let _ = DB_PATH.set(path);
}

/// 获取数据库路径，未初始化时回退到项目目录（便于单元测试）
fn db_path() -> PathBuf {
    DB_PATH
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("books.db"))
}

/// 在指定路径初始化 SQLite 并建表
pub fn init_db_at(path: &Path) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Database(format!("创建数据目录失败: {e}")))?;
    }

    let conn = Connection::open(path).map_err(|e| AppError::Database(e.to_string()))?;
    run_migrations(&conn)?;
    Ok(())
}

/// 初始化默认路径的数据库
pub fn init_db() -> AppResult<Connection> {
    let path = db_path();
    init_db_at(&path)?;
    get_connection()
}

/// 获取数据库连接
pub fn get_connection() -> AppResult<Connection> {
    let path = db_path();
    let conn = Connection::open(&path).map_err(|e| AppError::Database(e.to_string()))?;
    Ok(conn)
}

/// 建表与增量迁移
fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT,
            cover_url TEXT,
            source_rule_name TEXT,
            source_rule_json TEXT NOT NULL DEFAULT '',
            chapter_list_url TEXT,
            last_chapter_id INTEGER,
            last_read_offset INTEGER DEFAULT 0,
            update_time INTEGER
        );

        CREATE TABLE IF NOT EXISTS chapters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            content TEXT,
            is_read BOOLEAN DEFAULT 0,
            FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS book_sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            rule_json TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            last_verified INTEGER,
            created_at INTEGER,
            updated_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS search_cache (
            keyword TEXT NOT NULL,
            origin TEXT NOT NULL,
            results_json TEXT NOT NULL,
            cached_at INTEGER NOT NULL,
            PRIMARY KEY (keyword, origin)
        );

        CREATE TABLE IF NOT EXISTS rank_list (
            source_id TEXT NOT NULL,
            rank_type TEXT NOT NULL,
            books_json TEXT NOT NULL,
            cached_at INTEGER NOT NULL,
            PRIMARY KEY (source_id, rank_type)
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            chapter_id INTEGER NOT NULL,
            note_type TEXT NOT NULL DEFAULT 'highlight',
            start_offset INTEGER,
            end_offset INTEGER,
            quote TEXT,
            context_before TEXT,
            context_after TEXT,
            body TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '#D4A373',
            content_hash TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_notes_book ON notes(book_id);
        CREATE INDEX IF NOT EXISTS idx_notes_chapter ON notes(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at DESC);
        ",
    )
    .map_err(|e| AppError::Database(e.to_string()))?;

    // 兼容旧库：补 source_rule_json 列
    let _ = conn.execute(
        "ALTER TABLE books ADD COLUMN source_rule_json TEXT NOT NULL DEFAULT ''",
        [],
    );

    // 兼容旧库：记录书籍来自哪个书源
    let _ = conn.execute("ALTER TABLE books ADD COLUMN source_id TEXT", []);

    // 正文落盘标记（1 = 正文在 content_cache 目录）
    let _ = conn.execute(
        "ALTER TABLE chapters ADD COLUMN content_on_disk INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // 清理重复划词：同书同章同偏移只保留最早一条
    let _ = conn.execute_batch(
        "
        DELETE FROM notes
        WHERE id NOT IN (
            SELECT MIN(id)
            FROM notes
            WHERE start_offset IS NOT NULL AND end_offset IS NOT NULL
            GROUP BY book_id, chapter_id, start_offset, end_offset
        )
        AND start_offset IS NOT NULL AND end_offset IS NOT NULL;
        ",
    );

    // 联合唯一索引：防止相同划词范围重复入库
    let _ = conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_notes_anchor
         ON notes(book_id, chapter_id, start_offset, end_offset)",
        [],
    );

    // 书源库第二期：来源、健康度、订阅关联
    let _ = conn.execute(
        "ALTER TABLE book_sources ADD COLUMN origin TEXT NOT NULL DEFAULT 'custom'",
        [],
    );
    let _ = conn.execute("ALTER TABLE book_sources ADD COLUMN subscription_url TEXT", []);
    let _ = conn.execute("ALTER TABLE book_sources ADD COLUMN tags TEXT", []);
    let _ = conn.execute("ALTER TABLE book_sources ADD COLUMN ping_ms INTEGER", []);
    let _ = conn.execute(
        "ALTER TABLE book_sources ADD COLUMN health_status TEXT NOT NULL DEFAULT 'unknown'",
        [],
    );

    // 内置书源标记 origin
    let _ = conn.execute(
        "UPDATE book_sources SET origin = 'builtin' WHERE is_builtin = 1",
        [],
    );

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS source_subscriptions (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            label TEXT,
            last_synced_at INTEGER,
            created_at INTEGER
        );
        ",
    )
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(())
}
