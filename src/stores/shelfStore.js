import { defineStore } from "pinia";
import {
  getShelfBooks,
  addBookToShelf,
  addBookFromSearch,
  importLocalBook,
  removeBookFromShelf,
} from "../api";
import { formatAppError } from "../utils/appError";

export const useShelfStore = defineStore("shelf", {
  state: () => ({
    books: [],
    loading: false,
    error: "",
  }),

  actions: {
    /** 从 SQLite 加载书架列表 */
    async loadFromDB() {
      this.loading = true;
      this.error = "";
      try {
        this.books = await getShelfBooks();
      } catch (err) {
        this.error = formatAppError(err);
        this.books = [];
      } finally {
        this.loading = false;
      }
    },

    /** 加入书架并刷新列表 */
    async addBook({ title, catalogUrl, rule, chapters }) {
      const book = await addBookToShelf({ title, catalogUrl, rule, chapters });
      await this.loadFromDB();
      return book;
    },

    /** 从搜索/榜单结果加入书架（后端自动解析目录） */
    async addBookFromSearchResult({ title, author, catalogUrl, sourceId }) {
      const book = await addBookFromSearch({
        title,
        author,
        catalogUrl,
        sourceId,
      });
      await this.loadFromDB();
      return book;
    },

    /** 从书架删除 */
    async removeBook(bookId) {
      await removeBookFromShelf(bookId);
      this.books = this.books.filter((b) => b.id !== bookId);
    },

    /** 导入本地 EPUB / TXT */
    async importLocal(filePath) {
      const book = await importLocalBook(filePath);
      await this.loadFromDB();
      return book;
    },
  },
});
