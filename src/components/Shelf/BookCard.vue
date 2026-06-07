<template>

  <article class="book-card">

    <RouterLink :to="`/read/${book.id}`" class="card-link">

      <div class="cover">
        <!-- 封面仅显示书名首字，避免与图标叠在一起 -->
        <span class="cover-letter">{{ book.title?.[0] ?? "?" }}</span>
      </div>

      <h3 class="title">{{ book.title ?? "未知书名" }}</h3>

      <p class="meta">

        <span>{{ book.chapter_count ?? 0 }} 章</span>

        <span v-if="book.source_rule_name" class="dot">·</span>

        <span v-if="book.source_rule_name" class="source">{{ book.source_rule_name }}</span>

      </p>

    </RouterLink>

    <button class="delete-btn" title="移出书架" @click.stop="handleDelete">×</button>

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

.book-card {
  position: relative;
  width: 100%;
  box-sizing: border-box;
  background: var(--color-surface);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  padding: 10px;
  isolation: isolate;
  transition: box-shadow 0.15s var(--ease-out), border-color 0.15s var(--ease-out);
}

.book-card:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--color-border);
}



@media (prefers-reduced-motion: reduce) {
  .book-card:hover {
    box-shadow: var(--shadow-sm);
  }
}



.card-link {

  display: block;

  text-decoration: none;

  color: inherit;

}



.cover {
  width: 100%;
  aspect-ratio: 3 / 4;
  background: linear-gradient(160deg, var(--color-primary-soft) 0%, #f5ebe0 100%);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  border: 1px solid var(--color-border-light);
  overflow: hidden;
}

.cover-letter {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-primary);
  opacity: 0.88;
  line-height: 1;
  user-select: none;
}

.title {
  font-size: 0.82rem;
  font-weight: 600;
  line-height: 1.35;
  min-height: 1.35em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta {
  font-size: 0.68rem;
  color: var(--color-muted);
  margin-top: 4px;
  min-height: 1.35em;
  display: flex;
  align-items: center;
  gap: 3px;
  overflow: hidden;
}



.source {

  overflow: hidden;

  text-overflow: ellipsis;

  white-space: nowrap;

}



.dot {

  opacity: 0.5;

}



.delete-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 22px;
  height: 22px;

  border: none;

  border-radius: 50%;

  background: var(--color-surface);

  color: var(--color-muted);

  cursor: pointer;

  font-size: 0.85rem;

  line-height: 1;

  opacity: 0;

  box-shadow: var(--shadow-sm);

  transition: opacity 0.15s, background 0.15s;

}



.book-card:hover .delete-btn {

  opacity: 1;

}



.delete-btn:hover {

  background: var(--color-danger-bg);

  color: var(--color-danger);

}

</style>

