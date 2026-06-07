<template>
  <footer class="reader-bottom-bar">
    <button class="bar-btn" :disabled="!bookStore.hasPrev" @click="bookStore.loadPrevChapter">
      上一章
    </button>
    <button class="bar-btn bar-btn--center" @click="$emit('open-catalog')">
      目录
    </button>
    <button class="bar-btn" :disabled="!bookStore.hasNext" @click="bookStore.loadNextChapter">
      下一章
    </button>
  </footer>
</template>

<script setup>
import { useBookStore } from "../../stores/bookStore";

defineEmits(["open-catalog"]);

const bookStore = useBookStore();
</script>

<style scoped>
.reader-bottom-bar {
  display: flex;
  align-items: stretch;
  border-top: 1px solid var(--reader-border, var(--color-border));
  background: var(--reader-bg, var(--color-surface));
  flex-shrink: 0;
}

.bar-btn {
  flex: 1;
  padding: 14px 8px;
  border: none;
  background: transparent;
  color: var(--reader-text, var(--color-text));
  font-size: 0.95rem;
  cursor: pointer;
  transition: background 0.15s;
}

.bar-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.05);
}

.bar-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.bar-btn--center {
  flex: 1.2;
  font-weight: 600;
  border-left: 1px solid var(--reader-border, var(--color-border));
  border-right: 1px solid var(--reader-border, var(--color-border));
}
</style>
