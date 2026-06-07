import { watch, onMounted, nextTick } from "vue";
import { useBookStore } from "../stores/bookStore";
import { readChapterContent } from "../api";

/**
 * 阅读进度：保存滚动位置、恢复上次位置、80% 时预加载下一章
 * @param {import('vue').Ref<HTMLElement|null>} contentRef
 */
export function useReadingProgress(contentRef) {
  const bookStore = useBookStore();
  let lastReport = 0;
  let preloadedChapterId = null;

  /** 恢复滚动位置 */
  async function restoreScroll() {
    await nextTick();
    const el = contentRef.value;
    if (!el || !bookStore.content) return;

    const book = bookStore.currentBook;
    const shouldRestore =
      book &&
      book.last_chapter_id === bookStore.currentChapterId &&
      book.last_read_offset > 0;

    el.scrollTop = shouldRestore ? book.last_read_offset : 0;
  }

  /** 阅读到 80% 时预加载下一章 */
  async function tryPreloadAtScrollRatio(ratio) {
    if (ratio < 0.8 || !bookStore.currentBook) return;

    const idx = bookStore.currentChapterIndex;
    const next = bookStore.chapters[idx + 1];
    if (!next || preloadedChapterId === next.id) return;

    preloadedChapterId = next.id;
    try {
      await readChapterContent(bookStore.currentBook.id, next.id);
    } catch {
      preloadedChapterId = null;
    }
  }

  function onScroll() {
    const el = contentRef.value;
    if (!el || !bookStore.currentChapterId) return;

    const scrollTop = Math.floor(el.scrollTop);
    const maxScroll = el.scrollHeight - el.clientHeight;
    const ratio = maxScroll > 0 ? scrollTop / maxScroll : 0;

    tryPreloadAtScrollRatio(ratio);

    const now = Date.now();
    if (now - lastReport < 2000) return;
    if (scrollTop > 0) {
      lastReport = now;
      bookStore.reportProgress(scrollTop);
    }
  }

  watch(
    () => bookStore.currentChapterId,
    () => {
      lastReport = 0;
      preloadedChapterId = null;
    }
  );

  watch(
    () => bookStore.content,
    () => {
      if (bookStore.content) restoreScroll();
    }
  );

  onMounted(() => {
    if (bookStore.content) restoreScroll();
  });

  return { onScroll };
}
