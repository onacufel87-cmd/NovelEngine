<template>
  <Teleport to="body">
    <Transition name="comment-fade">
      <div
        v-if="visible"
        class="comment-overlay"
        :class="{ 'comment-overlay--drawer': layout === 'drawer' }"
        @click.self="$emit('close')"
      >
        <aside
          class="comment-panel"
          :class="{ 'comment-panel--drawer': layout === 'drawer' }"
          role="dialog"
          aria-label="评论编辑"
        >
          <header class="panel-header">
            <h3>{{ isNew ? "发表评论" : "编辑" }}</h3>
            <button type="button" class="close-btn" title="关闭" @click="$emit('close')">
              ×
            </button>
          </header>

          <div class="panel-body">
            <blockquote v-if="quote" class="quote-preview">{{ quote }}</blockquote>
            <textarea
              ref="inputRef"
              v-model="draft"
              class="comment-input"
              placeholder="写下你的想法…"
              @keydown.ctrl.enter="save"
              @keydown.meta.enter="save"
            />
          </div>

          <footer class="panel-footer">
            <span class="hint">Ctrl + Enter 保存</span>
            <div class="actions">
              <button type="button" class="btn-ghost" @click="$emit('close')">取消</button>
              <button type="button" class="btn-primary" :disabled="saving" @click="save">
                {{ saving ? "保存中…" : "保存" }}
              </button>
            </div>
          </footer>
        </aside>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch, nextTick } from "vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
  quote: { type: String, default: "" },
  initialBody: { type: String, default: "" },
  isNew: { type: Boolean, default: true },
  /** drawer=阅读侧栏；modal=评论管理页居中弹窗 */
  layout: {
    type: String,
    default: "drawer",
    validator: (v) => ["drawer", "modal"].includes(v),
  },
});

const emit = defineEmits(["close", "save"]);

const draft = ref("");
const saving = ref(false);
const inputRef = ref(null);

watch(
  () => props.visible,
  async (open) => {
    if (open) {
      draft.value = props.initialBody || "";
      await nextTick();
      inputRef.value?.focus();
    }
  }
);

async function save() {
  saving.value = true;
  try {
    emit("save", draft.value.trim());
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.comment-overlay {
  position: fixed;
  inset: 0;
  z-index: 240;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.comment-overlay--drawer {
  align-items: stretch;
  justify-content: flex-end;
  background: rgba(0, 0, 0, 0.22);
}

/* modal 布局：居中弹窗 */
.comment-overlay:not(.comment-overlay--drawer) {
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.comment-panel {
  width: 100%;
  max-width: 480px;
  max-height: 75vh;
  background: var(--reader-bg, var(--color-surface));
  color: var(--reader-text, var(--color-text));
  border-radius: 16px 16px 0 0;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-md);
  animation: slide-up 0.25s ease;
}

.comment-overlay:not(.comment-overlay--drawer) .comment-panel {
  max-width: 420px;
  max-height: min(85vh, 640px);
  border-radius: var(--radius-lg, 16px);
  animation: none;
}

.comment-panel--drawer {
  width: min(340px, 92vw);
  max-width: none;
  max-height: none;
  height: 100%;
  border-radius: 0;
  border-left: 1px solid var(--reader-border, var(--color-border));
  animation: slide-in-right 0.25s ease;
}

@keyframes slide-up {
  from { transform: translateY(100%); }
  to { transform: translateY(0); }
}

@keyframes slide-in-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.comment-fade-enter-active,
.comment-fade-leave-active {
  transition: opacity 0.2s ease;
}

.comment-fade-enter-from,
.comment-fade-leave-to {
  opacity: 0;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  border-bottom: 1px solid var(--reader-border, var(--color-border));
  flex-shrink: 0;
}

.panel-header h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  border: none;
  background: none;
  font-size: 1.4rem;
  color: var(--color-muted);
  cursor: pointer;
  line-height: 1;
}

.panel-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 16px;
  overflow: hidden;
}

.quote-preview {
  flex-shrink: 0;
  margin: 0;
  padding: 10px 12px;
  font-size: 0.84rem;
  color: var(--color-muted);
  background: var(--color-bg, rgba(0, 0, 0, 0.04));
  border-radius: var(--radius-sm);
  line-height: 1.6;
  border-left: 3px solid var(--color-primary);
}

.comment-input {
  flex: 1;
  width: 100%;
  min-height: 280px;
  resize: none;
  border: 1px solid var(--reader-border, var(--color-border));
  border-radius: var(--radius-md);
  padding: 12px 14px;
  font-family: inherit;
  font-size: 0.9rem;
  line-height: 1.65;
  background: transparent;
  color: inherit;
}

.comment-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-soft);
}

.panel-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-top: 1px solid var(--reader-border, var(--color-border));
  flex-shrink: 0;
}

.hint {
  font-size: 0.72rem;
  color: var(--color-muted);
}

.actions {
  display: flex;
  gap: 8px;
}

.btn-primary {
  border: none;
  border-radius: var(--radius-pill);
  padding: 8px 18px;
  background: var(--color-primary);
  color: #fff;
  font-size: 0.88rem;
  cursor: pointer;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (prefers-reduced-motion: reduce) {
  .comment-panel,
  .comment-panel--drawer {
    animation: none;
  }
}
</style>
