import { defineStore } from "pinia";
import { getBookDetail, readChapterContent, saveReadProgress } from "../api";
import { formatAppError } from "../utils/appError";
export const useBookStore = defineStore("book", {
  state: () => ({
    currentBook: null,
    chapters: [],
    currentChapterId: null,
    content: "",
    loading: false,
    error: "",
  }),

  getters: {
    currentChapter(state) {
      return state.chapters.find((c) => c.id === state.currentChapterId) ?? null;
    },
    currentChapterIndex(state) {
      return state.chapters.findIndex((c) => c.id === state.currentChapterId);
    },
    hasPrev(state) {
      const idx = state.chapters.findIndex((c) => c.id === state.currentChapterId);
      return idx > 0;
    },
    hasNext(state) {
      const idx = state.chapters.findIndex((c) => c.id === state.currentChapterId);
      return idx >= 0 && idx < state.chapters.length - 1;
    },
  },

  actions: {
    /** 打开书籍：加载章节列表并恢复阅读进度 */
    async openBook(bookId) {
      this.loading = true;
      this.error = "";
      try {
        const detail = await getBookDetail(bookId);
        this.currentBook = detail.book;
        this.chapters = detail.chapters;

        const resumeId =
          detail.book.last_chapter_id ?? detail.chapters[0]?.id ?? null;
        if (resumeId) {
          await this.loadChapter(resumeId);
        }
      } catch (err) {
        this.error = formatAppError(err);
        this.currentBook = null;
        this.chapters = [];
      } finally {
        this.loading = false;
      }
    },

    /** 加载章节正文（优先读 SQLite 缓存） */
    async loadChapter(chapterId) {
      if (!this.currentBook) return;

      this.currentChapterId = chapterId;
      this.loading = true;
      this.error = "";
      this.content = "";

      try {
        this.content = await readChapterContent(this.currentBook.id, chapterId);
      } catch (err) {
        this.error = formatAppError(err);
        this.content = "";
      } finally {
        this.loading = false;
      }
    },

    async loadPrevChapter() {
      const idx = this.currentChapterIndex;
      if (idx > 0) {
        await this.loadChapter(this.chapters[idx - 1].id);
      }
    },

    async loadNextChapter() {
      const idx = this.currentChapterIndex;
      if (idx >= 0 && idx < this.chapters.length - 1) {
        await this.loadChapter(this.chapters[idx + 1].id);
      }
    },

    /** 上报阅读进度到 SQLite，并同步本地状态 */
    async reportProgress(offset = 0) {
      if (!this.currentBook || !this.currentChapterId) return;
      try {
        await saveReadProgress(
          this.currentBook.id,
          this.currentChapterId,
          offset
        );
        this.currentBook.last_chapter_id = this.currentChapterId;
        this.currentBook.last_read_offset = offset;
      } catch {
        // 进度保存失败静默处理
      }
    },

    reset() {
      this.currentBook = null;
      this.chapters = [];
      this.currentChapterId = null;
      this.content = "";
      this.error = "";
    },
  },
});
