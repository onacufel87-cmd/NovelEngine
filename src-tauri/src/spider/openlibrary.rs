use regex::Regex;
use serde::Deserialize;

use super::fetcher::fetch_html;
use super::parser::SearchResultItem;
use super::rule::Chapter;
use crate::utils::{AppError, AppResult};

const OL_SEARCH: &str = "https://openlibrary.org/search.json";
const IA_DETAILS: &str = "https://archive.org/details";

/// 是否为 Open Library / Internet Archive 目录 URL
pub fn is_openlibrary_catalog(url: &str) -> bool {
    url.contains("openlibrary.org/works/")
        || url.contains("openlibrary.org/books/")
        || url.contains("archive.org/details/")
}

/// 是否为 IA 章节 URL（自定义 #ia-ch-N 锚点）
pub fn is_openlibrary_chapter_url(url: &str) -> bool {
    url.contains("archive.org/details/") && url.contains("#ia-ch-")
}

/// 通过 Open Library JSON API 搜索公版电子书（关联 Internet Archive 全文）
pub fn search(keyword: &str) -> AppResult<Vec<SearchResultItem>> {
    let kw = keyword.trim();
    let encoded: String = url::form_urlencoded::byte_serialize(kw.as_bytes()).collect();
    let mut url = format!("{OL_SEARCH}?q={encoded}&has_fulltext=true&limit=25");
    if contains_cjk(kw) {
        url.push_str("&language=chi");
    }

    let json = fetch_html(&url)?;
    let response: OlSearchResponse = serde_json::from_str(&json)
        .map_err(|e| AppError::Parse(format!("Open Library 搜索 JSON 解析失败: {e}")))?;

    let mut results = Vec::new();
    for doc in response.docs {
        let title = doc.title.unwrap_or_default().trim().to_string();
        if title.is_empty() {
            continue;
        }
        let author = doc
            .author_name
            .as_ref()
            .and_then(|names| names.first())
            .cloned();
        let ia = doc.ia.and_then(|ids| ids.into_iter().next());
        let key = doc.key.unwrap_or_default();
        if key.is_empty() {
            continue;
        }
        // 将 IA 标识写入 fragment，便于后续拉取全文
        let catalog_url = if let Some(ref ia_id) = ia {
            format!("https://openlibrary.org{key}#ia={ia_id}")
        } else {
            format!("https://openlibrary.org{key}")
        };
        results.push(SearchResultItem {
            title,
            author,
            catalog_url,
        });
    }

    Ok(results)
}

/// 解析目录：从 Internet Archive 全文按章节标题切分
pub fn fetch_chapters(catalog_url: &str) -> AppResult<Vec<Chapter>> {
    let ia_id = resolve_ia_identifier(catalog_url)?;
    let details_url = format!("{IA_DETAILS}/{ia_id}");
    let text = fetch_ia_plaintext(&ia_id)?;

    let headings = split_chapter_headings(&text);
    if headings.is_empty() {
        return Ok(vec![Chapter {
            title: "全文".into(),
            url: details_url,
        }]);
    }

    Ok(headings
        .into_iter()
        .enumerate()
        .map(|(i, title)| Chapter {
            title,
            url: format!("{details_url}#ia-ch-{i}"),
        })
        .collect())
}

/// 读取 IA 某一章正文（或无分章时的全文）
pub fn fetch_chapter_content(chapter_url: &str) -> AppResult<String> {
    // 无 #ia-ch-N 锚点时返回全书正文
    if !is_openlibrary_chapter_url(chapter_url) {
        let ia_id = resolve_ia_identifier(chapter_url)?;
        let text = fetch_ia_plaintext(&ia_id)?;
        return Ok(trim_ia_boilerplate(&text));
    }

    let (details_url, index) = parse_chapter_url(chapter_url)?;
    let ia_id = resolve_ia_identifier(&details_url)?;
    let text = fetch_ia_plaintext(&ia_id)?;
    extract_chapter_text(&text, index)
}

#[derive(Debug, Deserialize)]
struct OlSearchResponse {
    docs: Vec<OlDoc>,
}

#[derive(Debug, Deserialize)]
struct OlDoc {
    title: Option<String>,
    author_name: Option<Vec<String>>,
    key: Option<String>,
    ia: Option<Vec<String>>,
}

