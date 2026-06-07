import { invoke } from "@tauri-apps/api/core";

/** 阅读、进度、导出 */

export async function getBookDetail(bookId) {
  return invoke("get_book_detail_cmd", { bookId });
}

export async function readChapterContent(bookId, chapterId) {
  return invoke("read_chapter_content", { bookId, chapterId });
}

export async function saveReadProgress(bookId, chapterId, offset) {
  return invoke("save_read_progress", { bookId, chapterId, offset });
}

export async function exportBook(bookId) {
  return invoke("export_book", { bookId });
}
