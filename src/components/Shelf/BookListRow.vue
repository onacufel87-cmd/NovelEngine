<template>
  <article class="book-list-row">
    <RouterLink :to="`/read/${book.id}`" class="row-link">
      <div class="row-cover">
        <span class="cover-letter">{{ book.title?.[0] ?? "?" }}</span>
      </div>
      <div class="row-info">
        <h3 class="title">{{ book.title ?? "未知书名" }}</h3>
        <p class="meta">
          <span v-if="book.author">{{ book.author }}</span>
          <span v-if="book.author && book.source_rule_name" class="dot">·</span>
          <span v-if="book.source_rule_name">{{ book.source_rule_name }}</span>
        </p>
        <p class="sub-meta">
          <span>{{ book.chapter_count ?? 0 }} 章</span>
          <span v-if="book.last_chapter_id" class="progress">继续阅读</span>
        </p>
      </div>
    </RouterLink>
    <button type="button" class="delete-btn" title="移出书架" @click.stop="handleDelete">
      ×
    </button>
  </article>
</template>

<script setup>
import { useShelfStore } from "../../stores/shelfStore";

const props = defineProps({
  book: { type: Object, required: true },
});

const shelfStore = useShelfStore();

async function handleDelete() {
  if (!confirm(`确定将《${props.book.title}》移出书架？`)) return;
  await shelfStore.removeBook(props.book.id);
}
</script>

<style scoped>
.book-list-row {
  position: relative;
  display: flex;
  align-items: stretch;
  background: var(--color-surface);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  transition: box-shadow 0.15s var(--ease-out);
}

.book-list-row:hover {
  box-shadow: var(--shadow-sm);
}

.row-link {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  padding: 12px 14px;
  text-decoration: none;
  color: inherit;
  min-width: 0;
}

.row-cover {
  width: 44px;
  height: 58px;
  flex-shrink: 0;
  border-radius: var(--radius-sm);
  background: linear-gradient(160deg, var(--color-primary-soft) 0%, #f5ebe0 100%);
  border: 1px solid var(--color-border-light);
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-letter {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-primary);
}

.row-info {
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 0.96rem;
  font-weight: 600;
  line-height: 1.35;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta {
  font-size: 0.8rem;
  color: var(--color-muted);
  margin-top: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sub-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.75rem;
  color: var(--color-muted);
  margin-top: 4px;
}

.progress {
  color: var(--color-primary);
  font-weight: 500;
}

.dot {
  opacity: 0.5;
}

.delete-btn {
  align-self: center;
  margin-right: 12px;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: var(--color-surface);
  color: var(--color-muted);
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  opacity: 0;
  box-shadow: var(--shadow-sm);
  transition: opacity 0.15s, background 0.15s;
  flex-shrink: 0;
}

.book-list-row:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: var(--color-danger-bg);
  color: var(--color-danger);
}
</style>
