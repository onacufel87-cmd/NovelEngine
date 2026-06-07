//! 全网搜索与榜单

use serde::Deserialize;

use crate::crawler::{BookListItem, SearchResult};
use crate::services::{discovery, to_cmd_err};
use crate::storage::Book;

/// 从搜索加入书架请求体
#[derive(Debug, Deserialize)]
pub struct AddBookFromSearchPayload {
    pub title: String,
    pub author: Option<String>,
    pub catalog_url: String,
    pub source_id: String,
}

#[tauri::command]
pub fn search_books(keyword: String, origin: String) -> Result<Vec<SearchResult>, String> {
    discovery::search_books(&keyword, &origin).map_err(to_cmd_err)
}

#[tauri::command]
pub fn add_book_from_search(payload: AddBookFromSearchPayload) -> Result<Book, String> {
    discovery::add_book_from_search(
        &payload.title,
        payload.author.as_deref(),
        &payload.catalog_url,
        &payload.source_id,
    )
    .map_err(to_cmd_err)
}

#[tauri::command]
pub fn get_rank_types(source_id: String) -> Result<Vec<String>, String> {
    discovery::get_rank_types(&source_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn fetch_rankings_cmd(
    source_id: String,
    rank_type: String,
    origin: String,
) -> Result<Vec<BookListItem>, String> {
    discovery::fetch_rankings(&source_id, &rank_type, &origin).map_err(to_cmd_err)
}
