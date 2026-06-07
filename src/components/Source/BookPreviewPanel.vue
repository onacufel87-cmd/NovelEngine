<template>
  <BaseModal :visible="visible" wide @close="$emit('close')">
    <div class="book-preview">
      <header class="preview-header">
        <div>
          <h3>{{ book?.title }}</h3>
          <p v-if="book?.author" class="meta">作者：{{ book.author }}</p>
          <p v-if="sourceName" class="meta">书源：{{ sourceName }}</p>
        </div>
        <button type="button" class="close-btn" aria-label="关闭" @click="$emit('close')">
          ×
        </button>
      </header>

      <p v-if="error" class="message error">{{ error }}</p>

      <LoadingSpinner v-if="loadingCatalog" text="正在解析目录…" />

      <template v-else-if="chapters.length">
        <div class="preview-body">
          <!-- 左侧章节目录 -->
          <aside class="chapter-sidebar">
            <p class="sidebar-title">目录（{{ chapters.length }} 章）</p>
            <ul class="chapter-list">
              <li
                v-for="(ch, idx) in chapters"
                :key="idx"
                class="chapter-item"
                :class="{ active: activeIndex === idx }"
                @click="loadChapter(ch, idx)"
              >
                {{ ch.title }}
              </li>
            </ul>
          </aside>

          <!-- 右侧正文试读 -->
          <main class="content-area">
            <p v-if="activeTitle" class="content-title">{{ activeTitle }}</p>
            <LoadingSpinner v-if="loadingContent" text="加载章节正文…" />
            <div v-else class="content-text">{{ previewContent || "点击左侧章节开始试读" }}</div>
          </main>
        </div>

        <footer class="preview-footer">
          <p class="hint">试读满意后再加入书架，不会占用书架空间。</p>
          <div class="actions">
            <button type="button" class="ghost-btn" @click="$emit('close')">关闭</button>
            <BaseButton :disabled="adding" @click="handleAdd">
              {{ adding ? "保存中…" : "加入书架" }}
            </BaseButton>
          </div>
          <p v-if="shelfMessage" class="message ok">{{ shelfMessage }}</p>
          <RouterLink v-if="addedBookId" :to="`/read/${addedBookId}`" class="read-link" @click="$emit('close')">
            前往阅读 →
          </RouterLink>
        </footer>
      </template>
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import { useBookSourceStore } from "../../stores/bookSourceStore";
import { useShelfStore } from "../../stores/shelfStore";
import { fetchChapters, getBookContent } from "../../api";
import BaseModal from "../Common/BaseModal.vue";
import BaseButton from "../Common/BaseButton.vue";
import LoadingSpinner from "../Common/LoadingSpinner.vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
  /** { title, author, catalog_url, source_id } */
  book: { type: Object, default: null },
});

defineEmits(["close"]);

const bookSourceStore = useBookSourceStore();
const shelfStore = useShelfStore();

const chapters = ref([]);
const rule = ref(null);
const loadingCatalog = ref(false);
const loadingContent = ref(false);
const previewContent = ref("");
const activeIndex = ref(-1);
const activeTitle = ref("");
const error = ref("");
const adding = ref(false);
const shelfMessage = ref("");
const addedBookId = ref(null);

const sourceName = computed(() => {
  if (!props.book?.source_id) return "";
  return bookSourceStore.sources.find((s) => s.id === props.book.source_id)?.name ?? "";
});

/** 打开预览时拉取目录并自动试读第一章 */
watch(
  () => [props.visible, props.book?.catalog_url, props.book?.source_id],
  async ([visible]) => {
    if (!visible || !props.book) return;
    await loadCatalog();
  }
);

async function loadCatalog() {
  chapters.value = [];
  previewContent.value = "";
  activeIndex.value = -1;
  activeTitle.value = "";
  error.value = "";
  shelfMessage.value = "";
  addedBookId.value = null;

  const sourceId = props.book.source_id;
  const source = bookSourceStore.sources.find((s) => s.id === sourceId);
  if (!source) {
    error.value = "找不到对应书源，请刷新书源列表后重试";
    return;
  }

  loadingCatalog.value = true;
  try {
    rule.value = JSON.parse(source.rule_json);
    chapters.value = await fetchChapters(props.book.catalog_url, rule.value);
    if (chapters.value.length) {
      await loadChapter(chapters.value[0], 0);
    }
  } catch (err) {
    error.value = String(err);
  } finally {
    loadingCatalog.value = false;
  }
}

