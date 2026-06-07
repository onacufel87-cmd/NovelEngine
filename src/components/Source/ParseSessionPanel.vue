<template>
  <section class="parse-session page-stack">
    <!-- 手动 JSON：折叠保留 -->
    <details v-if="showJsonPanel" class="card card-collapse">
      <summary>高级 · 手动 JSON 书源规则</summary>
      <div class="card-collapse-body">
        <div class="form-actions">
          <label class="btn-file">
            上传 JSON 文件
            <input type="file" accept=".json" hidden @change="onRuleFileChange" />
          </label>
          <button v-if="isDev" type="button" class="btn-link" @click="loadTestRule">加载本地测试书源</button>
        </div>

        <JsonRuleEditor
          v-model="ruleJsonText"
          class="rule-editor"
          min-height="200px"
          @valid="onRuleValid"
        />

        <div class="form-actions" style="margin-top: 10px">
          <BaseButton :disabled="!ruleJsonText.trim()" @click="loadRuleFromEditor">
            加载规则
          </BaseButton>
        </div>

        <div v-if="store.hasParseRule" class="rule-info">
          <span class="badge badge--ok">已加载</span>
          <strong>{{ store.parseRuleName }}</strong>
          <span class="meta">{{ store.parseSession.rule.chapter_list_selector }}</span>
        </div>
        <p v-if="store.parseSession.statusMessage" class="msg msg--ok" style="margin-top: 10px">
          {{ store.parseSession.statusMessage }}
        </p>
      </div>
    </details>

    <!-- 目录解析 -->
    <div class="card">
      <h3 class="card-title">解析目录并试读</h3>
      <form class="form-row" @submit.prevent="handleParseCatalog">
        <label class="form-field">
          <span>目录页 URL</span>
          <input
            v-model="catalogUrl"
            type="url"
            required
            placeholder="https://example.com/book/123/"
          />
        </label>
        <div class="form-actions">
          <BaseButton type="submit" :disabled="!store.hasParseRule || store.parseSession.loading">
            {{ store.parseSession.loading ? "解析中…" : "解析目录" }}
          </BaseButton>
          <button v-if="isDev" type="button" class="btn-link" @click="applyTestCatalogDefaults">本地测试目录</button>
        </div>
      </form>
    </div>

    <p v-if="store.parseSession.error" class="msg msg--error">{{ store.parseSession.error }}</p>

    <div v-if="store.parseSession.chapters.length" class="card">
      <h3 class="card-title">章节列表 · {{ store.parseSession.chapters.length }}</h3>
      <ul class="chapter-list">
        <li
          v-for="(ch, idx) in store.parseSession.chapters"
          :key="idx"
          class="chapter-item"
          :class="{ active: previewUrl === ch.url }"
          @click="previewChapter(ch)"
        >
          <span class="idx">{{ idx + 1 }}</span>
          <span class="title">{{ ch.title }}</span>
        </li>
      </ul>

      <div class="add-shelf">
        <h4 class="card-section-title">加入书架</h4>
        <label class="form-field">
          <span>书名</span>
          <input v-model="bookTitle" type="text" placeholder="书名" />
        </label>
        <BaseButton :disabled="addingToShelf" @click="handleAddToShelf">
          {{ addingToShelf ? "保存中…" : "加入书架" }}
        </BaseButton>
        <p v-if="shelfMessage" class="msg msg--ok" style="margin-top: 10px">{{ shelfMessage }}</p>
        <RouterLink v-if="addedBookId" :to="`/read/${addedBookId}`" class="read-link">
          前往阅读 →
        </RouterLink>
      </div>
    </div>

    <div v-if="previewContent || previewLoading" class="card preview">
      <h3 class="card-title">章节预览</h3>
      <p v-if="previewTitle" class="preview-title">{{ previewTitle }}</p>
      <LoadingSpinner v-if="previewLoading" text="加载章节正文…" />
      <div v-else class="preview-body">{{ previewContent }}</div>
    </div>
  </section>
</template>

<script setup>
import { ref, watch } from "vue";
import { useBookSourceStore } from "../../stores/bookSourceStore";
import { useShelfStore } from "../../stores/shelfStore";
import { getBookContent } from "../../api";
import BaseButton from "../Common/BaseButton.vue";
import LoadingSpinner from "../Common/LoadingSpinner.vue";
import JsonRuleEditor from "./JsonRuleEditor.vue";

const props = defineProps({
  showJsonPanel: { type: Boolean, default: true },
  initialCatalogUrl: { type: String, default: "" },
});

