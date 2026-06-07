use regex::Regex;
use scraper::{Html, Selector};

use super::cleaner::apply_global_clean;
use super::fetcher::fetch_html;
use super::parser::SearchResultItem;
use super::rule::Chapter;
use crate::utils::{AppError, AppResult};

const WIKISOURCE_ZH_ORIGIN: &str = "https://zh.wikisource.org";

/// 是否为中文维基文库页面
pub fn is_wikisource_url(url: &str) -> bool {
    url.contains("wikisource.org/wiki/") || url.contains("wikisource.org/w/index.php")
}

/// 在中文维基文库搜索公版书籍
pub fn search(keyword: &str) -> AppResult<Vec<SearchResultItem>> {
    let encoded: String = url::form_urlencoded::byte_serialize(keyword.trim().as_bytes()).collect();
    let search_url = format!(
        "{WIKISOURCE_ZH_ORIGIN}/w/index.php?title=Special:Search&search={encoded}&fulltext=1"
    );
    let html = fetch_html(&search_url)?;

    let document = Html::parse_document(&html);
    let heading_sel = Selector::parse(".mw-search-result-heading a")
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let mut results = Vec::new();
    for link in document.select(&heading_sel) {
        let title = normalize_ws(&link.text().collect::<String>());
        let href = link.value().attr("href").unwrap_or("").trim();
        if title.is_empty() || href.is_empty() {
            continue;
        }
        // 跳过讨论页、特殊页
        if href.contains("讨论:") || href.contains("Special:") {
            continue;
        }
        let catalog_url = resolve_wiki_url(href)?;
        results.push(SearchResultItem {
            title,
            author: None,
            catalog_url,
        });
        if results.len() >= 30 {
            break;
        }
    }

    Ok(results)
}

/// 拉取书籍目录：优先 PrefixIndex 子页面，其次正文内章节链接
pub fn fetch_chapters(catalog_url: &str) -> AppResult<Vec<Chapter>> {
    let book_title = extract_book_title(catalog_url)?;

    // 先尝试 Special:PrefixIndex 列出所有子页面（章回小说常用）
    if let Ok(chapters) = fetch_chapters_via_prefix_index(&book_title) {
        if chapters.len() >= 2 {
            return Ok(chapters);
        }
    }

    let html = fetch_html(catalog_url)?;
    let chapters = extract_chapter_links(&html, catalog_url, &book_title)?;
    if chapters.len() >= 2 {
        return Ok(chapters);
    }

    // 无分章时整页作为一章
    Ok(vec![Chapter {
        title: "全文".into(),
        url: catalog_url.to_string(),
    }])
}

/// 读取维基文库章节正文
pub fn fetch_chapter_content(chapter_url: &str) -> AppResult<String> {
    let html = fetch_html(chapter_url)?;
    let raw = extract_wikisource_body(&html)?;
    Ok(apply_global_clean(&raw))
}

fn fetch_chapters_via_prefix_index(book_title: &str) -> AppResult<Vec<Chapter>> {
    let prefix: String = url::form_urlencoded::byte_serialize(format!("{book_title}/").as_bytes()).collect();
    let url = format!(
        "{WIKISOURCE_ZH_ORIGIN}/w/index.php?title=Special:PrefixIndex&prefix={prefix}&namespace=0"
    );
    let html = fetch_html(&url)?;

    let document = Html::parse_document(&html);
    let link_sel = Selector::parse(".mw-prefixindex-list a, ul.mw-prefixindex-list a")
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let chapter_re = Regex::new(r"第\s*[\d〇零一二三四五六七八九十百千]+\s*[回章卷节]").unwrap();
    let mut chapters = Vec::new();

    for link in document.select(&link_sel) {
        let title = normalize_ws(&link.text().collect::<String>());
        let href = link.value().attr("href").unwrap_or("").trim();
        if title.is_empty() || href.is_empty() {
            continue;
        }
        // 只保留像章节的子页面
        if !chapter_re.is_match(&title) && !title.contains('回') {
            continue;
        }
        chapters.push(Chapter {
            title,
            url: resolve_wiki_url(href)?,
        });
    }

    Ok(chapters)
}

