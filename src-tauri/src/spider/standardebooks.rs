use regex::Regex;
use scraper::{Html, Selector};

use super::fetcher::fetch_html;
use super::parser::SearchResultItem;
use super::rule::Chapter;
use crate::utils::{AppError, AppResult};

const SE_ORIGIN: &str = "https://standardebooks.org";

/// 是否为 Standard Ebooks 书籍页
pub fn is_standardebooks_catalog(url: &str) -> bool {
    url.contains("standardebooks.org/ebooks/")
        && !url.contains("/text/")
        && !url.contains("/downloads/")
}

/// 是否为 SE 章节 URL（#se-ch-N 锚点）
pub fn is_standardebooks_chapter_url(url: &str) -> bool {
    url.contains("standardebooks.org") && url.contains("#se-ch-")
}

/// 搜索 Standard Ebooks 公版书库
pub fn search(keyword: &str) -> AppResult<Vec<SearchResultItem>> {
    let encoded: String = url::form_urlencoded::byte_serialize(keyword.trim().as_bytes()).collect();
    let search_url = format!("{SE_ORIGIN}/ebooks/search?q={encoded}");
    let html = fetch_html(&search_url)?;

    let document = Html::parse_document(&html);
    let article_sel = Selector::parse("article.ebook, li.ebook").map_err(|e| AppError::Parse(e.to_string()))?;
    let title_sel = Selector::parse("h2 a, h3 a").unwrap();
    let author_sel = Selector::parse("p.author, span.author").unwrap();

    let mut results = Vec::new();
    for article in document.select(&article_sel) {
        let link = article.select(&title_sel).next();
        let Some(link) = link else { continue };
        let title = normalize_ws(&link.text().collect::<String>());
        let href = link.value().attr("href").unwrap_or("").trim();
        if title.is_empty() || href.is_empty() {
            continue;
        }
        let author = article
            .select(&author_sel)
            .next()
            .map(|el| normalize_ws(&el.text().collect::<String>()))
            .filter(|s| !s.is_empty());
        let catalog_url = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("{SE_ORIGIN}{href}")
        };
        results.push(SearchResultItem {
            title,
            author,
            catalog_url,
        });
        if results.len() >= 25 {
            break;
        }
    }

    Ok(results)
}

/// 拉取目录：读取 single-page 文本并按 h2 分章
pub fn fetch_chapters(catalog_url: &str) -> AppResult<Vec<Chapter>> {
    let read_url = single_page_url(catalog_url);
    let html = fetch_html(&read_url)?;
    build_chapters(&html, &read_url)
}

/// 读取 SE 单章正文
pub fn fetch_chapter_content(chapter_url: &str) -> AppResult<String> {
    let (read_url, index) = parse_chapter_url(chapter_url)?;
    let html = fetch_html(&read_url)?;
    extract_chapter_text(&html, index)
}

fn single_page_url(catalog_url: &str) -> String {
    let base = catalog_url.trim_end_matches('/');
    if base.ends_with("/text/single-page") {
        return base.to_string();
    }
    format!("{base}/text/single-page")
}

fn build_chapters(html: &str, read_url: &str) -> AppResult<Vec<Chapter>> {
    let document = Html::parse_document(html);
    let h2_sel = Selector::parse("h2").map_err(|e| AppError::Parse(e.to_string()))?;
    let mut headings = Vec::new();

    for h2 in document.select(&h2_sel) {
        let title = normalize_ws(&h2.text().collect::<String>());
        if title.is_empty() || is_skipped_heading(&title) {
            continue;
        }
        if is_chapter_heading(&title) {
            headings.push(title);
        }
    }

    if headings.is_empty() {
        return Ok(vec![Chapter {
            title: "全文".into(),
            url: format!("{read_url}#se-ch-0"),
        }]);
    }

    Ok(headings
        .into_iter()
        .enumerate()
        .map(|(i, title)| Chapter {
            title,
            url: format!("{read_url}#se-ch-{i}"),
        })
        .collect())
}

fn extract_chapter_text(html: &str, chapter_index: usize) -> AppResult<String> {
    let document = Html::parse_document(html);
    let h2_sel = Selector::parse("h2").unwrap();
    let mut chapter_h2_indices: Vec<usize> = Vec::new();

    for (idx, h2) in document.select(&h2_sel).enumerate() {
        let title = normalize_ws(&h2.text().collect::<String>());
        if !title.is_empty() && is_chapter_heading(&title) && !is_skipped_heading(&title) {
            chapter_h2_indices.push(idx);
        }
    }

    if chapter_h2_indices.is_empty() {
        if chapter_index == 0 {
            return Ok(html_to_plain(html));
        }
        return Err(AppError::Parse("章节索引超出范围".into()));
    }

    if chapter_index >= chapter_h2_indices.len() {
        return Err(AppError::Parse(format!(
            "章节索引 {chapter_index} 超出范围（共 {} 章）",
            chapter_h2_indices.len()
        )));
    }

    let offsets = h2_byte_offsets(html);
    let h2_idx = chapter_h2_indices[chapter_index];
    let start = offsets
        .get(h2_idx)
        .copied()
        .ok_or_else(|| AppError::Parse("章节定位失败".into()))?;
    let end = if chapter_index + 1 < chapter_h2_indices.len() {
        let next_h2_idx = chapter_h2_indices[chapter_index + 1];
        offsets.get(next_h2_idx).copied().unwrap_or(html.len())
    } else {
        html.len()
    };

    Ok(html_to_plain(&html[start..end.min(html.len())]))
}

fn parse_chapter_url(url: &str) -> AppResult<(String, usize)> {
    let re = Regex::new(r"#se-ch-(\d+)").unwrap();
    let index = re
        .captures(url)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| AppError::Parse(format!("无效的 SE 章节 URL: {url}")))?;
    let read_url = url.split('#').next().unwrap_or(url).to_string();
    Ok((read_url, index))
}

fn h2_byte_offsets(html: &str) -> Vec<usize> {
    Regex::new(r"(?is)<h2\b")
        .unwrap()
        .find_iter(html)
        .map(|m| m.start())
        .collect()
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
        || upper.starts_with("ACT ")
        || upper.starts_with("SCENE ")
        || upper.starts_with("PROLOGUE")
        || upper.starts_with("EPILOGUE")
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
