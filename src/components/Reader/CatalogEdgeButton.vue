<template>
  <button
    type="button"
    class="catalog-tile"
    :class="{ 'catalog-tile--active': active }"
    :title="active ? '收起目录' : '打开目录'"
    :aria-expanded="active"
    @click.stop="$emit('click')"
  >
    <svg
      class="catalog-tile__icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      aria-hidden="true"
    >
      <line x1="9" y1="6" x2="20" y2="6" />
      <line x1="9" y1="12" x2="20" y2="12" />
      <line x1="9" y1="18" x2="20" y2="18" />
      <circle cx="5" cy="6" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="5" cy="12" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="5" cy="18" r="1.2" fill="currentColor" stroke="none" />
    </svg>
    <span class="catalog-tile__label">目录</span>
  </button>
</template>

<script setup>
defineProps({
  /** 目录面板是否已展开 */
  active: { type: Boolean, default: false },
});

defineEmits(["click"]);
</script>

<style scoped>
/* 右侧常驻浮钮，点击切换目录弹出/收起 */
.catalog-tile {
  position: fixed;
  top: 50%;
  right: 14px;
  transform: translateY(-50%);
  z-index: 220;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 52px;
  height: 56px;
  padding: 8px 4px;
  border: none;
  border-radius: 12px;
  background: var(--catalog-tile-bg, var(--color-surface, #faf8f5));
  color: var(--color-primary);
  cursor: pointer;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  transition:
    box-shadow 0.15s,
    transform 0.15s,
    background 0.15s,
    color 0.15s;
}

.catalog-tile:hover {
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.14);
  transform: translateY(-50%) scale(1.03);
}

.catalog-tile:active {
  transform: translateY(-50%) scale(0.98);
}

.catalog-tile--active {
  background: var(--color-primary-soft);
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.12);
}

.catalog-tile__icon {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
}

.catalog-tile__label {
  font-size: 0.72rem;
  font-weight: 600;
  line-height: 1;
  letter-spacing: 0.04em;
}

@media (prefers-reduced-motion: reduce) {
  .catalog-tile:hover,
  .catalog-tile:active {
    transform: translateY(-50%);
  }
}
</style>
