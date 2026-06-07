//! 书架、阅读、导出、本地导入

use crate::spider::Chapter;
use crate::storage::{
    add_to_shelf, delete_book, export_book_to_file, get_book_detail as load_book_detail,
    import_local_file, list_books, read_chapter, update_book_progress, Book, BookDetail,
    ExportBookResult,
};
use crate::utils::AppResult;

/// 书架列表
pub fn list_shelf_books() -> AppResult<Vec<Book>> {
    list_books()
}

/// 加入书架
pub fn add_book_to_shelf(
    title: &str,
    catalog_url: &str,
    rule_json: &str,
    chapters: &[(String, String)],
) -> AppResult<Book> {
    add_to_shelf(title, catalog_url, rule_json, chapters)
}

/// 从书架移除
pub fn remove_book_from_shelf(book_id: i64) -> AppResult<()> {
    delete_book(book_id)
}

/// 书籍详情（含章节）
pub fn get_book_detail(book_id: i64) -> AppResult<BookDetail> {
    load_book_detail(book_id)
}

/// 阅读章节正文（成功后后台预加载下一章）
pub fn read_chapter_content(book_id: i64, chapter_id: i64) -> AppResult<String> {
    let content = read_chapter(book_id, chapter_id)?;
    schedule_preload_next_chapter(book_id, chapter_id);
    Ok(content)
}

/// 后台预加载下一章正文到缓存（不阻塞当前阅读）
fn schedule_preload_next_chapter(book_id: i64, chapter_id: i64) {
    let Ok(detail) = load_book_detail(book_id) else {
        return;
    };
    let Some(idx) = detail.chapters.iter().position(|c| c.id == chapter_id) else {
        return;
    };
    let Some(next) = detail.chapters.get(idx + 1) else {
        return;
    };
    if next.url.starts_with("local://") {
        return;
    }
    let next_id = next.id;
    std::thread::spawn(move || {
        let _ = read_chapter(book_id, next_id);
    });
}

/// 保存阅读进度
pub fn save_read_progress(book_id: i64, chapter_id: i64, offset: i64) -> AppResult<()> {
    update_book_progress(book_id, chapter_id, offset)
}

/// 导出全书 TXT（后台线程，不阻塞界面）
pub fn export_book(book_id: i64) -> AppResult<ExportBookResult> {
    export_book_to_file(book_id)
}

/// 导入本地 EPUB / TXT
pub fn import_local_book(file_path: &str) -> AppResult<Book> {
    import_local_file(file_path)
}

/// 将章节 DTO 转为存储用的 (title, url) 对
pub fn chapters_to_pairs(chapters: Vec<Chapter>) -> Vec<(String, String)> {
    chapters
        .into_iter()
        .map(|c| (c.title, c.url))
        .collect()
}
