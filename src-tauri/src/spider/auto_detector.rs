//! 书源 CSS 选择器自动检测（零配置接入）
//!
//! ## 局限性
//! - 无法处理完全由 JavaScript 客户端渲染的站点（需 headless 浏览器）
//! - 无法处理 class 名随机化或频繁变更的反爬页面
//! - 不适用于图片/漫画站

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use super::fetcher::fetch_html;
use crate::utils::AppResult;
use super::detect_log::DetectLogger;

/// 列表容器标签（章节/搜索结果向上聚合时使用）
const LIST_CONTAINER_TAGS: &[&str] = &["ul", "ol", "div", "table", "tbody", "section"];

/// 正文候选块级标签
const CONTENT_BLOCK_TAGS: &[&str] = &["div", "article", "section", "main", "td"];

/// 目录页章节链接回退选择器
const TOC_FALLBACK_SELECTORS: &str = "#list a, .chapter-list a, .chapters a, .catalog a, #catalog a";

/// 正文容器回退选择器
const CONTENT_FALLBACK_SELECTORS: &str = "#content, .content, .chapter-content, .read-content, #nr1, .showtxt";

/// 自动检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSelectors {
    /// 搜索结果条目容器选择器
    pub search_result_item: String,
    /// 搜索结果内书名选择器
    pub result_title: String,
    /// 搜索结果链接属性
    pub result_url_attr: String,
    /// 目录页章节链接选择器
    pub chapter_list_item: String,
    /// 正文页容器选择器
    pub content_container: String,
    /// 置信度 0-100
    pub confidence: u8,
}

/// 对外入口：拉取页面并推断选择器
pub fn detect(
    toc_url: &str,
    content_url: &str,
    search_url: Option<&str>,
) -> AppResult<DetectedSelectors> {
    detect_with_log(toc_url, content_url, search_url, &mut DetectLogger::new())
}

/// 带结构化日志的选择器检测
pub fn detect_with_log(
    toc_url: &str,
    content_url: &str,
    search_url: Option<&str>,
    log: &mut DetectLogger,
) -> AppResult<DetectedSelectors> {
    log.info(format!("Fetching catalog: {toc_url}"));
    let toc_start = std::time::Instant::now();
    let toc_html = fetch_html(toc_url).map_err(|e| {
        log.error(format!("Catalog fetch failed: {e}"));
        e
    })?;
    log.success(format!(
        "Catalog → OK ({}ms, {} bytes)",
        toc_start.elapsed().as_millis(),
        toc_html.len()
    ));

    log.info(format!("Fetching content: {content_url}"));
    let content_start = std::time::Instant::now();
    let content_html = fetch_html(content_url).map_err(|e| {
        log.error(format!("Content fetch failed: {e}"));
        e
    })?;
    log.success(format!(
        "Content → OK ({}ms, {} bytes)",
        content_start.elapsed().as_millis(),
        content_html.len()
    ));

    let search_html = if let Some(url) = search_url.filter(|u| !u.trim().is_empty()) {
        log.info(format!("Fetching search: {url}"));
        let search_start = std::time::Instant::now();
        match fetch_html(url) {
            Ok(html) => {
                log.success(format!(
                    "Search → OK ({}ms, {} bytes)",
                    search_start.elapsed().as_millis(),
                    html.len()
                ));
                Some(html)
            }
            Err(e) => {
                log.warn(format!("Search fetch failed (skipped): {e}"));
                None
            }
        }
    } else {
        log.info("No search URL provided, skipping search detection");
        None
    };

    detect_from_html_with_log(&toc_html, &content_html, search_html.as_deref(), log)
}

/// 基于已下载 HTML 推断（便于单元测试）
pub fn detect_from_html(
    toc_html: &str,
    content_html: &str,
    search_html: Option<&str>,
) -> AppResult<DetectedSelectors> {
    detect_from_html_with_log(toc_html, content_html, search_html, &mut DetectLogger::new())
}

