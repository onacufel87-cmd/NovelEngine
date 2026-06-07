//! 单 URL 一键书源接入：页面类型判断、目录/正文 URL 发现、搜索接口探测
//!
//! ## 局限性
//! - 依赖静态 HTML，无法处理纯 JS 渲染站点
//! - 首页探索深度有限（默认 4 层），复杂站点可能需手动提供目录页

use std::sync::OnceLock;

use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use super::auto_detector::{detect_with_log as auto_detect_selectors, detected_to_book_source, is_likely_chapter_title};
use super::detect_log::DetectLogger;
use super::fetcher::fetch_html;
use super::rule::BookSource;
use crate::utils::{AppError, AppResult};

/// 最大从首页向下探索层数
const MAX_EXPLORE_DEPTH: u8 = 4;

/// 搜索表单常见 query 参数名
const SEARCH_PARAM_NAMES: &[&str] = &[
    "q", "searchword", "keyword", "search", "key", "wd", "query", "bookname", "title",
];

/// 小说链接 href 常见关键词
const NOVEL_HREF_KEYWORDS: &[&str] = &[
    "book", "read", "novel", "chapter", "info", "detail", "txt", "html", "article", "fiction",
];

/// 目录页反向链接文本关键词
const TOC_LINK_TEXTS: &[&str] = &["返回目录", "书目录", "全部章节", "章节目录", "目录", "TOC"];

/// 页面类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    /// 正文页：大段文字 + 章节标记
    ContentPage,
    /// 目录页：大量章节链接
    TocPage,
    /// 首页 / 分类 / 书籍详情等需继续探索
    HomePage,
    Unknown,
}

/// 一键接入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConnectResult {
    pub source: BookSource,
    pub confidence: u8,
    pub toc_url: String,
    pub content_url: String,
}

/// 对外入口：用户仅提供一个 URL，自动完成书源接入
pub fn detect_from_single_url(url: &str) -> AppResult<AutoConnectResult> {
    detect_from_single_url_with_log(url, &mut DetectLogger::new())
}

/// 带日志的一键接入
pub fn detect_from_single_url_with_log(
    url: &str,
    log: &mut DetectLogger,
) -> AppResult<AutoConnectResult> {
    let url = url.trim();
    if url.is_empty() {
        log.error("URL 不能为空");
        return Err(AppError::Parse("URL 不能为空".into()));
    }
    log.info(format!("One-click connect started: {url}"));
    explore_and_build(url, 0, None, None, log)
}

/// 使用 WebView 渲染后再分析（首屏 HTML 来自浏览器内核）
pub fn detect_from_single_url_rendered(
    app: &tauri::AppHandle,
    url: &str,
) -> AppResult<AutoConnectResult> {
    detect_from_single_url_rendered_with_log(app, url, &mut DetectLogger::new())
}

/// 带日志的 WebView 渲染接入
pub fn detect_from_single_url_rendered_with_log(
    app: &tauri::AppHandle,
    url: &str,
    log: &mut DetectLogger,
) -> AppResult<AutoConnectResult> {
    use super::rendered_fetch::fetch_html_rendered;
    let url = url.trim();
    if url.is_empty() {
        log.error("URL 不能为空");
        return Err(AppError::Parse("URL 不能为空".into()));
    }
    log.info(format!("One-click connect (rendered): {url}"));
    let render_start = std::time::Instant::now();
    let html = fetch_html_rendered(app, url).map_err(|e| {
        log.error(format!("WebView render failed: {e}"));
        e
    })?;
    log.success(format!(
        "WebView render → OK ({}ms, {} bytes)",
        render_start.elapsed().as_millis(),
        html.len()
    ));
    explore_and_build(url, 0, None, Some(html), log)
}

