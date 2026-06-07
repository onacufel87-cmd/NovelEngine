<template>

  <div

    class="reader-view"

    :class="settingStore.effectiveReaderThemeClass"

    :style="readerStyle"

  >

    <header class="reader-toolbar">

      <RouterLink to="/" class="back-link">← 书架</RouterLink>

      <span class="chapter-title">{{ bookStore.currentChapter?.title ?? "阅读中" }}</span>

      <div class="toolbar-actions">

        <RouterLink

          v-if="chapterCommentCount > 0"

          :to="`/notes?book=${bookStore.currentBook?.id}`"

          class="comment-count-link"

        >

          评论 {{ chapterCommentCount }}

        </RouterLink>

        <button

          v-if="bookStore.currentBook"

          type="button"

          class="icon-btn"

          title="导出 TXT（仅已读/已缓存章节）"

          :disabled="exporting"

          @click="handleExport"

        >

          {{ exporting ? "…" : "↓" }}

        </button>

        <button type="button" class="settings-btn" title="设置" @click="toggleSettings">⚙</button>

      </div>

    </header>



    <SettingPanel v-if="showSettings" mode="popover" @close="showSettings = false" />



    <p v-if="bookStore.error" class="reader-error">{{ bookStore.error }}</p>

    <p v-if="exportMessage" class="export-msg">{{ exportMessage }}</p>

    <p v-if="noteError" class="reader-error">{{ noteError }}</p>



    <article ref="contentRef" class="reader-content" @scroll="onScroll">

      <LoadingSpinner v-if="bookStore.loading" text="加载章节中…" />

      <ChapterWithComments

        v-else-if="displayContent"

        :plain-text="displayContent"

        :comments="commentStore.chapterComments"

        :active-comment-id="activeCommentId"

        @selection-change="onSelectionChange"

        @comment-click="openCommentEditor"

      />

      <p v-else class="placeholder">暂无正文</p>

    </article>



    <NoteSelectionBar

      :visible="selectionBar.visible"

      :rect="selectionBar.rect"

      :busy="creatingComment"

      @comment="handleCreateComment"

    />



    <CommentEditor

      layout="drawer"

      :visible="editor.visible"

      :quote="editor.quote"

      :initial-body="editor.body"

      :is-new="editor.isNew"

      @close="closeEditor"

      @save="handleSaveComment"

    />



    <ReaderBottomBar @open-catalog="toggleCatalog" />



    <ChapterCatalogPanel :visible="showCatalog" @close="showCatalog = false" />

  </div>

</template>



<script setup>

import { ref, computed, watch, nextTick } from "vue";

import { useRoute } from "vue-router";

import { useBookStore } from "../../stores/bookStore";

import { useCommentStore } from "../../stores/commentStore";

import { useSettingStore } from "../../stores/settingStore";

import { useReadingProgress } from "../../hooks/useReadingProgress";

import { exportBook } from "../../api";

import { getSelectionAnchor, clearSelection } from "../../utils/textAnchor";

import { formatAppError } from "../../utils/appError";

import SettingPanel from "./SettingPanel.vue";

import ReaderBottomBar from "./ReaderBottomBar.vue";

import ChapterCatalogPanel from "./ChapterCatalogPanel.vue";

import ChapterWithComments from "./ChapterWithComments.vue";

import CommentEditor from "./CommentEditor.vue";

import NoteSelectionBar from "./NoteSelectionBar.vue";

import LoadingSpinner from "../Common/LoadingSpinner.vue";



const route = useRoute();

const bookStore = useBookStore();

const commentStore = useCommentStore();

const settingStore = useSettingStore();

const showSettings = ref(false);

const showCatalog = ref(false);

const contentRef = ref(null);

const exportMessage = ref("");

const exporting = ref(false);

const noteError = ref("");

const activeCommentId = ref(null);

const creatingComment = ref(false);



const selectionBar = ref({ visible: false, rect: null, anchor: null });

const editor = ref({

  visible: false,

  noteId: null,

  quote: "",

  body: "",

  isNew: true,

});



const readerStyle = computed(() => ({

  fontSize: `${settingStore.fontSize}px`,

  lineHeight: settingStore.lineHeight,

  fontFamily: settingStore.fontFamilyCss,

}));



const displayContent = computed(() => bookStore.content);

const chapterCommentCount = computed(() => commentStore.chapterComments.length);



watch(

  () => settingStore.chineseVariant,

  () => {

    if (bookStore.currentChapterId) {

      bookStore.loadChapter(bookStore.currentChapterId);

    }

  }

);



/** 章节切换时加载本章评论 */

watch(

  () => [bookStore.currentBook?.id, bookStore.currentChapterId],

  async ([bookId, chapterId]) => {

    selectionBar.value = { visible: false, rect: null, anchor: null };

    if (bookId && chapterId) {

      await commentStore.loadChapterComments(bookId, chapterId);

    } else {

      commentStore.clearChapterComments();

    }

  },

  { immediate: true }

);



/** 从评论页跳转：定位到指定评论 */

watch(

  () => [route.query.note, bookStore.content, commentStore.chapterComments.length],

  async () => {

    const noteId = Number(route.query.note);

    if (!noteId || !bookStore.content) return;



    await nextTick();

    activeCommentId.value = noteId;

    const el = contentRef.value?.querySelector(`[data-note-id="${noteId}"]`);

    el?.scrollIntoView({ block: "center", behavior: "smooth" });

    setTimeout(() => {

      if (activeCommentId.value === noteId) activeCommentId.value = null;

    }, 2500);

  }

);



