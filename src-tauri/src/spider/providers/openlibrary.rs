//! Open Library / Internet Archive Provider

use crate::spider::openlibrary;
use crate::spider::parser::SearchResultItem;
use crate::spider::providers::BookProvider;
use crate::spider::rule::Chapter;
use crate::utils::AppResult;

pub struct OpenLibraryProvider;

impl BookProvider for OpenLibraryProvider {
    fn id(&self) -> &'static str {
        "openlibrary"
    }

    fn matches_catalog(&self, url: &str) -> bool {
        openlibrary::is_openlibrary_catalog(url)
    }

    fn matches_chapter(&self, url: &str) -> bool {
        // 正文抓取沿用目录 URL 判定（与旧逻辑一致）
        openlibrary::is_openlibrary_catalog(url)
    }

    fn fetch_chapters(&self, url: &str) -> AppResult<Vec<Chapter>> {
        openlibrary::fetch_chapters(url)
    }

    fn fetch_chapter_content(&self, url: &str) -> AppResult<String> {
        openlibrary::fetch_chapter_content(url)
    }

    fn try_search(&self, keyword: &str) -> Option<AppResult<Vec<SearchResultItem>>> {
        Some(openlibrary::search(keyword))
    }
}
