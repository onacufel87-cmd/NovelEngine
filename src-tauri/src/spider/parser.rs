use scraper::{Html, Selector};

use super::rule::{BookSource, Chapter};
use super::cleaner::clean_content_for_source;
use crate::utils::{AppError, AppResult};

/// 搜索结果中的单条记录（解析阶段，尚未绑定书源 ID）
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub title: String,
    pub author: Option<String>,
    pub catalog_url: String,
}

/// 榜单/列表中的书本条目
#[derive(Debug, Clone)]
pub struct BookListItemParsed {
    pub title: String,
    pub author: Option<String>,
    pub catalog_url: String,
}

/// 根据书源规则，从目录页 HTML 提取章节列表
pub fn parse_chapters(html: &str, source: &BookSource, base_url: &str) -> AppResult<Vec<Chapter>> {
    let document = Html::parse_document(html);
    let list_selector = Selector::parse(&source.chapter_list_selector)
        .map_err(|e| AppError::Parse(format!("chapter_list_selector 无效: {e}")))?;

    let mut chapters = Vec::new();

    for element in document.select(&list_selector) {
        let title = extract_chapter_title(element, &source.chapter_title_selector);

        let href = element
            .value()
            .attr("href")
            .unwrap_or("")
            .trim()
            .to_string();

        if href.is_empty() {
            continue;
        }

        let url = resolve_absolute_url(&href, base_url)?;

        chapters.push(Chapter {
            title,
            url,
        });
    }

    if chapters.is_empty() {
        return Err(AppError::Parse(format!(
            "未找到任何章节，请检查目录 URL 与选择器 \"{}\"",
            source.chapter_list_selector
        )));
    }

    Ok(chapters)
}

/// 从搜索页 HTML 提取书本列表
pub fn parse_search_results(
    html: &str,
    source: &BookSource,
    base_url: &str,
) -> AppResult<Vec<SearchResultItem>> {
    let result_selector = source
        .search_result_selector
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Parse("书源未配置 search_result_selector".into()))?;

    let document = Html::parse_document(html);
    let container_sel = Selector::parse(result_selector)
        .map_err(|e| AppError::Parse(format!("search_result_selector 无效: {e}")))?;

    let title_sel = source.search_title_selector.as_deref();
    let author_sel = source.search_author_selector.as_deref();
    let link_sel = source
        .search_link_selector
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("a");
    let link_attr = source
        .search_link_attr
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("href");

    let link_selector = if link_sel == "self" {
        None
    } else {
        Some(
            Selector::parse(link_sel)
                .map_err(|e| AppError::Parse(format!("search_link_selector 无效: {e}")))?,
        )
    };

    let mut results = Vec::new();

    for container in document.select(&container_sel) {
        let title = title_sel
            .map(|sel| extract_text_in_element(container, sel))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| normalize_inline_text(&container.text().collect::<String>()));

        if title.is_empty() {
            continue;
        }

        let author = author_sel
            .map(|sel| extract_text_in_element(container, sel))
            .filter(|t| !t.is_empty());

        let catalog_url = match extract_search_href(container, link_selector.as_ref(), link_sel, link_attr) {
            Some(href) => resolve_absolute_url(&href, base_url)?,
            None => continue,
        };

        results.push(SearchResultItem {
            title,
            author,
            catalog_url,
        });
    }

    Ok(results)
}

/// 从榜单页 HTML 提取书本列表（复用 book_list_selector 或 chapter_list_selector）
pub fn parse_book_list(
    html: &str,
    source: &BookSource,
    base_url: &str,
) -> AppResult<Vec<BookListItemParsed>> {
    let list_selector = source
        .book_list_selector
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(source.chapter_list_selector.as_str());

    let title_selector = source
        .book_title_selector
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(source.chapter_title_selector.as_str());

    let document = Html::parse_document(html);
    let list_sel = Selector::parse(list_selector)
        .map_err(|e| AppError::Parse(format!("book_list_selector 无效: {e}")))?;

    let mut books = Vec::new();

    for element in document.select(&list_sel) {
        let title = extract_chapter_title(element, title_selector);
        if title.is_empty() {
            continue;
        }

        let href = element
            .value()
            .attr("href")
            .unwrap_or("")
            .trim()
            .to_string();

        if href.is_empty() {
            continue;
        }

        let catalog_url = resolve_absolute_url(&href, base_url)?;

        books.push(BookListItemParsed {
            title,
            author: None,
            catalog_url,
        });
    }

    if books.is_empty() {
        return Err(AppError::Parse(format!(
            "榜单页未找到书本条目，请检查选择器 \"{list_selector}\""
        )));
    }

    Ok(books)
}

/// 从搜索结果条目提取链接 href
fn extract_search_href(
    container: scraper::ElementRef<'_>,
    link_selector: Option<&Selector>,
    link_sel: &str,
    link_attr: &str,
) -> Option<String> {
    // 条目本身即为链接（Gutenberg 等站点）
    if link_sel == "self" {
        return container
            .value()
            .attr(link_attr)
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty() && *h != "#");
    }

    if let Some(sel) = link_selector {
        if let Some(el) = container.select(sel).next() {
            if let Some(href) = el.value().attr(link_attr).filter(|h| !h.is_empty() && *h != "#") {
                return Some(href.trim().to_string());
            }
        }
    }

    // 回退：容器自身携带链接属性
    container
        .value()
        .attr(link_attr)
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty() && *h != "#")
}