/// 判断页面类型（启发式）
pub fn classify_page(html: &str) -> PageType {
    let text = body_text(html);
    let text_len = text.chars().count();

    // 正文页：文本较长且含章节标记（标题或首段）
    let has_chapter_mark = chapter_mark_regex().is_match(&text);
    if text_len > 2000 && has_chapter_mark {
        return PageType::ContentPage;
    }

    let (total_links, chapter_links) = count_chapter_links(html);

    // 目录页：章节链接占比高
    if chapter_links >= 3 {
        let ratio = if total_links == 0 {
            1.0
        } else {
            chapter_links as f64 / total_links as f64
        };
        if chapter_links >= 5 || (total_links > 5 && ratio > 0.25) {
            return PageType::TocPage;
        }
    }

    // 中等长度正文（无大量章节链接）
    if text_len > 800 && has_chapter_mark && chapter_links <= 2 {
        return PageType::ContentPage;
    }

    if total_links > 0 {
        PageType::HomePage
    } else {
        PageType::Unknown
    }
}

/// 从正文页 HTML 中查找「返回目录」类链接
pub fn find_toc_link_in_html(base_url: &str, html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let link_sel = Selector::parse("a[href]").ok()?;

    for link in document.select(&link_sel) {
        let text = normalize_text(&link.text().collect::<String>());
        let text_lower = text.to_lowercase();
        if TOC_LINK_TEXTS
            .iter()
            .any(|pat| text.contains(pat) || text_lower.contains(&pat.to_lowercase()))
        {
            if let Some(href) = link.value().attr("href") {
                if let Some(full) = join_url(base_url, href) {
                    return Some(full);
                }
            }
        }
    }
    None
}

/// 根据 URL 路径猜测可能的目录页地址
pub fn guess_toc_url_candidates(content_url: &str) -> Vec<String> {
    let Ok(parsed) = url::Url::parse(content_url) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    let path = parsed.path();

    // 去掉最后一级文件名
    if path.ends_with(".html") || path.ends_with(".htm") || path.ends_with(".php") {
        if let Some(parent) = path.rsplit_once('/') {
            let base_path = parent.0;
            let origin = parsed.origin().ascii_serialization();
            for suffix in ["", "/index.html", "/index.php", "/catalog.html", "/目录.html"] {
                candidates.push(format!("{origin}{base_path}{suffix}"));
            }
        }
    }

    // 向上两级（常见 /book/123/1.html -> /book/123/）
    if let Ok(parent) = parsed.join("..") {
        let p = parent.to_string();
        if p != content_url && !candidates.contains(&p) {
            candidates.push(p);
        }
    }

    candidates
}

/// 从正文页解析目录 URL（链接 + 路径猜测 + 抓取验证）
pub fn resolve_toc_from_content(content_url: &str, content_html: &str) -> AppResult<String> {
    if let Some(url) = find_toc_link_in_html(content_url, content_html) {
        return Ok(url);
    }

    for candidate in guess_toc_url_candidates(content_url) {
        if let Ok(html) = fetch_html(&candidate) {
            if classify_page(&html) == PageType::TocPage {
                return Ok(candidate);
            }
        }
    }

    Err(AppError::Parse(
        "无法从正文页找到目录页，请直接提供该书的目录页 URL".into(),
    ))
}

/// 从目录页 HTML 提取第一个章节链接
pub fn find_first_chapter_from_toc(toc_url: &str, toc_html: &str) -> AppResult<String> {
    let document = Html::parse_document(toc_html);
    let link_sel = Selector::parse("a[href]")
        .map_err(|e| AppError::Parse(format!("内部选择器错误: {e}")))?;

    for link in document.select(&link_sel) {
        let text = normalize_text(&link.text().collect::<String>());
        if !is_likely_chapter_title(&text) {
            continue;
        }
        let href = link.value().attr("href").unwrap_or("").trim();
        if href.is_empty() || href == "#" {
            continue;
        }
        if let Some(full) = join_url(toc_url, href) {
            return Ok(full);
        }
    }

    Err(AppError::Parse(
        "目录页未找到有效章节链接，请确认 URL 是否为小说目录页".into(),
    ))
}