const { onScroll } = useReadingProgress(contentRef);



function onSelectionChange(containerEl) {

  if (!containerEl || !displayContent.value) {

    selectionBar.value = { visible: false, rect: null, anchor: null };

    return;

  }



  const anchor = getSelectionAnchor(containerEl, displayContent.value);

  if (!anchor) {

    selectionBar.value = { visible: false, rect: null, anchor: null };

    return;

  }



  const sel = window.getSelection();

  const range = sel?.rangeCount ? sel.getRangeAt(0) : null;

  const rect = range?.getBoundingClientRect?.();

  if (!rect || (rect.width === 0 && rect.height === 0)) {

    selectionBar.value = { visible: false, rect: null, anchor: null };

    return;

  }



  selectionBar.value = { visible: true, rect, anchor };

}



/** 先标记评论锚点，再打开右侧评论栏（防连击重复提交） */

async function handleCreateComment() {

  if (creatingComment.value) return;



  const { anchor } = selectionBar.value;

  if (!anchor || !bookStore.currentBook || !bookStore.currentChapterId) return;



  creatingComment.value = true;

  noteError.value = "";

  selectionBar.value = { visible: false, rect: null, anchor: null };

  clearSelection();



  try {

    const note = await commentStore.createHighlight(

      anchor,

      bookStore.currentBook.id,

      bookStore.currentChapterId

    );

    editor.value = {

      visible: true,

      noteId: note.id,

      quote: note.quote || anchor.quote,

      body: "",

      isNew: true,

    };

  } catch (err) {

    noteError.value = formatAppError(err);

  } finally {

    creatingComment.value = false;

  }

}



function openCommentEditor(commentId) {

  const note = commentStore.chapterComments.find((n) => n.id === commentId);

  if (!note) return;

  editor.value = {

    visible: true,

    noteId: note.id,

    quote: note.quote || "",

    body: note.body || "",

    isNew: !note.body?.trim(),

  };

}



function closeEditor() {

  editor.value = { ...editor.value, visible: false };

}



async function handleSaveComment(body) {

  if (!editor.value.noteId) return;

  noteError.value = "";

  try {

    await commentStore.saveCommentBody(editor.value.noteId, body);

    closeEditor();

  } catch (err) {

    noteError.value = formatAppError(err);

  }

}



function toggleSettings() {

  showSettings.value = !showSettings.value;

}



function toggleCatalog() {

  showCatalog.value = !showCatalog.value;

}



async function handleExport() {

  if (!bookStore.currentBook || exporting.value) return;

  exportMessage.value = "";

  exporting.value = true;

  try {

    const result = await exportBook(bookStore.currentBook.id);

    const skipped = result.total_chapters - result.exported_chapters;

    exportMessage.value =

      `已导出至：${result.path}（${result.exported_chapters}/${result.total_chapters} 章有正文` +

      (skipped > 0 ? "，未读章节已留占位" : "") +

      "）";

  } catch (err) {

    exportMessage.value = `导出失败：${err}`;

  } finally {

    exporting.value = false;

  }

}

</script>



<style scoped>

.reader-view {

  display: flex;

  flex-direction: column;

  height: 100%;

  min-height: 100vh;

  overflow: hidden;

  transition: background 0.25s, color 0.25s;

}



.reader-toolbar {

  flex-shrink: 0;

  z-index: 90;

  display: flex;

  align-items: center;

  gap: 12px;

  padding: 10px 16px;

  border-bottom: 1px solid var(--reader-border, var(--color-border));

  font-size: 0.85rem;

  background: var(--reader-bg, var(--color-surface));

  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);

}



.back-link {

  color: var(--color-primary);

  text-decoration: none;

  white-space: nowrap;

  flex-shrink: 0;

}



.comment-count-link {

  font-size: 0.78rem;

  color: var(--color-primary);

  text-decoration: none;

  white-space: nowrap;

  padding: 4px 8px;

  border-radius: var(--radius-pill);

  background: var(--color-primary-soft);

}



.chapter-title {

  flex: 1;

  overflow: hidden;

  text-overflow: ellipsis;

  white-space: nowrap;

  text-align: center;

  font-weight: 500;

}



.toolbar-actions {

  display: flex;

  align-items: center;

  gap: 6px;

  flex-shrink: 0;

}



.settings-btn,

.icon-btn {

  border: 1px solid var(--reader-border, var(--color-border));

  border-radius: 4px;

  padding: 4px 10px;

  cursor: pointer;

  font-size: 0.85rem;

  background: transparent;

  color: var(--reader-text, var(--color-text));

}

.icon-btn:disabled {

  opacity: 0.5;

  cursor: wait;

}



.reader-error {

  flex-shrink: 0;

  padding: 8px 16px;

  background: var(--color-danger-bg, #fdecea);

  color: var(--color-danger, #c0392b);

  font-size: 0.85rem;

}



.export-msg {

  flex-shrink: 0;

  padding: 6px 16px;

  font-size: 0.75rem;

  color: #27ae60;

  word-break: break-all;

}



.reader-content {

  flex: 1;

  min-height: 0;

  overflow-y: auto;

  padding: 24px 48px;

  touch-action: pan-y;

}



.placeholder {

  text-align: center;

  color: var(--color-muted);

  padding: 48px;

  pointer-events: none;

}

</style>


