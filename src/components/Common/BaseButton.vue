<template>
  <button
    class="base-button"
    :type="type"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<script setup>
defineProps({
  disabled: { type: Boolean, default: false },
  type: { type: String, default: "button" },
});

defineEmits(["click"]);
</script>

<style scoped>
.base-button {
  padding: 9px 20px;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--color-primary);
  color: #fff;
  cursor: pointer;
  font-size: 0.88rem;
  font-weight: 500;
  transition: background 0.15s, transform 0.1s, box-shadow 0.15s;
  box-shadow: var(--shadow-sm);
}

.base-button:hover:not(:disabled) {
  background: var(--color-primary-hover);
  box-shadow: var(--shadow-md);
}

.base-button:active:not(:disabled) {
  transform: scale(0.98);
}

.base-button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

@media (prefers-reduced-motion: reduce) {
  .base-button:active:not(:disabled) {
    transform: none;
  }
}
</style>
