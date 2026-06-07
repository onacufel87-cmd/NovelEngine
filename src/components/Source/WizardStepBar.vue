<template>
  <nav class="wizard-steps" aria-label="接入步骤">
    <div
      v-for="(label, idx) in steps"
      :key="label"
      class="wizard-step"
      :class="{
        'wizard-step--done': idx < step,
        'wizard-step--active': idx === step,
      }"
    >
      <span class="wizard-step__dot">{{ idx < step ? "✓" : idx + 1 }}</span>
      <span class="wizard-step__label">{{ label }}</span>
    </div>
  </nav>
</template>

<script setup>
defineProps({
  /** 当前步骤 0～2 */
  step: { type: Number, default: 0 },
});

const steps = ["检测规则", "预览确认", "目录试读"];
</script>

<style scoped>
.wizard-steps {
  display: flex;
  gap: 0;
  padding: 12px 16px;
  background: var(--color-bg);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
}

.wizard-step {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-muted);
  font-size: 0.82rem;
  position: relative;
}

.wizard-step:not(:last-child)::after {
  content: "";
  flex: 1;
  height: 1px;
  margin: 0 10px;
  background: var(--color-border-light);
  min-width: 12px;
}

.wizard-step--active {
  color: var(--color-primary);
  font-weight: 600;
}

.wizard-step--done {
  color: var(--color-text);
}

.wizard-step__dot {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.72rem;
  flex-shrink: 0;
  background: var(--color-surface);
}

.wizard-step--active .wizard-step__dot {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary);
}

.wizard-step--done .wizard-step__dot {
  border-color: var(--color-success);
  background: var(--color-success-bg);
  color: var(--color-success);
}
</style>