const emit = defineEmits(["catalog-url"]);

const store = useBookSourceStore();
const shelfStore = useShelfStore();
const catalogUrl = ref("");
const bookTitle = ref("测试小说");
const previewUrl = ref("");
const previewTitle = ref("");
const previewContent = ref("");
const previewLoading = ref(false);
const addingToShelf = ref(false);
const shelfMessage = ref("");
const addedBookId = ref(null);
const ruleJsonText = ref("");
const isDev = import.meta.env.DEV;
function setCatalogUrl(url) {
  if (url) {
    catalogUrl.value = url;
    emit("catalog-url", url);
  }
}

defineExpose({ setCatalogUrl });

watch(
  () => props.initialCatalogUrl,
  (url) => {
    if (url) catalogUrl.value = url;
  },
  { immediate: true }
);

async function onRuleFileChange(event) {
  const file = event.target.files?.[0];
  if (!file) return;
  try {
    ruleJsonText.value = await file.text();
  } catch (err) {
    store.setParseError(`读取文件失败: ${err}`);
  }
  event.target.value = "";
}

async function loadRuleFromEditor() {
  if (!ruleJsonText.value.trim()) return;
  try {
    const rule = JSON.parse(ruleJsonText.value);
    await store.loadParseRule(rule);
  } catch (err) {
    store.setParseError(`书源 JSON 无效: ${err}`);
  }
}

/** 校验通过后自动加载规则 */
async function onRuleValid({ rule }) {
  await store.loadParseRule(rule);
}

async function loadTestRule() {
  try {
    await store.loadTestParseRule();
    if (store.parseSession.rule) {
      ruleJsonText.value = JSON.stringify(store.parseSession.rule, null, 2);
    }
    applyTestCatalogDefaults();
  } catch (err) {
    store.setParseError(String(err));
  }
}

function applyTestCatalogDefaults() {
  catalogUrl.value = `${window.location.origin}/test-catalog.html`;
  bookTitle.value = "测试小说";
}

async function handleParseCatalog() {
  previewContent.value = "";
  previewTitle.value = "";
  previewUrl.value = "";
  await store.parseCatalog(catalogUrl.value);
}

async function handleAddToShelf() {
  if (!store.parseSession.rule || !store.parseSession.chapters.length) return;
  addingToShelf.value = true;
  shelfMessage.value = "";
  addedBookId.value = null;
  try {
    const book = await shelfStore.addBook({
      title: bookTitle.value.trim() || "未命名书籍",
      catalogUrl: store.parseSession.catalogUrl || catalogUrl.value,
      rule: store.parseSession.rule,
      chapters: store.parseSession.chapters,
    });
    shelfMessage.value = `已加入书架：《${book.title}》（${book.chapter_count} 章）`;
    addedBookId.value = book.id;
  } catch (err) {
    store.setParseError(String(err));
  } finally {
    addingToShelf.value = false;
  }
}

async function previewChapter(chapter) {
  if (!store.parseSession.rule) return;
  previewUrl.value = chapter.url;
  previewTitle.value = chapter.title;
  previewContent.value = "";
  previewLoading.value = true;
  try {
    previewContent.value = await getBookContent(chapter.url, store.parseSession.rule);
  } catch (err) {
    previewContent.value = `加载失败: ${err}`;
  } finally {
    previewLoading.value = false;
  }
}
</script>

<style scoped>
.rule-editor {
  margin-top: 12px;
}

.rule-info {
  margin-top: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 0.88rem;
}

.meta {
  color: var(--color-muted);
  font-size: 0.8rem;
}

.chapter-list {
  list-style: none;
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
}

.chapter-item {
  display: flex;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border-light);
  cursor: pointer;
  transition: background 0.12s;
}

.chapter-item:last-child {
  border-bottom: none;
}

.chapter-item:hover,
.chapter-item.active {
  background: var(--color-primary-soft);
}

.chapter-item .idx {
  color: var(--color-muted);
  min-width: 22px;
  font-size: 0.82rem;
}

.chapter-item .title {
  font-size: 0.88rem;
}

.preview-body {
  white-space: pre-wrap;
  line-height: 1.9;
  max-height: 360px;
  overflow-y: auto;
  text-indent: 2em;
  font-size: 0.92rem;
  color: var(--color-text);
}

.preview-title {
  font-size: 0.88rem;
  color: var(--color-muted);
  margin-bottom: 12px;
}

.add-shelf {
  margin-top: 18px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border-light);
}

.read-link {
  display: inline-block;
  margin-top: 10px;
  font-size: 0.88rem;
}
</style>
