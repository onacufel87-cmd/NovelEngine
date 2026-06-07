<template>
  <div v-if="visible" class="modal-overlay" @click.self="close">
    <div class="modal-content" :class="{ wide }">
      <slot />
    </div>
  </div>
</template>

<script setup>
defineProps({
  visible: { type: Boolean, default: false },
  /** 宽屏模式，用于试读预览 */
  wide: { type: Boolean, default: false },
});

const emit = defineEmits(["close"]);

function close() {
  emit("close");
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--color-surface);
  border-radius: var(--radius-modal);
  padding: 24px;
  min-width: 320px;
  max-width: 90vw;
  box-shadow: var(--shadow-md);
}

.modal-content.wide {
  width: min(960px, 95vw);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
