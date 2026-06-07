<template>
  <div v-if="show" class="loading-spinner">
    <div class="spinner" />
    <p v-if="text">{{ text }}</p>
  </div>
</template>

<script setup>
defineProps({
  show: { type: Boolean, default: true },
  text: { type: String, default: "加载中…" },
});
</script>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: #888;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
