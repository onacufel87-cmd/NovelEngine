<template>
  <div
    v-if="visible"
    class="note-selection-bar"
    :style="barStyle"
    @mousedown.prevent
  >
    <button
      type="button"
      class="bar-btn bar-btn--primary"
      :disabled="busy"
      @click="$emit('comment')"
    >
      {{ busy ? "…" : "评论" }}
    </button>
  </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
  rect: { type: Object, default: null },
  /** 提交中禁用按钮，防止连击重复创建 */
  busy: { type: Boolean, default: false },
});

defineEmits(["comment"]);

const barStyle = computed(() => {
  if (!props.rect) return { display: "none" };
  const top = Math.max(8, props.rect.top - 44);
  const left = Math.max(8, props.rect.left + props.rect.width / 2);
  return {
    top: `${top}px`,
    left: `${left}px`,
    transform: "translateX(-50%)",
  };
});
</script>

<style scoped>
.note-selection-bar {
  position: fixed;
  z-index: 120;
  display: flex;
  gap: 6px;
  padding: 4px;
  background: var(--reader-bg, var(--color-surface));
  border: 1px solid var(--reader-border, var(--color-border));
  border-radius: var(--radius-pill, 20px);
  box-shadow: var(--shadow-md, 0 8px 20px rgba(0, 0, 0, 0.12));
}

.bar-btn {
  border: none;
  border-radius: var(--radius-pill, 20px);
  padding: 6px 18px;
  font-size: 0.85rem;
  cursor: pointer;
  background: transparent;
  color: var(--reader-text, var(--color-text));
}

.bar-btn--primary {
  background: var(--color-primary);
  color: #fff;
  font-weight: 500;
}

.bar-btn--primary:hover:not(:disabled) {
  filter: brightness(1.05);
}

.bar-btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
</style>
