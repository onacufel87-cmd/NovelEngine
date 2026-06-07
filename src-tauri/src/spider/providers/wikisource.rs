//! 中文维基文库 Provider

use crate::spider::wikisource;
use crate::spider::parser::SearchResultItem;
use crate::spider::providers::BookProvider;
use crate::spider::rule::Chapter;
use crate::utils::AppResult;

pub struct WikisourceProvider;

impl BookProvider for WikisourceProvider {
    fn id(&self) -> &'static str {
        "wikisource-zh"
    }

    fn matches_catalog(&self, url: &str) -> bool {
        wikisource::is_wikisource_url(url)
    }

    fn matches_chapter(&self, url: &str) -> bool {
        wikisource::is_wikisource_url(url)
    }

    fn fetch_chapters(&self, url: &str) -> AppResult<Vec<Chapter>> {
        wikisource::fetch_chapters(url)
    }

    fn fetch_chapter_content(&self, url: &str) -> AppResult<String> {
        wikisource::fetch_chapter_content(url)
    }

    fn try_search(&self, keyword: &str) -> Option<AppResult<Vec<SearchResultItem>>> {
        Some(wikisource::search(keyword))
    }
}
