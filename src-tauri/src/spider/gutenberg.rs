use regex::Regex;
use scraper::{Html, Selector};

use super::fetcher::fetch_html;
use super::rule::Chapter;
use crate::utils::{AppError, AppResult};

/// 是否为 Gutenberg 书籍详情页（/ebooks/123）
pub fn is_gutenberg_catalog(url: &str) -> bool {
    url.contains("gutenberg.org/ebooks/")
}

/// 是否为 Gutenberg 章节 URL（带 #guten-ch-N 锚点）
pub fn is_gutenberg_chapter_url(url: &str) -> bool {
    url.contains("gutenberg.org/cache/epub/") && url.contains("#guten-ch-")
}

/// 从 /ebooks/{id} 解析目录：拉取在线 HTML 并按 h2 标题分章
pub fn fetch_chapters(catalog_url: &str) -> AppResult<Vec<Chapter>> {
    let id = extract_ebook_id(catalog_url)?;
    let (read_url, html) = fetch_read_html(&id)?;
    build_chapters(&html, &read_url)
}

/// 读取 Gutenberg 某一章正文（同一 HTML 页按 h2 切分）
pub fn fetch_chapter_content(chapter_url: &str) -> AppResult<String> {
    let (read_url, index) = parse_chapter_url(chapter_url)?;
    let html = fetch_html(&read_url)?;
    extract_chapter_text(&html, index)
}

fn extract_ebook_id(url: &str) -> AppResult<String> {
    let re = Regex::new(r"gutenberg\.org/ebooks/(\d+)").unwrap();
    re.captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AppError::Parse(format!("无法从 URL 解析 Gutenberg 书籍 ID: {url}")))
}

/// 拉取可读 HTML（优先 cache/epub 在线版）
fn fetch_read_html(id: &str) -> AppResult<(String, String)> {
    let candidates = [
        format!("https://www.gutenberg.org/cache/epub/{id}/pg{id}-images.html"),
        format!("https://www.gutenberg.org/cache/epub/{id}/pg{id}.html"),
    ];

    for url in candidates {
        if let Ok(html) = fetch_html(&url) {
            return Ok((url, html));
        }
    }

    Err(AppError::Network(format!(
        "无法打开 Gutenberg 在线阅读页 (ebook #{id})"
    )))
}

fn chapter_heading_indices(document: &Html) -> AppResult<Vec<(String, usize)>> {
    let h2_sel = Selector::parse("h2").map_err(|e| AppError::Parse(e.to_string()))?;
    let mut headings = Vec::new();

    for (idx, h2) in document.select(&h2_sel).enumerate() {
        let title = normalize_ws(&h2.text().collect::<String>());
        if title.is_empty() || is_skipped_heading(&title) {
            continue;
        }
        if is_chapter_heading(&title) {
            headings.push((title, idx));
        }
    }

    Ok(headings)
}

fn is_skipped_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower == "contents" || lower.starts_with("table of contents")
}

fn is_chapter_heading(title: &str) -> bool {
    let upper = title.to_uppercase();
    upper.starts_with("CHAPTER")
        || upper.starts_with("PART ")
        || upper.starts_with("BOOK ")
        || upper.starts_with("LETTER ")
        || upper.starts_with("SECTION ")
        || upper.starts_with("ACT ")
        || upper.starts_with("SCENE ")
        || upper.contains("CHAPTER ")
}

fn build_chapters(html: &str, read_url: &str) -> AppResult<Vec<Chapter>> {
    let document = Html::parse_document(html);
    let headings = chapter_heading_indices(&document)?;

    if headings.is_empty() {
        return Ok(vec![Chapter {
            title: "全文".into(),
            url: format!("{read_url}#guten-ch-0"),
        }]);
    }

    Ok(headings
        .into_iter()
        .enumerate()
        .map(|(i, (title, _))| Chapter {
            title,
            url: format!("{read_url}#guten-ch-{i}"),
        })
        .collect())
}

fn parse_chapter_url(url: &str) -> AppResult<(String, usize)> {
    let re = Regex::new(r"#guten-ch-(\d+)").unwrap();
    let index = re
        .captures(url)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| AppError::Parse(format!("无效的 Gutenberg 章节 URL: {url}")))?;

    let read_url = url.split('#').next().unwrap_or(url).to_string();
    Ok((read_url, index))
}

/// 按 h2 在 HTML 中的字节位置切出章节片段并转纯文本
fn extract_chapter_text(html: &str, chapter_index: usize) -> AppResult<String> {
    let document = Html::parse_document(html);
    let headings = chapter_heading_indices(&document)?;

    if headings.is_empty() {
        if chapter_index == 0 {
            return Ok(html_to_plain(html));
        }
        return Err(AppError::Parse("章节索引超出范围".into()));
    }

    if chapter_index >= headings.len() {
        return Err(AppError::Parse(format!(
            "章节索引 {chapter_index} 超出范围（共 {} 章）",
            headings.len()
        )));
    }

    let offsets = h2_byte_offsets(html);
    let (_, h2_idx) = &headings[chapter_index];
    let start = offsets
        .get(*h2_idx)
        .copied()
        .ok_or_else(|| AppError::Parse("章节定位失败".into()))?;

    let end = if chapter_index + 1 < headings.len() {
        let (_, next_idx) = &headings[chapter_index + 1];
        offsets.get(*next_idx).copied().unwrap_or(html.len())
    } else {
        find_end_of_book(html).unwrap_or(html.len())
    };

    Ok(html_to_plain(&html[start..end.min(html.len())]))
}

fn h2_byte_offsets(html: &str) -> Vec<usize> {
    Regex::new(r"(?is)<h2\b")
        .unwrap()
        .find_iter(html)
        .map(|m| m.start())
        .collect()
}

fn find_end_of_book(html: &str) -> Option<usize> {
    Regex::new(r"(?i)\*\*\* END OF (THE|THIS) PROJECT GUTENBERG")
        .unwrap()
        .find(html)
        .map(|m| m.start())
}

fn html_to_plain(html: &str) -> String {
    let document = Html::parse_fragment(html);
    let p_sel = Selector::parse("p").unwrap();
    let parts: Vec<String> = document
        .select(&p_sel)
        .map(|p| normalize_ws(&p.text().collect::<String>()))
        .filter(|s| !s.is_empty())
        .collect();

    if !parts.is_empty() {
        return parts.join("\n\n");
    }

    normalize_ws(&document.root_element().text().collect::<String>())
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