/// 带日志的 HTML 推断
pub fn detect_from_html_with_log(
    toc_html: &str,
    content_html: &str,
    search_html: Option<&str>,
    log: &mut DetectLogger,
) -> AppResult<DetectedSelectors> {
    let (chapter_list_item, toc_score) = detect_chapter_list_selector(toc_html);
    if chapter_list_item.contains("#list") || chapter_list_item.contains(".chapter-list") {
        log.warn(format!(
            "Chapter list fallback selector: {chapter_list_item}"
        ));
    } else {
        log.info(format!("chapter_list_selector=\"{chapter_list_item}\" (score {toc_score})"));
    }

    let (content_container, content_score) = detect_content_selector(content_html);
    if content_container.contains("#content") || content_container.contains(".content") {
        log.warn(format!(
            "Content container fallback selector: {content_container}"
        ));
    } else {
        log.info(format!("content_selector=\"{content_container}\" (score {content_score})"));
    }

    let (search_result_item, search_score) = if let Some(html) = search_html {
        let (sel, score) = detect_search_selectors(html);
        if sel.is_empty() {
            log.warn("Search result selector not found");
        } else {
            log.info(format!("search_result_selector=\"{sel}\" (score {score})"));
        }
        (sel, score)
    } else {
        (String::new(), 0)
    };

    let confidence = (toc_score + content_score + search_score).min(100);
    if confidence >= 80 {
        log.success(format!("confidence={confidence}% — selectors look reliable"));
    } else if confidence >= 60 {
        log.warn(format!("confidence={confidence}% — manual review recommended"));
    } else {
        log.warn(format!("confidence={confidence}% — low confidence, check selectors"));
    }

    Ok(DetectedSelectors {
        search_result_item,
        result_title: "a".into(),
        result_url_attr: "href".into(),
        chapter_list_item,
        content_container,
        confidence,
    })
}

/// 将检测结果转为可导入的书源规则 JSON 对象（前端可直接 loadRule）
pub fn detected_to_book_source_json(
    name: &str,
    search_url: Option<&str>,
    detected: &DetectedSelectors,
) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "search_url": search_url.unwrap_or(""),
        "search_result_selector": detected.search_result_item,
        "search_title_selector": detected.result_title,
        "search_link_selector": "a",
        "search_link_attr": detected.result_url_attr,
        "chapter_list_selector": detected.chapter_list_item,
        "chapter_title_selector": "text",
        "content_selector": detected.content_container,
        "encoding": "utf-8",
        "ad_keywords": [],
        "clean_patterns": []
    })
}

/// 将检测结果转为 BookSource 结构体（一键接入 / 导入书源库）
pub fn detected_to_book_source(
    name: &str,
    search_url: Option<&str>,
    detected: &DetectedSelectors,
) -> super::rule::BookSource {
    use super::rule::BookSource;

    let search_result = if detected.search_result_item.trim().is_empty() {
        None
    } else {
        Some(detected.search_result_item.clone())
    };

    BookSource {
        name: name.to_string(),
        search_url: search_url.unwrap_or("").to_string(),
        search_result_selector: search_result,
        search_title_selector: Some(detected.result_title.clone()),
        search_author_selector: None,
        search_link_selector: Some("a".to_string()),
        search_link_attr: Some(detected.result_url_attr.clone()),
        book_list_selector: None,
        book_title_selector: None,
        rank_urls: None,
        chapter_list_selector: detected.chapter_list_item.clone(),
        chapter_title_selector: "text".to_string(),
        content_selector: detected.content_container.clone(),
        next_page_selector: None,
        encoding: Some("utf-8".to_string()),
        ad_keywords: Some(vec![]),
        clean_patterns: None,
        cookies: None,
        request_interval_ms: None,
    }
}

// ── 目录页章节列表 ─────────────────────────────────────────────

fn detect_chapter_list_selector(html: &str) -> (String, u8) {
    let document = Html::parse_document(html);
    let link_sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return (TOC_FALLBACK_SELECTORS.to_string(), 20),
    };

    let mut container_counts: HashMap<String, usize> = HashMap::new();
    let mut total_chapter_links = 0usize;

    for link in document.select(&link_sel) {
        let text = normalize_text(&link.text().collect::<String>());
        if !is_likely_chapter_title(&text) {
            continue;
        }
        total_chapter_links += 1;
        if let Some(container) = find_nearest_list_container(link) {
            if let Some(key) = simple_selector(container) {
                *container_counts.entry(key).or_default() += 1;
            }
        }
    }

    if total_chapter_links == 0 {
        return (TOC_FALLBACK_SELECTORS.to_string(), 25);
    }

    if let Some((container_sel, count)) = container_counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
    {
        let ratio = count as f64 / total_chapter_links as f64;
        if count >= 3 && ratio > 0.3 {
            let selector = format!("{container_sel} a");
            let score = if ratio > 0.8 { 45 } else { 35 };
            return (selector, score);
        }
    }

    (TOC_FALLBACK_SELECTORS.to_string(), 28)
}

