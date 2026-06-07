pub mod pinyin_restore;
pub mod detect_log;
pub mod pipeline_test;
pub mod providers;
pub mod auto_detector;
pub mod cleaner;
pub mod fetcher;
pub mod gutenberg;
pub mod openlibrary;
pub mod parser;
pub mod rule;
pub mod rule_schema;
pub mod rendered_fetch;
pub mod standardebooks;
pub mod text_conv;
pub mod url_analyzer;
pub mod wikisource;

use rule::parse_book_source;

pub use auto_detector::{
    detect as auto_detect_selectors, detect_with_log as auto_detect_selectors_with_log,
    detected_to_book_source, detected_to_book_source_json, DetectedSelectors,
};
pub use detect_log::{DetectLogEntry, DetectLogger, DetectResponse};
pub use url_analyzer::{
    detect_from_single_url, detect_from_single_url_rendered,
    detect_from_single_url_with_log, detect_from_single_url_rendered_with_log,
    AutoConnectResult, PageType,
};
pub use rendered_fetch::{fetch_html_rendered, store_captured_html};
pub use text_conv::{apply_chinese_variant, to_simplified, to_traditional};
pub use cleaner::{clean_content, clean_content_for_source};
pub use rule::{BookSource, Chapter};
pub use fetcher::{
    apply_reader_settings_json, fetch_for_source, fetch_html, fetch_html_with_encoding,
    fetch_with_options, set_global_fetch_config, FetchOptions, GlobalFetchConfig,
};
pub use parser::{
    parse_book_list, parse_chapters, parse_content, parse_and_clean_content,
    parse_search_results, extract_next_page_url,
};

/// 一站式：下载目录页并解析章节列表
pub fn fetch_and_parse_chapters(url: &str, rule_json: &str) -> crate::utils::AppResult<Vec<Chapter>> {
    if let Some(result) = providers::dispatch_chapters(url) {
        return result;
    }

    let source = parse_book_source(rule_json)?;
    let html = fetch_for_source(url, &source)?;
    parse_chapters(&html, &source, url)
}

/// 一站式：下载章节页、解析并清洗正文（支持正文分页）
pub fn fetch_and_parse_content(url: &str, rule_json: &str) -> crate::utils::AppResult<String> {
    if let Some(result) = providers::dispatch_chapter_content(url) {
        return result;
    }

    let source = parse_book_source(rule_json)?;
    fetch_chapter_by_source(url, &source)
}

/// 阶段 1：仅用 CSS 选择器解析单个章节 URL
pub fn fetch_chapter_by_selector(
    url: &str,
    content_selector: &str,
    encoding: Option<&str>,
    next_page_selector: Option<&str>,
    ad_keywords: Option<Vec<String>>,
) -> crate::utils::AppResult<String> {
    let source = BookSource {
        name: "临时解析".into(),
        search_url: String::new(),
        search_result_selector: None,
        search_title_selector: None,
        search_author_selector: None,
        search_link_selector: None,
        search_link_attr: None,
        book_list_selector: None,
        book_title_selector: None,
        rank_urls: None,
        chapter_list_selector: String::new(),
        chapter_title_selector: String::new(),
        content_selector: content_selector.to_string(),
        next_page_selector: next_page_selector.map(String::from),
        encoding: encoding.map(String::from),
        ad_keywords,
        clean_patterns: None,
        cookies: None,
        request_interval_ms: None,
    };
    fetch_chapter_by_source(url, &source)
}

/// 按书源规则抓取完整章节（自动跟随「下一页」）
fn fetch_chapter_by_source(start_url: &str, source: &BookSource) -> crate::utils::AppResult<String> {
    let mut url = start_url.to_string();
    let mut parts: Vec<String> = Vec::new();
    let max_pages = 50;

    for _ in 0..max_pages {
        let html = fetch_for_source(&url, source)?;
        let chunk = parse_and_clean_content(&html, source)?;
        if !chunk.is_empty() {
            parts.push(chunk);
        }

        // 有下一页选择器则继续抓取
        if let Some(ref next_sel) = source.next_page_selector {
            match extract_next_page_url(&html, next_sel, &url)? {
                Some(next_url) if next_url != url => {
                    url = next_url;
                    continue;
                }
                _ => break,
            }
        } else {
            break;
        }
    }

    if parts.is_empty() {
        return Err(crate::utils::AppError::Parse(
            "未能从页面中提取到正文，请检查 URL 与 CSS 选择器".into(),
        ));
    }

    let raw = parts.join("\n\n");
    // 多页合并后再做一次空白压缩（各页已单独清洗）
    Ok(clean_content_for_source(&raw, Some(source)))
}
