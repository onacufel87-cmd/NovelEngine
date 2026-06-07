use super::books::{add_book_with_chapters, get_book_by_id, get_source_rule_json, Book};
use super::chapters::{cache_content, get_chapter, list_chapters, ChapterRecord};
use super::sources::get_source_by_id;
use crate::spider::{fetch_and_parse_chapters, fetch_and_parse_content};
use crate::utils::AppResult;

/// 书籍详情（含章节列表）
#[derive(Debug, serde::Serialize)]
pub struct BookDetail {
    pub book: Book,
    pub chapters: Vec<ChapterRecord>,
}

/// 获取书籍详情
pub fn get_book_detail(book_id: i64) -> AppResult<BookDetail> {
    let book = get_book_by_id(book_id)?;
    let chapters = list_chapters(book_id)?;
    Ok(BookDetail { book, chapters })
}

/// 阅读章节：优先读缓存，否则联网抓取并缓存
pub fn read_chapter(book_id: i64, chapter_id: i64) -> AppResult<String> {
    let chapter = get_chapter(chapter_id)?;
    if chapter.book_id != book_id {
        return Err(crate::utils::AppError::Database(
            "章节不属于该书籍".into(),
        ));
    }

    if let Some(ref cached) = chapter.content {
        if !cached.is_empty() {
            return Ok(finalize_chapter_content(cached));
        }
    }

    // 优先从文件系统读取正文缓存
    if let Some(disk) = super::chapters::load_chapter_content(&chapter) {
        return Ok(finalize_chapter_content(&disk));
    }

    // 本地导入的书籍不联网，正文应在导入时已写入
    if chapter.url.starts_with("local://") {
        return Err(crate::utils::AppError::Parse(
            "本地书籍章节内容缺失，请重新导入文件".into(),
        ));
    }

    let rule_json = get_source_rule_json(book_id)?;
    if rule_json.contains("\"kind\":\"local\"") {
        return Err(crate::utils::AppError::Parse(
            "本地书籍章节内容缺失，请重新导入文件".into(),
        ));
    }

    let content = fetch_and_parse_content(&chapter.url, &rule_json)?;
    cache_content(chapter_id, &content)?;
    Ok(crate::spider::cleaner::apply_global_clean(&content))
}

/// 对已缓存正文再套一层全局清洗（用户可在设置中随时调整规则）
fn finalize_chapter_content(content: &str) -> String {
    crate::spider::cleaner::apply_global_clean(content)
}

/// 加入书架
pub fn add_to_shelf(
    title: &str,
    catalog_url: &str,
    rule_json: &str,
    chapters: &[(String, String)],
) -> AppResult<Book> {
    let source = crate::spider::rule::parse_book_source(rule_json)?;
    add_book_with_chapters(
        title,
        None,
        catalog_url,
        &source.name,
        rule_json,
        None,
        chapters,
    )
}

/// 从搜索结果加入书架：按书源规则自动解析目录
pub fn add_from_search(
    title: &str,
    author: Option<&str>,
    catalog_url: &str,
    source_id: &str,
) -> AppResult<Book> {
    let source_record = get_source_by_id(source_id)?;
    let rule_json = source_record.rule_json.clone();
    let chapters = fetch_and_parse_chapters(catalog_url, &rule_json)?;
    let pairs: Vec<(String, String)> = chapters
        .iter()
        .map(|c| (c.title.clone(), c.url.clone()))
        .collect();
    let rule = crate::spider::rule::parse_book_source(&rule_json)?;

    add_book_with_chapters(
        title,
        author,
        catalog_url,
        &rule.name,
        &rule_json,
        Some(source_id),
        &pairs,
    )
}

/// 导出结果（仅写入已缓存章节，避免导出时联网卡死 UI）
#[derive(Debug, serde::Serialize)]
pub struct ExportBookResult {
    pub path: String,
    pub total_chapters: usize,
    pub exported_chapters: usize,
}

/// 仅读取已缓存正文，不触发联网抓取
pub fn read_chapter_if_cached(book_id: i64, chapter_id: i64) -> AppResult<Option<String>> {
    let chapter = get_chapter(chapter_id)?;
    if chapter.book_id != book_id {
        return Err(crate::utils::AppError::Database(
            "章节不属于该书籍".into(),
        ));
    }

    if let Some(ref cached) = chapter.content {
        if !cached.is_empty() {
            return Ok(Some(finalize_chapter_content(cached)));
        }
    }

    if let Some(disk) = super::chapters::load_chapter_content(&chapter) {
        return Ok(Some(finalize_chapter_content(&disk)));
    }

    Ok(None)
}

/// 导出到用户文档目录（只含已缓存章节，未读章节留占位说明）
pub fn export_book_to_file(book_id: i64) -> AppResult<ExportBookResult> {
    let detail = get_book_detail(book_id)?;
    let total = detail.chapters.len();
    let mut exported = 0usize;
    let mut text = format!("《{}》\n", detail.book.title);

    for ch in &detail.chapters {
        text.push_str(&format!("\n\n{}\n\n", ch.title));
        if let Some(content) = read_chapter_if_cached(book_id, ch.id)? {
            text.push_str(&content);
            exported += 1;
        } else {
            text.push_str("[未缓存：请先在应用内阅读该章后再导出完整正文]\n");
        }
    }

    let user_dirs = directories::UserDirs::new()
        .ok_or_else(|| crate::utils::AppError::Database("无法获取用户目录".into()))?;
    let docs = user_dirs
        .document_dir()
        .ok_or_else(|| crate::utils::AppError::Database("无法获取文档目录".into()))?
        .to_path_buf();

    let export_dir = docs.join("NovelReaderCore");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| crate::utils::AppError::Database(format!("创建导出目录失败: {e}")))?;

    let safe_name: String = detail
        .book
        .title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&c) {
                c
            } else {
                '_'
            }
        })
        .collect();

    let path = export_dir.join(format!("{safe_name}.txt"));
    std::fs::write(&path, &text)
        .map_err(|e| crate::utils::AppError::Database(format!("写入文件失败: {e}")))?;

    Ok(ExportBookResult {
        path: path.to_string_lossy().into_owned(),
        total_chapters: total,
        exported_chapters: exported,
    })
}
