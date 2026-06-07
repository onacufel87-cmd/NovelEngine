//! 公版书源 Provider 适配层：将各站点专用逻辑接入统一注册表

use crate::spider::gutenberg;
use crate::spider::providers::BookProvider;
use crate::spider::rule::Chapter;
use crate::utils::AppResult;

pub struct GutenbergProvider;

impl BookProvider for GutenbergProvider {
    fn id(&self) -> &'static str {
        "gutenberg-en"
    }

    fn matches_catalog(&self, url: &str) -> bool {
        gutenberg::is_gutenberg_catalog(url)
    }

    fn matches_chapter(&self, url: &str) -> bool {
        gutenberg::is_gutenberg_chapter_url(url)
    }

    fn fetch_chapters(&self, url: &str) -> AppResult<Vec<Chapter>> {
        gutenberg::fetch_chapters(url)
    }

    fn fetch_chapter_content(&self, url: &str) -> AppResult<String> {
        gutenberg::fetch_chapter_content(url)
    }
}
