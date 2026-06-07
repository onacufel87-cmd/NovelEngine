<template>
  <Teleport to="body">
    <Transition name="catalog-fade">
      <div
        v-if="visible"
        class="catalog-overlay"
        :class="{ 'catalog-overlay--drawer': isDrawer }"
        @click.self="$emit('close')"
      >
        <aside
          class="catalog-panel"
          :class="{ 'catalog-panel--drawer': isDrawer }"
          role="dialog"
          aria-label="章节目录"
        >
          <header class="catalog-header">
            <div class="catalog-header__title">
              <h3>目录</h3>
              <span v-if="bookStore.chapters.length" class="chapter-count">
                共 {{ bookStore.chapters.length }} 章
              </span>
            </div>
            <button
              v-if="!isDrawer"
              type="button"
              class="close-btn"
              title="关闭"
              @click="$emit('close')"
            >
              ×
            </button>
          </header>
          <ChapterCatalogList @select="$emit('close')" />
        </aside>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { onMounted, onUnmounted, computed } from "vue";
import { useBookStore } from "../../stores/bookStore";
import { useBreakpoint } from "../../hooks/useBreakpoint";
import ChapterCatalogList from "./ChapterCatalogList.vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
});

const emit = defineEmits(["close"]);

const bookStore = useBookStore();
const isDesktop = useBreakpoint();

/** 桌面端右侧抽屉，移动端底部弹出 */
const isDrawer = computed(() => isDesktop.value);

function onKeydown(event) {
  if (!props.visible || event.key !== "Escape") return;
  emit("close");
}

onMounted(() => {
  document.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown);
});
</script>

<style scoped>
.catalog-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

/* 桌面：从右侧滑出，不挤压正文 */
.catalog-overlay--drawer {
  align-items: stretch;
  justify-content: flex-end;
  background: rgba(0, 0, 0, 0.2);
}

.catalog-panel {
  position: relative;
  width: 100%;
  max-width: 480px;
  max-height: 70vh;
  background: var(--reader-bg, var(--color-surface));
  color: var(--reader-text, var(--color-text));
  border-radius: 16px 16px 0 0;
  display: flex;
  flex-direction: column;
  animation: slide-up 0.25s ease;
  box-shadow: var(--shadow-md, 0 8px 24px rgba(0, 0, 0, 0.15));
}

.catalog-panel--drawer {
  width: min(360px, calc(100vw - 72px));
  max-width: none;
  max-height: none;
  height: 100%;
  border-radius: 0;
  border-left: 1px solid var(--reader-border, var(--color-border));
  animation: slide-in-right 0.25s ease;
  /* 为右侧常驻目录钮留出空间 */
  margin-right: 0;
}

@keyframes slide-up {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}

@keyframes slide-in-right {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

.catalog-fade-enter-active,
.catalog-fade-leave-active {
  transition: opacity 0.2s ease;
}

.catalog-fade-enter-from,
.catalog-fade-leave-to {
  opacity: 0;
}

.catalog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--reader-border, var(--color-border));
  flex-shrink: 0;
}

.catalog-header__title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.catalog-header h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin: 0;
}

.chapter-count {
  font-size: 0.75rem;
  color: var(--color-muted);
  white-space: nowrap;
}

.close-btn {
  border: none;
  background: none;
  font-size: 1.4rem;
  cursor: pointer;
  color: var(--color-muted);
  line-height: 1;
  flex-shrink: 0;
  padding: 4px;
}

.close-btn:hover {
  color: var(--reader-text, var(--color-text));
}

@media (prefers-reduced-motion: reduce) {
  .catalog-panel,
  .catalog-panel--drawer {
    animation: none;
  }

  .catalog-fade-enter-active,
  .catalog-fade-leave-active {
    transition: none;
  }
}
</style>
