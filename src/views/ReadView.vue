<template>
  <section class="read-view">
    <LoadingSpinner v-if="bookStore.loading && !bookStore.currentBook" text="打开书籍中…" />
    <ReaderView v-else-if="bookStore.currentBook" />
    <div v-else class="placeholder">
      <p>{{ bookStore.error || "请从书架选择一本书开始阅读" }}</p>
      <RouterLink to="/">返回书架</RouterLink>
    </div>
  </section>
</template>

<script setup>
import { watch, onMounted } from "vue";
import { useRoute } from "vue-router";
import { useBookStore } from "../stores/bookStore";
import ReaderView from "../components/Reader/ReaderView.vue";
import LoadingSpinner from "../components/Common/LoadingSpinner.vue";

const route = useRoute();
const bookStore = useBookStore();

async function openFromRoute() {
  const bookId = route.params.bookId;
  if (!bookId) return;

  await bookStore.openBook(Number(bookId));

  // 笔记页跳转：指定章节
  const chapterId = route.query.chapter;
  if (chapterId && Number(chapterId) !== bookStore.currentChapterId) {
    await bookStore.loadChapter(Number(chapterId));
  }
}

onMounted(openFromRoute);

watch(() => [route.params.bookId, route.query.chapter], openFromRoute);
</script>

<style scoped>
.read-view {
  height: 100vh;
  overflow: hidden;
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--color-muted);
}
</style>
