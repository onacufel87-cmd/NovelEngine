use std::path::{Path, PathBuf};

use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};

use super::books::{add_book_with_chapters, Book};
use super::chapters::save_chapters_with_content;
use crate::utils::{AppError, AppResult};

/// 本地书籍标记用规则 JSON（不联网抓取）
pub const LOCAL_SOURCE_RULE_JSON: &str = r##"{
  "name": "本地文件",
  "kind": "local",
  "search_url": "",
  "chapter_list_selector": "-",
  "content_selector": "-"
}"##;

/// 从本地 EPUB / TXT 导入并加入书架（章节正文预写入数据库）
pub fn import_local_file(file_path: &str) -> AppResult<Book> {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Err(AppError::InvalidRule(format!("文件不存在: {file_path}")));
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let parsed = match ext.as_str() {
        "txt" => parse_txt(&path)?,
        "epub" => parse_epub(&path)?,
        _ => {
            return Err(AppError::InvalidRule(
                "仅支持 .txt 与 .epub 格式".into(),
            ))
        }
    };

    let file_url = format!("file://{}", path.to_string_lossy());
    let book = add_book_with_chapters(
        &parsed.title,
        parsed.author.as_deref(),
        &file_url,
        "本地文件",
        LOCAL_SOURCE_RULE_JSON,
        Some("local"),
        &parsed
            .chapters
            .iter()
            .map(|(title, _)| (title.clone(), String::new()))
            .collect::<Vec<_>>(),
    )?;

    let with_urls: Vec<(String, String, String)> = parsed
        .chapters
        .into_iter()
        .enumerate()
        .map(|(i, (title, content))| {
            (
                title,
                format!("local://{}/chapter/{}", book.id, i),
                content,
            )
        })
        .collect();

    // 覆盖章节 URL 并写入正文缓存
    replace_local_chapters(book.id, &with_urls)?;

    get_book_by_id_after_import(book.id)
}

struct ParsedLocalBook {
    title: String,
    author: Option<String>,
    chapters: Vec<(String, String)>,
}

fn parse_txt(path: &Path) -> AppResult<ParsedLocalBook> {
    let bytes = std::fs::read(path).map_err(|e| AppError::Parse(format!("读取 TXT 失败: {e}")))?;
    let text = decode_text_bytes(&bytes)?;

    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "未命名".into());

    let chapters = split_txt_chapters(&text, &stem);
    Ok(ParsedLocalBook {
        title: stem,
        author: None,
        chapters,
    })
}

fn parse_epub(path: &Path) -> AppResult<ParsedLocalBook> {
    let mut doc = EpubDoc::new(path).map_err(|e| AppError::Parse(format!("EPUB 打开失败: {e}")))?;

    let title = doc
        .mdata("title")
        .map(|v| v.value.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "未命名".into())
        });

    let author = doc
        .mdata("creator")
        .map(|v| v.value.trim().to_string())
        .filter(|a| !a.is_empty());

    let num_chapters = doc.get_num_chapters();
    if num_chapters == 0 {
        return Err(AppError::Parse("EPUB 无可读章节".into()));
    }

    let mut chapters = Vec::new();
    for i in 0..num_chapters {
        if !doc.set_current_chapter(i) {
            continue;
        }

        let (html, mime) = match doc.get_current_str() {
            Some(v) => v,
            None => continue,
        };

        let plain = html_to_plain(&html);
        if !plain.trim().is_empty() {
            let chapter_title = if mime.contains("html") || mime.contains("xhtml") {
                extract_html_title(&html).unwrap_or_else(|| format!("第 {} 节", chapters.len() + 1))
            } else {
                format!("第 {} 节", chapters.len() + 1)
            };
            chapters.push((chapter_title, plain));
        }
    }

    if chapters.is_empty() {
        return Err(AppError::Parse("EPUB 未能提取到正文".into()));
    }

    Ok(ParsedLocalBook {
        title,
        author,
        chapters,
    })
}

/// TXT 按常见章节标题切分
fn split_txt_chapters(text: &str, fallback_title: &str) -> Vec<(String, String)> {
    let re = Regex::new(
        r"(?m)^(?:第[0-9一二三四五六七八九十百千零两]+[章节回卷集部篇][^\n]{0,40}|Chapter\s+[0-9IVXLC]+[^\n]{0,40})$",
    )
    .unwrap();

    let mut starts: Vec<(usize, String)> = Vec::new();
    for cap in re.find_iter(text) {
        starts.push((cap.start(), cap.as_str().trim().to_string()));
    }

    if starts.is_empty() {
        return vec![(fallback_title.to_string(), text.trim().to_string())];
    }

    let mut chapters = Vec::new();
    for (idx, (start, title)) in starts.iter().enumerate() {
        let end = starts.get(idx + 1).map(|(s, _)| *s).unwrap_or(text.len());
        let body = text[*start..end].trim().to_string();
        if !body.is_empty() {
            chapters.push((title.clone(), body));
        }
    }

    if chapters.is_empty() {
        vec![(fallback_title.to_string(), text.trim().to_string())]
    } else {
        chapters
    }
}

/// HTML 片段转纯文本（保留段落）
fn html_to_plain(html: &str) -> String {
    let document = Html::parse_fragment(html);
    let p_sel = Selector::parse("p").ok();
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref sel) = p_sel {
        for p in document.select(sel) {
            let t: String = p.text().collect();
            let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            if !t.is_empty() {
                parts.push(t);
            }
        }
    }

    if !parts.is_empty() {
        return parts.join("\n\n");
    }

    let text: String = document.root_element().text().collect();
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_html_title(html: &str) -> Option<String> {
    let document = Html::parse_fragment(html);
    for sel in ["h1", "h2", "title"] {
        if let Ok(s) = Selector::parse(sel) {
            if let Some(el) = document.select(&s).next() {
                let t: String = el.text().collect();
                let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
    }
    None
}

fn decode_text_bytes(bytes: &[u8]) -> AppResult<String> {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }
    let (cow, _, _) = encoding_rs::GBK.decode(bytes);
    Ok(cow.into_owned())
}

/// 删除占位章节并写入带正文的本地章节
fn replace_local_chapters(book_id: i64, chapters: &[(String, String, String)]) -> AppResult<()> {
    let conn = super::db::get_connection()?;
    conn.execute(
        "DELETE FROM chapters WHERE book_id = ?1",
        rusqlite::params![book_id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    save_chapters_with_content(book_id, chapters)
}

fn get_book_by_id_after_import(book_id: i64) -> AppResult<Book> {
    super::books::get_book_by_id(book_id)
}