fn resolve_ia_identifier(catalog_url: &str) -> AppResult<String> {
    // 优先从 #ia=xxx fragment 读取
    if let Some(ia) = extract_fragment_param(catalog_url, "ia") {
        return Ok(ia);
    }
    if catalog_url.contains("archive.org/details/") {
        let re = Regex::new(r"archive\.org/details/([^/?#]+)").unwrap();
        if let Some(caps) = re.captures(catalog_url) {
            return Ok(caps[1].to_string());
        }
    }
    // 从 Open Library work 页面 JSON 查找可读版本
    let work_key = extract_work_key(catalog_url)?;
    let json_url = format!("https://openlibrary.org{work_key}/editions.json?limit=10");
    let json = fetch_html(&json_url)?;
    let editions: OlEditionsResponse = serde_json::from_str(&json)
        .map_err(|e| AppError::Parse(format!("Open Library 版本 JSON 解析失败: {e}")))?;

    for entry in editions.entries {
        if let Some(ia) = entry.ia.and_then(|ids| ids.into_iter().next()) {
            return Ok(ia);
        }
    }

    Err(AppError::Parse(
        "未找到 Internet Archive 全文标识，该书可能暂无在线可读版本".into(),
    ))
}

#[derive(Debug, Deserialize)]
struct OlEditionsResponse {
    entries: Vec<OlEdition>,
}

#[derive(Debug, Deserialize)]
struct OlEdition {
    ia: Option<Vec<String>>,
}

fn extract_work_key(url: &str) -> AppResult<String> {
    let re = Regex::new(r"(openlibrary\.org)(/works/OL[^/?#]+)").unwrap();
    re.captures(url)
        .map(|c| c[2].to_string())
        .ok_or_else(|| AppError::Parse(format!("无法解析 Open Library work key: {url}")))
}

fn extract_fragment_param(url: &str, key: &str) -> Option<String> {
    let fragment = url.split('#').nth(1)?;
    for pair in fragment.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts.next().map(|v| v.to_string());
        }
    }
    None
}

fn fetch_ia_plaintext(ia_id: &str) -> AppResult<String> {
    let candidates = [
        format!("https://archive.org/download/{ia_id}/{ia_id}.txt"),
        format!("https://archive.org/stream/{ia_id}/{ia_id}_djvu.txt"),
        format!("https://archive.org/download/{ia_id}/{ia_id}_djvu.txt"),
    ];
    for url in candidates {
        if let Ok(text) = fetch_html(&url) {
            if text.len() > 200 {
                return Ok(text);
            }
        }
    }
    Err(AppError::Network(format!(
        "无法从 Internet Archive 获取全文 ({ia_id})"
    )))
}

fn split_chapter_headings(text: &str) -> Vec<String> {
    let re = Regex::new(
        r"(?m)^[\s　]*(?:第\s*[\d〇零一二三四五六七八九十百千]+\s*[回章卷节]|Chapter\s+\d+|CHAPTER\s+[IVXLCDM\d]+)[^\n]{0,40}$",
    )
    .unwrap();
    re.find_iter(text)
        .map(|m| normalize_line(m.as_str()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn extract_chapter_text(text: &str, chapter_index: usize) -> AppResult<String> {
    let re = Regex::new(
        r"(?m)^[\s　]*(?:第\s*[\d〇零一二三四五六七八九十百千]+\s*[回章卷节]|Chapter\s+\d+|CHAPTER\s+[IVXLCDM\d]+)[^\n]{0,40}$",
    )
    .unwrap();
    let matches: Vec<_> = re.find_iter(text).collect();
    if matches.is_empty() {
        if chapter_index == 0 {
            return Ok(trim_ia_boilerplate(text));
        }
        return Err(AppError::Parse("章节索引超出范围".into()));
    }
    if chapter_index >= matches.len() {
        return Err(AppError::Parse(format!(
            "章节索引 {chapter_index} 超出范围（共 {} 章）",
            matches.len()
        )));
    }

    let start = matches[chapter_index].start();
    let end = matches
        .get(chapter_index + 1)
        .map(|m| m.start())
        .unwrap_or(text.len());
    Ok(trim_ia_boilerplate(&text[start..end]))
}

fn parse_chapter_url(url: &str) -> AppResult<(String, usize)> {
    let re = Regex::new(r"#ia-ch-(\d+)").unwrap();
    let index = re
        .captures(url)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| AppError::Parse(format!("无效的 IA 章节 URL: {url}")))?;
    let base = url.split('#').next().unwrap_or(url).to_string();
    Ok((base, index))
}

fn trim_ia_boilerplate(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("***") && !l.contains("Internet Archive"))
        .collect();
    lines.join("\n")
}

fn normalize_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{F900}'..='\u{FAFF}'
        )
    })
}
