//! 书架、阅读、导出、本地导入

use serde::Deserialize;

use crate::services::{read, to_cmd_err};
use crate::spider::Chapter;
use crate::storage::{Book, BookDetail, ExportBookResult};

/// 加入书架请求体
#[derive(Debug, Deserialize)]
pub struct AddBookPayload {
    pub title: String,
    pub catalog_url: String,
    pub rule_json: String,
    pub chapters: Vec<Chapter>,
}

#[tauri::command]
pub fn get_shelf_books() -> Result<Vec<Book>, String> {
    read::list_shelf_books().map_err(to_cmd_err)
}

#[tauri::command]
pub fn add_book_to_shelf(payload: AddBookPayload) -> Result<Book, String> {
    let pairs = read::chapters_to_pairs(payload.chapters);
    read::add_book_to_shelf(
        &payload.title,
        &payload.catalog_url,
        &payload.rule_json,
        &pairs,
    )
    .map_err(to_cmd_err)
}

#[tauri::command]
pub fn remove_book_from_shelf(book_id: i64) -> Result<(), String> {
    read::remove_book_from_shelf(book_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn get_book_detail_cmd(book_id: i64) -> Result<BookDetail, String> {
    read::get_book_detail(book_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn read_chapter_content(book_id: i64, chapter_id: i64) -> Result<String, String> {
    read::read_chapter_content(book_id, chapter_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn save_read_progress(book_id: i64, chapter_id: i64, offset: i64) -> Result<(), String> {
    read::save_read_progress(book_id, chapter_id, offset).map_err(to_cmd_err)
}

#[tauri::command]
pub async fn export_book(book_id: i64) -> Result<ExportBookResult, String> {
    tauri::async_runtime::spawn_blocking(move || read::export_book(book_id))
        .await
        .map_err(|e| format!("导出任务异常: {e}"))?
        .map_err(to_cmd_err)
}

#[tauri::command]
pub fn import_local_book(file_path: String) -> Result<Book, String> {
    read::import_local_book(&file_path).map_err(to_cmd_err)
}
