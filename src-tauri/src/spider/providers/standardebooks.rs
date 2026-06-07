//! Standard Ebooks Provider

use crate::spider::standardebooks;
use crate::spider::parser::SearchResultItem;
use crate::spider::providers::BookProvider;
use crate::spider::rule::Chapter;
use crate::utils::AppResult;

pub struct StandardEbooksProvider;

impl BookProvider for StandardEbooksProvider {
    fn id(&self) -> &'static str {
        "standardebooks-en"
    }

    fn matches_catalog(&self, url: &str) -> bool {
        standardebooks::is_standardebooks_catalog(url)
    }

    fn matches_chapter(&self, url: &str) -> bool {
        standardebooks::is_standardebooks_chapter_url(url)
            || (url.contains("standardebooks.org") && url.contains("/text/single-page"))
    }

    fn fetch_chapters(&self, url: &str) -> AppResult<Vec<Chapter>> {
        standardebooks::fetch_chapters(url)
    }

    fn fetch_chapter_content(&self, url: &str) -> AppResult<String> {
        standardebooks::fetch_chapter_content(url)
    }

    fn try_search(&self, keyword: &str) -> Option<AppResult<Vec<SearchResultItem>>> {
        Some(standardebooks::search(keyword))
    }
}
