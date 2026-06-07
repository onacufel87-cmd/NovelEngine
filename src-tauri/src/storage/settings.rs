use rusqlite::params;

use super::db::get_connection;
use crate::utils::{AppError, AppResult};

/// 读取单项设置
pub fn get_setting(key: &str) -> AppResult<Option<String>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let result = stmt
        .query_row(params![key], |row| row.get(0))
        .ok();

    Ok(result)
}

/// 写入或更新设置
pub fn set_setting(key: &str, value: &str) -> AppResult<()> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

/// 读取所有设置
pub fn get_all_settings() -> AppResult<Vec<(String, String)>> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| AppError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(settings)
}
