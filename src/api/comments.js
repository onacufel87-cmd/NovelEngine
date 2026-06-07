import { invoke } from "@tauri-apps/api/core";

/** 创建划词评论（正文可为空） */
export async function createComment(payload) {
  return invoke("create_reader_note", { payload });
}

/** 更新评论正文 */
export async function updateComment(payload) {
  return invoke("update_reader_note", { payload });
}

/** 删除评论 */
export async function deleteComment(noteId) {
  return invoke("delete_reader_note", { noteId });
}

/** 某章全部评论（阅读页渲染） */
export async function listChapterComments(bookId, chapterId) {
  return invoke("list_chapter_notes", { bookId, chapterId });
}

/** 按书分组全部评论（评论管理页） */
export async function listCommentsByBook() {
  return invoke("list_reader_notes_by_book");
}