/// 从首页/分类页找第一本小说详情或目录链接
pub fn find_first_novel_link(base_url: &str, html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let link_sel = Selector::parse("a[href]").ok()?;
    let base_host = url::Url::parse(base_url)
        .ok()
        .and_then(|u| u.host_str().map(String::from));

    let mut fallback: Option<String> = None;

    for link in document.select(&link_sel) {
        let href = link.value().attr("href").unwrap_or("").trim();
        if href.is_empty()
            || href.starts_with('#')
            || href.starts_with("javascript:")
            || href.starts_with("mailto:")
        {
            continue;
        }

        let href_lower = href.to_lowercase();
        if href_lower.contains("login")
            || href_lower.contains("register")
            || href_lower.contains("logout")
            || href_lower.contains("pay")
        {
            continue;
        }

        let text = normalize_text(&link.text().collect::<String>());
        let text_len = text.chars().count();

        if NOVEL_HREF_KEYWORDS
            .iter()
            .any(|kw| href_lower.contains(kw))
        {
            if text_len >= 2 && text_len <= 40 {
                if let Some(full) = join_url(base_url, href) {
                    return Some(full);
                }
            }
        }

        // 记录同域内第一个看起来像详情页的链接
        if fallback.is_none() && text_len >= 2 && text_len <= 30 {
            if let Some(full) = join_url(base_url, href) {
                if full != base_url {
                    if let Some(ref host) = base_host {
                        if full.contains(host.as_str()) {
                            fallback = Some(full);
                        }
                    } else if href.starts_with("http") {
                        fallback = Some(full);
                    }
                }
            }
        }
    }

    fallback
}

/// 从页面 form 标签发现 GET 搜索接口模板（含 `{keyword}` 占位符）
pub fn discover_search_endpoint(base_url: &str, html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let form_sel = Selector::parse("form").ok()?;
    let input_sel = Selector::parse("input").ok()?;

    for form in document.select(&form_sel) {
        let method = form
            .value()
            .attr("method")
            .unwrap_or("get")
            .to_lowercase();
        if method != "get" {
            continue;
        }

        let mut query_param: Option<&str> = None;
        for input in form.select(&input_sel) {
            let input_type = input
                .value()
                .attr("type")
                .unwrap_or("text")
                .to_lowercase();
            if input_type == "hidden" || input_type == "submit" || input_type == "button" {
                continue;
            }
            let name = input.value().attr("name").unwrap_or("").trim();
            if SEARCH_PARAM_NAMES.contains(&name) {
                query_param = Some(name);
                break;
            }
        }

        if let Some(param) = query_param {
            let action = form.value().attr("action").unwrap_or("").trim();
            let action_url = if action.is_empty() {
                base_url.to_string()
            } else if let Some(full) = join_url(base_url, action) {
                full
            } else {
                continue;
            };

            let template = if action_url.contains('?') {
                format!("{action_url}&{param}={{keyword}}")
            } else {
                format!("{action_url}?{param}={{keyword}}")
            };
            return Some(template);
        }
    }

    None
}

/// 提取 hostname 用于自动生成书源名
pub fn extract_hostname(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
        .unwrap_or_else(|| "unknown".into())
}

/// 相对链接转绝对 URL
pub fn join_url(base: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with("javascript:") || href.starts_with('#') {
        return None;
    }
    url::Url::parse(base)
        .ok()?
        .join(href)
        .ok()
        .map(|u| u.to_string())
}

// ── 内部流程 ─────────────────────────────────────────────────