// ── 正文容器 ─────────────────────────────────────────────────

fn detect_content_selector(html: &str) -> (String, u8) {
    let cleaned = strip_non_content_tags(html);
    let document = Html::parse_document(&cleaned);

    let mut best: Option<(String, usize, f64)> = None;

    for tag in CONTENT_BLOCK_TAGS {
        let sel_str = tag.to_string();
        let Ok(sel) = Selector::parse(&sel_str) else {
            continue;
        };
        for elem in document.select(&sel) {
            let text_len = visible_text_len(elem);
            if text_len <= 200 {
                continue;
            }
            let html_len = outer_html_len(elem);
            if html_len == 0 {
                continue;
            }
            let density = text_len as f64 / html_len as f64;
            let content_like = is_content_like_element(elem);
            if density <= 0.3 && !content_like {
                continue;
            }
            if density <= 0.12 && content_like {
                continue;
            }
            let selector = simple_selector(elem).unwrap_or_else(|| tag.to_string());
            let score_boost = if content_like { 5usize } else { 0 };
            let replace = match &best {
                None => true,
                Some((_, len, _)) => text_len + score_boost > *len,
            };
            if replace {
                best = Some((selector, text_len + score_boost, density));
            }
        }
    }

    if let Some((sel, len, density)) = best {
        let score = if len > 800 && density > 0.5 {
            45
        } else if len > 400 {
            38
        } else {
            32
        };
        (sel, score)
    } else {
        (CONTENT_FALLBACK_SELECTORS.to_string(), 25)
    }
}

// ── 搜索结果列表 ─────────────────────────────────────────────

fn detect_search_selectors(html: &str) -> (String, u8) {
    let document = Html::parse_document(html);
    let link_sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return (String::new(), 0),
    };

    let mut container_counts: HashMap<String, usize> = HashMap::new();

    for link in document.select(&link_sel) {
        let text = normalize_text(&link.text().collect::<String>());
        let len = text.chars().count();
        if len < 4 || len > 50 {
            continue;
        }
        if is_pagination_text(&text) {
            continue;
        }
        if let Some(container) = find_nearest_list_container(link) {
            if let Some(key) = simple_selector(container) {
                *container_counts.entry(key).or_default() += 1;
            }
        }
    }

    if let Some((container_sel, count)) = container_counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
    {
        if count >= 2 {
            return (format!("{container_sel} a"), 20);
        }
    }

    // 回退：常见搜索结果容器
    (".search-item, .result-item, .book-item, ul li".to_string(), 12)
}

// ── 辅助函数 ─────────────────────────────────────────────────

/// 判断文本是否像章节标题
pub fn is_likely_chapter_title(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || t.chars().count() > 80 {
        return false;
    }
    chapter_title_regex().is_match(t)
}

fn chapter_title_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?x)
            (?:第[0-9零一二三四五六七八九十百千]+[章节回卷])
            |(?:序章|楔子|尾声|后记|引子|番外)
            ",
        )
        .expect("chapter title regex")
    })
}

fn is_pagination_text(text: &str) -> bool {
    let t = text.trim();
    t.contains("下一页")
        || t.contains("上一页")
        || t.contains("next")
        || t.contains("prev")
        || t == "»"
        || t == "«"
}

