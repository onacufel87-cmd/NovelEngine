import { defineStore } from "pinia";
import {
  createComment,
  updateComment,
  deleteComment,
  listChapterComments,
  listCommentsByBook,
} from "../api/comments";
import { formatAppError } from "../utils/appError";

/** 兼容后端嵌套 / 扁平两种 NoteListItem 结构 */
function normalizeListItem(raw) {
  if (raw?.note) return raw;
  const { book_title, chapter_title, ...note } = raw;
  return { note, book_title, chapter_title };
}

function normalizeGroups(groups) {
  return groups.map((group) => ({
    ...group,
    notes: group.notes.map(normalizeListItem),
  }));
}

export const useCommentStore = defineStore("comment", {
  state: () => ({
    /** 当前章节评论 */
    chapterComments: [],
    /** 按书分组的全部评论 */
    groupedComments: [],
    loading: false,
    chapterLoading: false,
    error: "",
  }),

  getters: {
    totalCount(state) {
      return state.groupedComments.reduce((sum, g) => sum + g.notes.length, 0);
    },
  },

  actions: {
    async loadChapterComments(bookId, chapterId) {
      if (!bookId || !chapterId) {
        this.chapterComments = [];
        return;
      }
      this.chapterLoading = true;
      try {
        this.chapterComments = await listChapterComments(bookId, chapterId);
      } catch (err) {
        this.error = formatAppError(err);
        this.chapterComments = [];
      } finally {
        this.chapterLoading = false;
      }
    },

    /** 划词后创建空评论（同锚点幂等） */
    async createHighlight(anchor, bookId, chapterId) {
      const existing = this.chapterComments.find(
        (n) =>
          n.start_offset === anchor.start_offset &&
          n.end_offset === anchor.end_offset
      );
      if (existing) return existing;

      const note = await createComment({
        book_id: bookId,
        chapter_id: chapterId,
        start_offset: anchor.start_offset,
        end_offset: anchor.end_offset,
        quote: anchor.quote,
        context_before: anchor.context_before,
        context_after: anchor.context_after,
        content_hash: anchor.content_hash,
        body: "",
      });

      if (!this.chapterComments.some((n) => n.id === note.id)) {
        this.chapterComments = [...this.chapterComments, note].sort(
          (a, b) => a.start_offset - b.start_offset
        );
      }
      return note;
    },

    async saveCommentBody(noteId, body) {
      const updated = await updateComment({ id: noteId, body });
      const idx = this.chapterComments.findIndex((n) => n.id === noteId);
      if (idx >= 0) this.chapterComments[idx] = updated;
      return updated;
    },

    async removeComment(noteId) {
      await deleteComment(noteId);
      this.chapterComments = this.chapterComments.filter((n) => n.id !== noteId);
      this.groupedComments = this.groupedComments
        .map((group) => ({
          ...group,
          notes: group.notes.filter((item) => item.note.id !== noteId),
        }))
        .filter((group) => group.notes.length > 0);
    },

    async loadGroupedComments() {
      this.loading = true;
      this.error = "";
      try {
        const groups = await listCommentsByBook();
        this.groupedComments = normalizeGroups(groups);
      } catch (err) {
        this.error = formatAppError(err);
        this.groupedComments = [];
      } finally {
        this.loading = false;
      }
    },

    clearChapterComments() {
      this.chapterComments = [];
    },
  },
});