/// 在元素内按子选择器提取文本
fn extract_text_in_element(element: scraper::ElementRef<'_>, selector: &str) -> String {
    let sel = selector.trim();
    if sel.is_empty() || sel == "text" {
        return normalize_inline_text(&element.text().collect::<String>());
    }

    if let Ok(child_sel) = Selector::parse(sel) {
        if let Some(el) = element.select(&child_sel).next() {
            return normalize_inline_text(&el.text().collect::<String>());
        }
    }

    String::new()
}

/// 提取章节标题：`text` 表示链接自身文本
fn extract_chapter_title(element: scraper::ElementRef<'_>, selector: &str) -> String {
    let sel = selector.trim();
    if sel.is_empty() || sel == "text" {
        return normalize_inline_text(&element.text().collect::<String>());
    }

    if let Ok(title_selector) = Selector::parse(sel) {
        if let Some(el) = element.select(&title_selector).next() {
            return normalize_inline_text(&el.text().collect::<String>());
        }
    }

    normalize_inline_text(&element.text().collect::<String>())
}

/// 将相对链接转为绝对 URL
fn resolve_absolute_url(href: &str, base_url: &str) -> AppResult<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Ok(href.to_string());
    }

    url::Url::parse(base_url)
        .and_then(|base| base.join(href))
        .map(|u| u.to_string())
        .map_err(|e| AppError::Parse(format!("章节链接 \"{href}\" 无法解析: {e}")))
}

/// 根据书源规则，从章节页 HTML 提取正文
pub fn parse_content(html: &str, source: &BookSource) -> AppResult<String> {
    let document = Html::parse_document(html);
    let content_selector = Selector::parse(&source.content_selector)
        .map_err(|e| AppError::Parse(format!("content_selector 无效: {e}")))?;

    let element = document
        .select(&content_selector)
        .next()
        .ok_or_else(|| AppError::Parse(format!(
            "未找到匹配选择器 \"{}\" 的正文区域",
            source.content_selector
        )))?;

    Ok(extract_text_with_paragraphs(element))
}

/// 解析章节正文并清洗噪音（集成 cleaner 模块）
pub fn parse_and_clean_content(html: &str, source: &BookSource) -> AppResult<String> {
    let raw = parse_content(html, source)?;
    Ok(clean_content_for_source(&raw, Some(source)))
}

/// 从正文中提取「下一页」链接（用于分页章节）
pub fn extract_next_page_url(
    html: &str,
    next_selector: &str,
    base_url: &str,
) -> AppResult<Option<String>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(next_selector)
        .map_err(|e| AppError::Parse(format!("next_page_selector 无效: {e}")))?;

    let href = document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|h| h.trim())
        .filter(|h| !h.is_empty() && *h != "#");

    match href {
        Some(relative) => {
            let absolute = url::Url::parse(base_url)
                .and_then(|base| base.join(relative))
                .map_err(|e| AppError::Parse(format!("下一页 URL 解析失败: {e}")))?;
            Ok(Some(absolute.to_string()))
        }
        None => Ok(None),
    }
}

/// 保留段落换行：优先按 <p> 分段，其次按 <br> 换行
fn extract_text_with_paragraphs(element: scraper::ElementRef<'_>) -> String {
    let p_selector = Selector::parse("p").unwrap();
    let paragraphs: Vec<String> = element
        .select(&p_selector)
        .map(|p| normalize_inline_text(&p.text().collect::<String>()))
        .filter(|s| !s.is_empty())
        .collect();

    if !paragraphs.is_empty() {
        return paragraphs.join("\n\n");
    }

    // 无 <p> 标签时，尝试保留 <br> 换行
    let inner = element.html();
    let with_breaks = inner
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");

    let fragment = Html::parse_fragment(&with_breaks);
    let text = fragment.root_element().text().collect::<String>();
    normalize_block_text(&text)
}

/// 合并行内多余空白
fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 合并块级文本中的多余空行
fn normalize_block_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spider::rule::BookSource;

    fn test_source(selector: &str) -> BookSource {
        BookSource {
            name: "测试".into(),
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
            content_selector: selector.into(),
            next_page_selector: None,
            encoding: None,
            ad_keywords: None,
            clean_patterns: None,
            cookies: None,
            request_interval_ms: None,
        }
    }

    fn catalog_source() -> BookSource {
        BookSource {
            name: "测试".into(),
            search_url: String::new(),
            search_result_selector: None,
            search_title_selector: None,
            search_author_selector: None,
            search_link_selector: None,
            search_link_attr: None,
            book_list_selector: None,
            book_title_selector: None,
            rank_urls: None,
            chapter_list_selector: "#list a".into(),
            chapter_title_selector: "text".into(),
            content_selector: "#content".into(),
            next_page_selector: None,
            encoding: None,
            ad_keywords: None,
            clean_patterns: None,
            cookies: None,
            request_interval_ms: None,
        }
    }

    #[test]
    fn parse_chapters_from_catalog() {
        let html = r#"
        <html><body>
          <ul id="list">
            <li><a href="/chapter/1.html">第一章 开端</a></li>
            <li><a href="/chapter/2.html">第二章 发展</a></li>
          </ul>
        </body></html>
        "#;
        let source = catalog_source();
        let chapters = parse_chapters(html, &source, "http://localhost:1420/book/").unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 开端");
        assert_eq!(
            chapters[0].url,
            "http://localhost:1420/chapter/1.html"
        );
    }

    #[test]
    fn parse_content_extracts_paragraphs() {
        let html = r#"
        <html><body>
          <div id="content">
            <p>第一段正文。</p>
            <p>第二段正文。</p>
          </div>
        </body></html>
        "#;
        let result = parse_content(html, &test_source("#content")).unwrap();
        assert!(result.contains("第一段正文。"));
        assert!(result.contains("第二段正文。"));
        assert!(result.contains("\n\n"));
    }
}