fn extract_chapter_links(html: &str, base_url: &str, book_title: &str) -> AppResult<Vec<Chapter>> {
    let document = Html::parse_document(html);
    let link_sel = Selector::parse("#mw-content-text .mw-parser-output a[href]")
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let chapter_re = Regex::new(r"第\s*[\d〇零一二三四五六七八九十百千]+\s*[回章卷节]").unwrap();
    let mut chapters = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for link in document.select(&link_sel) {
        let title = normalize_ws(&link.text().collect::<String>());
        let href = link.value().attr("href").unwrap_or("").trim();
        if title.is_empty() || href.is_empty() {
            continue;
        }
        if !href_under_book(href, book_title) && !chapter_re.is_match(&title) {
            continue;
        }
        if title.len() > 80 {
            continue;
        }
        let url = resolve_wiki_url(href)?;
        if !seen.insert(url.clone()) {
            continue;
        }
        chapters.push(Chapter { title, url });
    }

    // 若正文链接不足，尝试目录 #toc
    if chapters.len() < 2 {
        let toc_sel = Selector::parse("#toc a").map_err(|e| AppError::Parse(e.to_string()))?;
        for link in document.select(&toc_sel) {
            let title = normalize_ws(&link.text().collect::<String>());
            let href = link.value().attr("href").unwrap_or("").trim();
            if title.is_empty() || !href.starts_with('#') {
                continue;
            }
            chapters.push(Chapter {
                title,
                url: format!("{base_url}{href}"),
            });
        }
    }

    Ok(chapters)
}

fn extract_wikisource_body(html: &str) -> AppResult<String> {
    let document = Html::parse_document(html);
    let content_sel = Selector::parse("#mw-content-text .mw-parser-output")
        .map_err(|e| AppError::Parse(e.to_string()))?;
    let root = document
        .select(&content_sel)
        .next()
        .ok_or_else(|| AppError::Parse("维基文库正文区域未找到".into()))?;

    let p_sel = Selector::parse("p").unwrap();
    let div_sel = Selector::parse("div.poeme").unwrap();
    let mut parts: Vec<String> = Vec::new();

    for p in root.select(&p_sel) {
        let text = normalize_ws(&p.text().collect::<String>());
        if !text.is_empty() && !is_boilerplate(&text) {
            parts.push(text);
        }
    }

    // 诗词块（部分页面用 div.poeme）
    for div in root.select(&div_sel) {
        let text = normalize_ws(&div.text().collect::<String>());
        if !text.is_empty() && !is_boilerplate(&text) {
            parts.push(text);
        }
    }

    if parts.is_empty() {
        // 诗词等无 p 标签时取整段文本
        let text = normalize_ws(&root.text().collect::<String>());
        if text.is_empty() {
            return Err(AppError::Parse("维基文库页面无正文内容".into()));
        }
        parts.push(text);
    }

    Ok(parts.join("\n\n"))
}

fn extract_book_title(catalog_url: &str) -> AppResult<String> {
    let parsed = url::Url::parse(catalog_url)
        .map_err(|e| AppError::Parse(format!("无效维基文库 URL: {e}")))?;
    let path = parsed.path();
    let wiki_path = path
        .strip_prefix("/wiki/")
        .ok_or_else(|| AppError::Parse(format!("非维基文库书籍 URL: {catalog_url}")))?;
    let title = wiki_path
        .split('/')
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Parse("无法解析书籍标题".into()))?;
    // URL 路径段仍为百分号编码，需解码为中文标题
    Ok(urlencoding_helper(title))
}

fn urlencoding_helper(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, _)| k.into_owned())
        .collect()
}

fn resolve_wiki_url(href: &str) -> AppResult<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Ok(href.to_string());
    }
    if href.starts_with('/') {
        return Ok(format!("{WIKISOURCE_ZH_ORIGIN}{href}"));
    }
    Ok(format!("{WIKISOURCE_ZH_ORIGIN}/{href}"))
}

fn href_under_book(href: &str, book_title: &str) -> bool {
    let wiki_path = href
        .split("wikisource.org")
        .nth(1)
        .unwrap_or(href)
        .trim_start_matches("/wiki/");
    let page_path = wiki_path.split('#').next().unwrap_or(wiki_path);
    let decoded = urlencoding_helper(page_path);
    decoded.starts_with(&format!("{book_title}/"))
}

fn is_boilerplate(s: &str) -> bool {    let t = s.trim();
    t.starts_with("检索自")
        || t.contains("维基文库")
        || t.starts_with("分类：")
        || t.starts_with("姊妹计划")
        || t.contains("Public domain")
        || t.contains("falsefalse")
        || (t.contains("上一回") && t.contains("下一回"))
        || t == "回目录"
        || (t.contains("公有领域") && t.contains("作者逝世"))
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
