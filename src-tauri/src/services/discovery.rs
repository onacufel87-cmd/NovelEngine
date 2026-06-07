//! 全网搜索、榜单、从搜索加书架

use crate::crawler::{
    fetch_rankings as crawl_rankings, list_rank_types, search_books as crawl_search,
    BookListItem, SearchResult,
};
use crate::storage::{add_from_search, Book};
use crate::utils::AppResult;

/// 聚合搜索
pub fn search_books(keyword: &str, origin: &str) -> AppResult<Vec<SearchResult>> {
    crawl_search(keyword, origin)
}

/// 从搜索结果加入书架
pub fn add_book_from_search(
    title: &str,
    author: Option<&str>,
    catalog_url: &str,
    source_id: &str,
) -> AppResult<Book> {
    add_from_search(title, author, catalog_url, source_id)
}

/// 榜单类型列表
pub fn get_rank_types(source_id: &str) -> AppResult<Vec<String>> {
    list_rank_types(source_id)
}

/// 抓取榜单
pub fn fetch_rankings(
    source_id: &str,
    rank_type: &str,
    origin: &str,
) -> AppResult<Vec<BookListItem>> {
    crawl_rankings(source_id, rank_type, origin)
}
