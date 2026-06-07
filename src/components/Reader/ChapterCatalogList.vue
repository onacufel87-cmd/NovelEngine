<template>
  <ul ref="listRef" class="catalog-list">
    <li
      v-for="(ch, idx) in bookStore.chapters"
      :key="ch.id"
      class="catalog-item"
      :class="{ active: ch.id === bookStore.currentChapterId }"
      :data-chapter-id="ch.id"
      @click="selectChapter(ch.id)"
    >
      <span class="idx">{{ idx + 1 }}</span>
      <span class="title">{{ ch.title }}</span>
    </li>
  </ul>
</template>

<script setup>
import { ref, watch, nextTick } from "vue";
import { useBookStore } from "../../stores/bookStore";

const emit = defineEmits(["select"]);

const bookStore = useBookStore();
const listRef = ref(null);

/** 选中章节并通知父级（侧栏不关闭，浮层需关闭） */
function selectChapter(chapterId) {
  bookStore.loadChapter(chapterId);
  emit("select", chapterId);
}

/** 当前章变化时滚动到可见区域 */
watch(
  () => bookStore.currentChapterId,
  async () => {
    await nextTick();
    const el = listRef.value?.querySelector(".catalog-item.active");
    el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  },
  { immediate: true }
);
</script>

<style scoped>
.catalog-list {
  list-style: none;
  overflow-y: auto;
  flex: 1;
  padding: 8px 0;
  margin: 0;
}

.catalog-item {
  display: flex;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.catalog-item:hover,
.catalog-item.active {
  background: var(--color-primary-soft, rgba(212, 163, 115, 0.12));
}

.catalog-item.active .title {
  color: var(--color-primary);
  font-weight: 600;
}

.idx {
  color: var(--color-muted);
  min-width: 28px;
  font-size: 0.85rem;
  flex-shrink: 0;
}

.title {
  flex: 1;
  font-size: 0.9rem;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
