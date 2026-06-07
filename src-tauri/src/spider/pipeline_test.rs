//! 离线集成测试：本地 HTML fixture → 解析目录/正文/搜索（不依赖网络）

#[cfg(test)]
mod tests {
    use crate::crawler::searcher::resolve_url_template;
    use crate::spider::cleaner::apply_global_clean;
    use crate::spider::parser::{parse_and_clean_content, parse_chapters, parse_search_results};
    use crate::spider::rule::parse_book_source;

    const RULE_JSON: &str = include_str!("fixtures/test_rule.json");
    const CATALOG_HTML: &str = include_str!("fixtures/test_catalog.html");
    const CHAPTER1_HTML: &str = include_str!("fixtures/test_chapter.html");
    const CHAPTER2_HTML: &str = include_str!("fixtures/test_chapter_page2.html");
    const SEARCH_HTML: &str = include_str!("fixtures/test_search.html");

    const BASE: &str = "http://localhost:1420";

    #[test]
    fn pipeline_parse_catalog() {
        let source = parse_book_source(RULE_JSON).expect("rule");
        let chapters = parse_chapters(CATALOG_HTML, &source, BASE).expect("chapters");
        assert!(chapters.len() >= 3, "expected >=3 chapters, got {}", chapters.len());
        assert!(chapters[0].title.contains("第一章"));
        assert!(chapters[0].url.contains("test-chapter"));
    }

    #[test]
    fn pipeline_parse_chapter_content() {
        let source = parse_book_source(RULE_JSON).expect("rule");
        let content = parse_and_clean_content(CHAPTER1_HTML, &source).expect("content");
        assert!(content.contains("少年站在山巅"));
        // 书源 ad_keywords 应移除广告句
        assert!(!content.contains("请记住本站网址"));
    }

    #[test]
    fn pipeline_chapter_page2() {
        let source = parse_book_source(RULE_JSON).expect("rule");
        let content = parse_and_clean_content(CHAPTER2_HTML, &source).expect("content");
        assert!(content.contains("第二页的正文"));
    }

    #[test]
    fn pipeline_search_results() {
        let source = parse_book_source(RULE_JSON).expect("rule");
        let search_url = resolve_url_template(&source.search_url, BASE, Some("测试"));
        let results = parse_search_results(SEARCH_HTML, &source, &search_url).expect("search");
        assert!(!results.is_empty());
        assert!(results[0].title.len() > 0);
    }

    #[test]
    fn pipeline_global_clean_applies() {
        let source = parse_book_source(RULE_JSON).expect("rule");
        let raw = parse_and_clean_content(CHAPTER1_HTML, &source).expect("content");
        let cleaned = apply_global_clean(&raw);
        assert!(!cleaned.is_empty());
    }
}
