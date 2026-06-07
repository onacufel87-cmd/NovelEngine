<template>
  <section class="rank-panel page-stack">
    <div class="card controls">
      <label class="form-field">
        <span>书源</span>
        <select v-model="bookSourceStore.selectedSourceId" @change="onSourceChange">
          <option v-for="src in bookSourceStore.enabledSources" :key="src.id" :value="src.id">
            {{ src.name }}
          </option>
        </select>
      </label>

      <label v-if="bookSourceStore.rankTypes.length" class="form-field">
        <span>榜单类型</span>
        <select
          v-model="bookSourceStore.selectedRankType"
          @change="bookSourceStore.loadRank(bookSourceStore.selectedRankType)"
        >
          <option v-for="t in bookSourceStore.rankTypes" :key="t" :value="t">
            {{ t }}
          </option>
        </select>
      </label>
    </div>

    <p v-if="bookSourceStore.error" class="msg msg--error">{{ bookSourceStore.error }}</p>

    <LoadingSpinner v-if="bookSourceStore.loadingRank" text="加载榜单…" />

    <div v-else-if="bookSourceStore.rankBooks.length" class="card">
      <h3 class="card-title">
        {{ bookSourceStore.selectedRankType || "榜单" }} · {{ bookSourceStore.rankBooks.length }}
      </h3>
      <ul class="list-rows">
        <li v-for="(book, idx) in bookSourceStore.rankBooks" :key="idx" class="list-row">
          <div class="list-row__main">
            <strong>{{ book.title }}</strong>
            <span v-if="book.author" class="sub">{{ book.author }}</span>
          </div>
          <div class="list-row__actions">
            <button type="button" class="btn-ghost" @click="openPreview(book)">试读</button>
            <BaseButton :disabled="addingId === idx" @click="handleAdd(book, idx)">
              {{ addingId === idx ? "解析中…" : "加入书架" }}
            </BaseButton>
          </div>
        </li>
      </ul>
      <p v-if="shelfMessage" class="msg msg--ok" style="margin-top: 12px">{{ shelfMessage }}</p>
      <RouterLink v-if="addedBookId" :to="`/read/${addedBookId}`" class="read-link">
        前往阅读 →
      </RouterLink>
    </div>

    <p
      v-else-if="bookSourceStore.selectedSourceId && !bookSourceStore.loadingRank"
      class="empty-state card"
    >
      该书源未配置榜单，或暂无数据。
    </p>

    <BookPreviewPanel :visible="previewVisible" :book="previewBook" @close="closePreview" />
  </section>
</template>

<script setup>
import { onMounted, ref } from "vue";
import { useBookSourceStore } from "../../stores/bookSourceStore";
import { useShelfStore } from "../../stores/shelfStore";
import { formatAppError } from "../../utils/appError";
import BookPreviewPanel from "../Source/BookPreviewPanel.vue";
import BaseButton from "../Common/BaseButton.vue";
import LoadingSpinner from "../Common/LoadingSpinner.vue";

const bookSourceStore = useBookSourceStore();
const shelfStore = useShelfStore();
const addingId = ref(null);
const previewVisible = ref(false);
const previewBook = ref(null);
const shelfMessage = ref("");
const addedBookId = ref(null);

onMounted(async () => {
  await bookSourceStore.loadSources();
  const first = bookSourceStore.enabledSources[0];
  if (first) {
    await bookSourceStore.loadRankMeta(first.id);
  }
});

async function onSourceChange() {
  shelfMessage.value = "";
  addedBookId.value = null;
  closePreview();
  await bookSourceStore.loadRankMeta(bookSourceStore.selectedSourceId);
}

function openPreview(book) {
  previewBook.value = {
    ...book,
    source_id: bookSourceStore.selectedSourceId,
  };
  previewVisible.value = true;
}

function closePreview() {
  previewVisible.value = false;
  previewBook.value = null;
}

async function handleAdd(book, idx) {
  addingId.value = idx;
  shelfMessage.value = "";
  addedBookId.value = null;

  try {
    const result = await shelfStore.addBookFromSearchResult({
      title: book.title,
      author: book.author,
      catalogUrl: book.catalog_url,
      sourceId: bookSourceStore.selectedSourceId,
    });
    shelfMessage.value = `已加入书架：《${result.title}》（${result.chapter_count} 章）`;
    addedBookId.value = result.id;
  } catch (err) {
    bookSourceStore.error = formatAppError(err);
  } finally {
    addingId.value = null;
  }
}
</script>

<style scoped>
.controls {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.controls .form-field {
  min-width: 180px;
  flex: 1;
}

.sub {
  color: var(--color-muted);
  font-size: 0.85rem;
}

.read-link {
  display: inline-block;
  margin-top: 10px;
  font-size: 0.88rem;
}
</style>