/// 向上查找最近的列表型容器
pub fn find_nearest_list_container(elem: ElementRef<'_>) -> Option<ElementRef<'_>> {
    let mut current = Some(elem);
    while let Some(node) = current {
        let tag = element_tag_name(node);
        if LIST_CONTAINER_TAGS.contains(&tag) {
            return Some(node);
        }
        current = node.parent().and_then(ElementRef::wrap);
    }
    None
}

/// 生成简化 CSS 选择器（优先 id，其次首个 class，否则标签名）
pub fn simple_selector(elem: ElementRef<'_>) -> Option<String> {
    if let Some(id) = element_attr(elem, "id") {
        let id = id.trim();
        if !id.is_empty() && !id.contains(' ') {
            return Some(format!("#{}", escape_css_ident(id)));
        }
    }
    if let Some(class) = element_attr(elem, "class") {
        if let Some(first) = class.split_whitespace().find(|c| !c.is_empty()) {
            return Some(format!(".{}", escape_css_ident(first)));
        }
    }
    Some(element_tag_name(elem).to_string())
}

fn element_tag_name(elem: ElementRef<'_>) -> &str {
    elem.value().name()
}

fn element_attr(elem: ElementRef<'_>, name: &str) -> Option<String> {
    elem.attr(name).map(|s| s.to_string())
}

fn escape_css_ident(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_string()
            } else {
                format!("\\{c}")
            }
        })
        .collect()
}

fn normalize_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_content_like_element(elem: ElementRef<'_>) -> bool {
    let id = element_attr(elem, "id").unwrap_or_default().to_lowercase();
    let class = element_attr(elem, "class").unwrap_or_default().to_lowercase();
    id.contains("content")
        || id.contains("chapter")
        || id.contains("text")
        || class.contains("content")
        || class.contains("chapter")
        || class.contains("read")
        || class.contains("showtxt")
}

fn visible_text_len(elem: ElementRef<'_>) -> usize {
    normalize_text(&elem.text().collect::<String>()).chars().count()
}

fn outer_html_len(elem: ElementRef<'_>) -> usize {
    elem.html().len()
}

/// 移除 script/style 及页眉页脚导航，降低正文检测干扰
fn strip_non_content_tags(html: &str) -> String {
    let patterns = [
        r"(?is)<script[^>]*>.*?</script>",
        r"(?is)<style[^>]*>.*?</style>",
        r"(?is)<noscript[^>]*>.*?</noscript>",
        r"(?is)<header[^>]*>.*?</header>",
        r"(?is)<footer[^>]*>.*?</footer>",
        r"(?is)<nav[^>]*>.*?</nav>",
    ];
    let mut out = html.to_string();
    for p in patterns {
        if let Ok(re) = Regex::new(p) {
            out = re.replace_all(&out, "").to_string();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_TOC: &str = include_str!("fixtures/auto_detect_toc.html");
    const FIXTURE_CONTENT: &str = include_str!("fixtures/auto_detect_content.html");
    const FIXTURE_SEARCH: &str = include_str!("fixtures/auto_detect_search.html");

    #[test]
    fn detects_jieqi_style_toc() {
        let (sel, score) = detect_chapter_list_selector(FIXTURE_TOC);
        assert!(sel.contains("#list") || sel.contains("chapter-list"), "got {sel}");
        assert!(score >= 30);
        assert!(is_likely_chapter_title("第一章 初入江湖"));
        assert!(!is_likely_chapter_title("网站首页"));
    }

    #[test]
    fn detects_content_div() {
        let (sel, score) = detect_content_selector(FIXTURE_CONTENT);
        assert!(
            sel.contains("#content") || sel.contains("chapter-content"),
            "got {sel}"
        );
        assert!(score >= 30);
    }

    #[test]
    fn detects_search_ul_list() {
        let (sel, score) = detect_search_selectors(FIXTURE_SEARCH);
        assert!(sel.contains("search-results") || sel.contains(" a"), "got {sel}");
        assert!(score >= 12);
    }

    #[test]
    fn detect_from_html_integration() {
        let result = detect_from_html(FIXTURE_TOC, FIXTURE_CONTENT, Some(FIXTURE_SEARCH))
            .expect("detect");
        assert!(!result.chapter_list_item.is_empty());
        assert!(!result.content_container.is_empty());
        assert!(!result.search_result_item.is_empty());
        assert!(result.confidence >= 50);
        assert_eq!(result.result_title, "a");
        assert_eq!(result.result_url_attr, "href");
    }

    #[test]
    fn simple_selector_prefers_id() {
        let html = r#"<html><body><ul id="list"><li><a href="/1">第一章</a></li></ul></body></html>"#;
        let doc = Html::parse_document(html);
        let ul = doc.select(&Selector::parse("ul").unwrap()).next().unwrap();
        assert_eq!(simple_selector(ul).as_deref(), Some("#list"));
    }
}
