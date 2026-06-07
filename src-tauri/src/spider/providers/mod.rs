//! 内置公版书源 Provider 注册表：统一 URL / 书源 ID 分发

mod gutenberg;
mod openlibrary;
mod standardebooks;
mod wikisource;

use std::sync::OnceLock;

use gutenberg::GutenbergProvider;
use openlibrary::OpenLibraryProvider;
use standardebooks::StandardEbooksProvider;
use wikisource::WikisourceProvider;

use super::parser::SearchResultItem;
use super::rule::Chapter;
use crate::utils::AppResult;

/// 公版/专用书源能力接口；未匹配时由通用 CSS 引擎兜底
pub trait BookProvider: Send + Sync {
    /// 与 builtin_sources.json 中的 id 一致
    fn id(&self) -> &'static str;

    /// 是否处理该目录页 URL
    fn matches_catalog(&self, url: &str) -> bool;

    /// 是否处理该正文页 URL
    fn matches_chapter(&self, url: &str) -> bool;

    fn fetch_chapters(&self, url: &str) -> AppResult<Vec<Chapter>>;
    fn fetch_chapter_content(&self, url: &str) -> AppResult<String>;

    /// 专用搜索；默认 None 表示走书源 JSON 的 CSS 搜索
    fn try_search(&self, _keyword: &str) -> Option<AppResult<Vec<SearchResultItem>>> {
        None
    }
}

/// 注册表：按固定顺序匹配，新增公版源只需注册一个 Provider
pub struct ProviderRegistry {
    providers: [&'static dyn BookProvider; 4],
}

impl ProviderRegistry {
    /// 内置公版源（顺序与旧 if-else 链一致）
    pub fn builtin() -> Self {
        static GUTENBERG: GutenbergProvider = GutenbergProvider;
        static WIKISOURCE: WikisourceProvider = WikisourceProvider;
        static OPENLIBRARY: OpenLibraryProvider = OpenLibraryProvider;
        static STANDARD_EBOOKS: StandardEbooksProvider = StandardEbooksProvider;

        Self {
            providers: [
                &GUTENBERG,
                &WIKISOURCE,
                &OPENLIBRARY,
                &STANDARD_EBOOKS,
            ],
        }
    }

    fn by_id(&self, source_id: &str) -> Option<&'static dyn BookProvider> {
        self.providers
            .iter()
            .find(|p| p.id() == source_id)
            .copied()
    }

    fn find_for_catalog(&self, url: &str) -> Option<&'static dyn BookProvider> {
        self.providers
            .iter()
            .find(|p| p.matches_catalog(url))
            .copied()
    }

    fn find_for_chapter(&self, url: &str) -> Option<&'static dyn BookProvider> {
        self.providers
            .iter()
            .find(|p| p.matches_chapter(url))
            .copied()
    }
}

static REGISTRY: OnceLock<ProviderRegistry> = OnceLock::new();

fn registry() -> &'static ProviderRegistry {
    REGISTRY.get_or_init(ProviderRegistry::builtin)
}

/// 尝试由专用 Provider 解析目录；None 表示走 CSS 书源规则
pub fn dispatch_chapters(url: &str) -> Option<AppResult<Vec<Chapter>>> {
    registry()
        .find_for_catalog(url)
        .map(|provider| provider.fetch_chapters(url))
}

/// 尝试由专用 Provider 抓取正文；None 表示走 CSS 书源规则
pub fn dispatch_chapter_content(url: &str) -> Option<AppResult<String>> {
    registry()
        .find_for_chapter(url)
        .map(|provider| provider.fetch_chapter_content(url))
}

/// 按书源 ID 尝试专用搜索；None 表示走通用 CSS 搜索
pub fn search_by_source_id(
    source_id: &str,
    keyword: &str,
) -> Option<AppResult<Vec<SearchResultItem>>> {
    registry().by_id(source_id)?.try_search(keyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_gutenberg_catalog() {
        let url = "https://www.gutenberg.org/ebooks/1342";
        assert!(registry().find_for_catalog(url).is_some());
        assert!(registry().find_for_chapter(url).is_none());
    }

    #[test]
    fn dispatches_gutenberg_chapter() {
        let url = "https://www.gutenberg.org/cache/epub/1342/pg1342-images.html#guten-ch-1";
        assert!(registry().find_for_chapter(url).is_some());
    }

    #[test]
    fn dispatches_wikisource() {
        let url = "https://zh.wikisource.org/wiki/红楼梦";
        assert!(registry().find_for_catalog(url).is_some());
        assert!(registry().find_for_chapter(url).is_some());
    }

    #[test]
    fn dedicated_search_by_id() {
        assert!(search_by_source_id("wikisource-zh", "test").is_some());
        assert!(search_by_source_id("gutenberg-en", "test").is_none());
    }
}