fn explore_and_build(
    url: &str,
    depth: u8,
    search_hint: Option<String>,
    initial_html: Option<String>,
    log: &mut DetectLogger,
) -> AppResult<AutoConnectResult> {
    if depth > MAX_EXPLORE_DEPTH {
        log.error(format!("Explore depth exceeded ({MAX_EXPLORE_DEPTH})"));
        return Err(AppError::Parse(
            "自动探索层数过多，请提供具体小说的目录页或正文页 URL".into(),
        ));
    }

    if depth > 0 {
        log.info(format!("Explore depth {depth}: {url}"));
    }

    let html = match initial_html {
        Some(h) => h,
        None => {
            log.info(format!("GET {url}"));
            let start = std::time::Instant::now();
            let fetched = fetch_html(url).map_err(|e| {
                log.error(format!("Fetch failed: {e}"));
                e
            })?;
            log.success(format!(
                "→ 200 ({}ms, {} bytes)",
                start.elapsed().as_millis(),
                fetched.len()
            ));
            fetched
        }
    };

    let page_type = classify_page(&html);
    let page_label = match page_type {
        PageType::ContentPage => "ContentPage",
        PageType::TocPage => "TocPage",
        PageType::HomePage => "HomePage",
        PageType::Unknown => "Unknown",
    };
    log.info(format!("Page classified as {page_label}"));

    let mut search_template = search_hint.or_else(|| {
        discover_search_endpoint(url, &html).map(|tpl| {
            log.info(format!("Discovered search endpoint: {tpl}"));
            tpl
        })
    });

    match page_type {
        PageType::ContentPage => {
            log.info("Resolving catalog URL from content page…");
            let toc_url = resolve_toc_from_content_logged(url, &html, log)?;
            log.success(format!("Catalog URL: {toc_url}"));
            build_connect_result(&toc_url, url, &mut search_template, log)
        }
        PageType::TocPage => {
            log.info("Extracting first chapter from catalog…");
            let content_url = find_first_chapter_from_toc(url, &html).map_err(|e| {
                log.error(format!("{e}"));
                e
            })?;
            log.success(format!("First chapter: {content_url}"));
            build_connect_result(url, &content_url, &mut search_template, log)
        }
        PageType::HomePage | PageType::Unknown => {
            let next_url = find_first_novel_link(url, &html).ok_or_else(|| {
                log.error("No novel link found on page");
                AppError::Parse(
                    "未在该页面找到小说链接。请粘贴具体小说的目录页或正文页 URL，或使用下方手动检测。"
                        .into(),
                )
            })?;
            log.info(format!("Following novel link: {next_url}"));
            explore_and_build(&next_url, depth + 1, search_template, None, log)
        }
    }
}

/// 从正文页解析目录 URL（带日志）
fn resolve_toc_from_content_logged(
    content_url: &str,
    content_html: &str,
    log: &mut DetectLogger,
) -> AppResult<String> {
    if let Some(url) = find_toc_link_in_html(content_url, content_html) {
        log.info(format!("Found TOC link in page: {url}"));
        return Ok(url);
    }
    log.warn("No TOC link in content page, trying path candidates…");
    for candidate in guess_toc_url_candidates(content_url) {
        log.info(format!("Trying catalog candidate: {candidate}"));
        if let Ok(html) = fetch_html(&candidate) {
            if classify_page(&html) == PageType::TocPage {
                log.success(format!("Candidate verified as catalog: {candidate}"));
                return Ok(candidate);
            }
        }
    }
    Err(AppError::Parse(
        "无法从正文页找到目录页，请直接提供该书的目录页 URL".into(),
    ))
}