/** 加载指定章节正文（试读，不写入书架） */
async function loadChapter(chapter, idx) {
  if (!rule.value) return;

  activeIndex.value = idx;
  activeTitle.value = chapter.title;
  previewContent.value = "";
  loadingContent.value = true;
  error.value = "";

  try {
    const raw = await getBookContent(chapter.url, rule.value);
    // 后端 get_book_content 已含全局清洗与简繁
    previewContent.value = raw;
  } catch (err) {
    previewContent.value = `加载失败: ${err}`;
  } finally {
    loadingContent.value = false;
  }
}

/** 试读满意后加入书架 */
async function handleAdd() {
  if (!props.book || !chapters.value.length) return;

  adding.value = true;
  shelfMessage.value = "";
  addedBookId.value = null;

  try {
    const book = await shelfStore.addBook({
      title: props.book.title,
      catalogUrl: props.book.catalog_url,
      rule: rule.value,
      chapters: chapters.value,
    });
    shelfMessage.value = `已加入书架：《${book.title}》（${book.chapter_count} 章）`;
    addedBookId.value = book.id;
  } catch (err) {
    error.value = String(err);
  } finally {
    adding.value = false;
  }
}
</script>

<style scoped>
.book-preview {
  display: flex;
  flex-direction: column;
  max-height: calc(88vh - 48px);
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
}

.preview-header h3 {
  margin: 0 0 4px;
  font-size: 1.1rem;
  color: var(--color-text);
}

.meta {
  margin: 0;
  font-size: 0.85rem;
  color: var(--color-muted);
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  line-height: 1;
  cursor: pointer;
  color: var(--color-muted);
  padding: 0 4px;
}

.preview-body {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 16px;
  flex: 1;
  min-height: 0;
  margin-bottom: 16px;
}

.chapter-sidebar {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 320px;
}

.sidebar-title {
  margin: 0;
  padding: 10px 12px;
  font-size: 0.85rem;
  background: var(--color-border-light);
  border-bottom: 1px solid var(--color-border);
  color: var(--color-muted);
}

.chapter-list {
  list-style: none;
  overflow-y: auto;
  flex: 1;
  background: var(--color-surface);
}

.chapter-item {
  padding: 8px 12px;
  font-size: 0.85rem;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text);
  transition: background 0.15s;
}

.chapter-item:hover,
.chapter-item.active {
  background: var(--color-primary-soft);
  color: var(--color-primary);
}

.content-area {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  overflow-y: auto;
  min-height: 320px;
  max-height: 420px;
  background: var(--color-surface);
}

.content-title {
  font-weight: 600;
  margin: 0 0 12px;
  font-size: 0.95rem;
  color: var(--color-text);
}

.content-text {
  white-space: pre-wrap;
  line-height: 1.9;
  font-size: 0.95rem;
  text-indent: 2em;
  color: var(--color-text);
}

.preview-footer {
  border-top: 1px solid var(--color-border);
  padding-top: 12px;
}

.preview-footer .hint {
  margin: 0 0 10px;
  font-size: 0.82rem;
  color: var(--color-muted);
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.ghost-btn {
  padding: 8px 16px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  font-size: 0.9rem;
}

.message.error {
  padding: 10px 12px;
  background: var(--color-danger-bg);
  color: var(--color-danger);
  border-radius: 6px;
  font-size: 0.85rem;
  margin-bottom: 12px;
}

.message.ok {
  margin-top: 8px;
  color: var(--color-success);
  font-size: 0.85rem;
}

.read-link {
  display: inline-block;
  margin-top: 8px;
  color: var(--color-primary);
  font-size: 0.9rem;
}

@media (max-width: 720px) {
  .preview-body {
    grid-template-columns: 1fr;
  }

  .chapter-sidebar {
    max-height: 160px;
  }
}
</style>
