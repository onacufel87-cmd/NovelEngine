<template>
  <details v-if="logs.length" class="detect-console" :open="expanded">
    <summary class="detect-console__summary">
      <span class="detect-console__title">检测日志</span>
      <span class="detect-console__count">{{ logs.length }} 条</span>
      <span v-if="failed" class="detect-console__badge detect-console__badge--fail">失败</span>
      <span v-else class="detect-console__badge detect-console__badge--ok">完成</span>
    </summary>
    <div ref="scrollRef" class="detect-console__body">
      <div
        v-for="(entry, idx) in logs"
        :key="idx"
        class="detect-console__line"
        :class="`detect-console__line--${entry.level.toLowerCase()}`"
      >
        <span class="detect-console__ts">{{ formatLogTime(entry.ts) }}</span>
        <span class="detect-console__level">[{{ entry.level }}]</span>
        <span class="detect-console__msg">{{ entry.message }}</span>
      </div>
    </div>
  </details>
</template>

<script setup>
import { computed, nextTick, ref, watch } from "vue";

const props = defineProps({
  logs: { type: Array, default: () => [] },
  /** 检测是否失败（失败时默认展开） */
  failed: { type: Boolean, default: false },
  /** 用户手动展开后保持 */
  forceOpen: { type: Boolean, default: false },
});

const scrollRef = ref(null);

/** 失败自动展开，成功默认折叠 */
const expanded = computed(() => props.failed || props.forceOpen);

/** 格式化为 HH:MM:SS */
function formatLogTime(unixSec) {
  if (!unixSec) return "--:--:--";
  const d = new Date(unixSec * 1000);
  return d.toLocaleTimeString("zh-CN", { hour12: false });
}

/** 新日志追加后滚到底部 */
watch(
  () => props.logs.length,
  async () => {
    await nextTick();
    const el = scrollRef.value;
    if (el) el.scrollTop = el.scrollHeight;
  }
);
</script>

<style scoped>
.detect-console {
  border: 1px solid #2a3a32;
  border-radius: var(--radius-md);
  background: #1a2420;
  overflow: hidden;
  font-family: "Cascadia Code", "Fira Code", Consolas, monospace;
}

.detect-console__summary {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  list-style: none;
  background: #222e28;
  color: #c8d4c8;
  font-size: 0.82rem;
  user-select: none;
}

.detect-console__summary::-webkit-details-marker {
  display: none;
}

.detect-console__title {
  font-weight: 600;
  color: #e8efe8;
}

.detect-console__count {
  color: #7a9a7a;
  font-size: 0.75rem;
}

.detect-console__badge {
  margin-left: auto;
  font-size: 0.68rem;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
}

.detect-console__badge--ok {
  background: rgba(156, 175, 136, 0.2);
  color: #9caf88;
}

.detect-console__badge--fail {
  background: rgba(214, 158, 122, 0.2);
  color: #d69e7a;
}

.detect-console__body {
  max-height: 280px;
  overflow-y: auto;
  padding: 10px 0;
}

.detect-console__line {
  display: flex;
  gap: 10px;
  padding: 3px 14px;
  font-size: 0.76rem;
  line-height: 1.55;
  color: #b8c8b8;
}

.detect-console__line:hover {
  background: rgba(255, 255, 255, 0.03);
}

.detect-console__ts {
  flex-shrink: 0;
  color: #5a7a5a;
  min-width: 64px;
}

.detect-console__level {
  flex-shrink: 0;
  min-width: 72px;
  font-weight: 600;
}

.detect-console__msg {
  word-break: break-all;
  white-space: pre-wrap;
}

.detect-console__line--info .detect-console__level {
  color: #7ab8d4;
}

.detect-console__line--warn .detect-console__level {
  color: #d4a373;
}

.detect-console__line--success .detect-console__level {
  color: #9caf88;
}

.detect-console__line--error .detect-console__level {
  color: #d69e7a;
}
</style>