fn build_connect_result(
    toc_url: &str,
    content_url: &str,
    search_template: &mut Option<String>,
    log: &mut DetectLogger,
) -> AppResult<AutoConnectResult> {
    // 若当前页未发现搜索，再尝试首页
    if search_template.is_none() {
        if let Ok(origin) = url::Url::parse(toc_url).map(|u| u.origin().ascii_serialization()) {
            log.info(format!("Probing site home for search form: {origin}"));
            if let Ok(home_html) = fetch_html(&origin) {
                *search_template = discover_search_endpoint(&origin, &home_html);
                if let Some(ref tpl) = search_template {
                    log.info(format!("Search endpoint from home: {tpl}"));
                }
            }
        }
    }

    // 搜索页检测：将 {keyword} 替换为测试词后抓取
    let search_fetch_url = search_template
        .as_ref()
        .map(|t| t.replace("{keyword}", "小说"));

    log.info("Running selector auto-detection…");
    let detected = auto_detect_selectors(
        toc_url,
        content_url,
        search_fetch_url.as_deref(),
        log,
    )?;

    let hostname = extract_hostname(toc_url);
    let name = format!("自动接入-{hostname}");
    let source = detected_to_book_source(
        &name,
        search_template.as_deref(),
        &detected,
    );

    log.success(format!(
        "Source rule generated: \"{}\" confidence={}%",
        source.name, detected.confidence
    ));

    Ok(AutoConnectResult {
        confidence: detected.confidence,
        source,
        toc_url: toc_url.to_string(),
        content_url: content_url.to_string(),
    })
}

fn body_text(html: &str) -> String {
    let document = Html::parse_document(html);
    if let Ok(sel) = Selector::parse("body") {
        if let Some(body) = document.select(&sel).next() {
            return normalize_text(&body.text().collect::<String>());
        }
    }
    normalize_text(html)
}

fn count_chapter_links(html: &str) -> (usize, usize) {
    let document = Html::parse_document(html);
    let Ok(link_sel) = Selector::parse("a[href]") else {
        return (0, 0);
    };
    let links: Vec<_> = document.select(&link_sel).collect();
    let total = links.len();
    let chapter = links
        .iter()
        .filter(|a| {
            let text = normalize_text(&a.text().collect::<String>());
            is_likely_chapter_title(&text)
        })
        .count();
    (total, chapter)
}

fn normalize_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn chapter_mark_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"第[0-9零一二三四五六七八九十百千]+[章节回卷]|序章|楔子|尾声|后记")
            .expect("chapter mark regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_TOC: &str = include_str!("fixtures/auto_detect_toc.html");
    const FIXTURE_CONTENT: &str = include_str!("fixtures/auto_detect_content.html");
    const FIXTURE_HOME: &str = include_str!("fixtures/auto_detect_home.html");

    #[test]
    fn classifies_toc_page() {
        assert_eq!(classify_page(FIXTURE_TOC), PageType::TocPage);
    }

    #[test]
    fn classifies_home_page() {
        assert_eq!(classify_page(FIXTURE_HOME), PageType::HomePage);
    }

    #[test]
    fn finds_first_chapter() {
        let url = find_first_chapter_from_toc(
            "http://localhost/book/1/",
            FIXTURE_TOC,
        )
        .expect("chapter");
        assert!(url.contains("1.html"));
    }

    #[test]
    fn discovers_search_form() {
        let html = r#"
        <html><body>
          <form action="/search" method="get">
            <input type="text" name="keyword" />
            <button type="submit">搜索</button>
          </form>
        </body></html>
        "#;
        let tpl = discover_search_endpoint("https://example.com/", html).expect("search");
        assert!(tpl.contains("{keyword}"));
        assert!(tpl.contains("keyword"));
    }

    #[test]
    fn finds_novel_link_on_home() {
        let url = find_first_novel_link("https://example.com/", FIXTURE_HOME).expect("link");
        assert!(url.contains("book"));
    }

    #[test]
    fn offline_pipeline_toc_to_source() {
        use super::super::auto_detector::detect_from_html;

        let detected = detect_from_html(FIXTURE_TOC, FIXTURE_CONTENT, None).expect("detect");
        let source = detected_to_book_source("测试源", None, &detected);
        assert!(!source.chapter_list_selector.is_empty());
        assert!(!source.content_selector.is_empty());
    }
}
