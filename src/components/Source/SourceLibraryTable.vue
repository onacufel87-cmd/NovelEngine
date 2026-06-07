<template>
  <section class="source-table-wrap">
    <div class="table-toolbar">
      <input
        v-model="keyword"
        type="search"
        class="table-search"
        placeholder="搜索名称或 ID…"
      />
      <select v-model="filterOrigin" class="table-filter">
        <option value="all">全部来源</option>
        <option value="subscription">订阅</option>
        <option value="custom">自定义</option>
      </select>
      <select v-model="filterHealth" class="table-filter">
        <option value="all">全部状态</option>
        <option value="online">在线</option>
        <option value="slow">较慢</option>
        <option value="offline">离线</option>
        <option value="unknown">未检测</option>
      </select>
      <select v-model="filterEnabled" class="table-filter">
        <option value="all">启用/禁用</option>
        <option value="enabled">已启用</option>
        <option value="disabled">已禁用</option>
      </select>
    </div>

    <div v-if="selectedIds.size" class="batch-bar">
      <span>已选 {{ selectedIds.size }} 项</span>
      <BaseButton :disabled="pinging" @click="handleBatchPing">
        {{ pinging ? "检测中…" : "一键检测" }}
      </BaseButton>
      <button type="button" class="ghost-btn" @click="handleBatchDisable">批量禁用</button>
      <button type="button" class="ghost-btn ghost-btn--danger" @click="handleBatchDelete">批量删除</button>
      <button type="button" class="link-btn" @click="clearSelection">取消选择</button>
    </div>

    <div class="table-scroll">
      <table class="source-table">
        <thead>
          <tr>
            <th class="col-check">
              <input
                type="checkbox"
                :checked="allVisibleSelected"
                :indeterminate="someVisibleSelected && !allVisibleSelected"
                @change="toggleSelectAll"
              />
            </th>
            <th>名称</th>
            <th>来源</th>
            <th class="col-health">健康</th>
            <th class="col-ping">延迟</th>
            <th class="col-time">上次检测</th>
            <th class="col-switch">启用</th>
            <th class="col-actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="src in filteredSources"
            :key="src.id"
            :class="{ 'row--off': !src.enabled }"
          >
            <td class="col-check">
              <input
                type="checkbox"
                :checked="selectedIds.has(src.id)"
                @change="toggleSelect(src.id)"
              />
            </td>
            <td>
              <div class="cell-name">{{ src.name }}</div>
              <div class="cell-id">{{ src.id }}</div>
            </td>
            <td>
              <span class="origin-badge" :class="`origin-badge--${src.origin || 'custom'}`">
                {{ originLabel(src) }}
              </span>
            </td>
            <td class="col-health">
              <span class="health-dot" :class="`health-dot--${src.health_status || 'unknown'}`" />
              {{ healthLabel(src.health_status) }}
            </td>
            <td class="col-ping">
              <span v-if="src.ping_ms != null">{{ src.ping_ms }} ms</span>
              <span v-else class="muted">—</span>
            </td>
            <td class="col-time muted">{{ formatRelativeTime(src.last_verified) }}</td>
            <td class="col-switch">
              <label class="mini-toggle">
                <input
                  type="checkbox"
                  :checked="src.enabled"
                  @change="$emit('toggle', src.id, $event.target.checked)"
                />
                <span />
              </label>
            </td>
            <td class="col-actions">
              <button
                type="button"
                class="link-btn"
                :disabled="pingingOne === src.id"
                @click="handlePingOne(src.id)"
              >
                {{ pingingOne === src.id ? "…" : "检测" }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="!filteredSources.length" class="empty-hint">没有匹配的书源</p>
    </div>
  </section>
</template>

<script setup>
import { computed } from "vue";
import BaseButton from "../Common/BaseButton.vue";
import { formatRelativeTime } from "../../utils/relativeTime";
import { useBookSourceStore } from "../../stores/bookSourceStore";

const bookSourceStore = useBookSourceStore();

const props = defineProps({
  /** 仅展示非内置书源 */
  sources: { type: Array, default: () => [] },
  pinging: { type: Boolean, default: false },
});

const emit = defineEmits(["toggle", "delete-batch", "disable-batch"]);

const keyword = ref("");
const filterOrigin = ref("all");
const filterHealth = ref("all");
const filterEnabled = ref("all");
const selectedIds = ref(new Set());
const pingingOne = ref("");

const filteredSources = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return props.sources.filter((s) => {
    if (filterOrigin.value !== "all" && (s.origin || "custom") !== filterOrigin.value) {
      return false;
    }
    if (filterHealth.value !== "all" && (s.health_status || "unknown") !== filterHealth.value) {
      return false;
    }
    if (filterEnabled.value === "enabled" && !s.enabled) return false;
    if (filterEnabled.value === "disabled" && s.enabled) return false;
    if (!kw) return true;
    return (
      (s.name || "").toLowerCase().includes(kw) ||
      (s.id || "").toLowerCase().includes(kw)
    );
  });
});

const allVisibleSelected = computed(
  () =>
    filteredSources.value.length > 0 &&
    filteredSources.value.every((s) => selectedIds.value.has(s.id))
);

const someVisibleSelected = computed(() =>
  filteredSources.value.some((s) => selectedIds.value.has(s.id))
);

function originLabel(src) {
  if (src.origin === "subscription") return "订阅";
  if (src.origin === "custom") return "自定义";
  return src.origin || "自定义";
}

function healthLabel(status) {
  const map = {
    online: "在线",
    slow: "较慢",
    offline: "离线",
    unknown: "未检测",
  };
  return map[status] || "未检测";
}

function toggleSelect(id) {
  const next = new Set(selectedIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selectedIds.value = next;
}

function toggleSelectAll(e) {
  const next = new Set(selectedIds.value);
  if (e.target.checked) {
    filteredSources.value.forEach((s) => next.add(s.id));
  } else {
    filteredSources.value.forEach((s) => next.delete(s.id));
  }
  selectedIds.value = next;
}

function clearSelection() {
  selectedIds.value = new Set();
}

async function handlePingOne(id) {
  pingingOne.value = id;
  try {
    await bookSourceStore.pingSource(id);
  } finally {
    pingingOne.value = "";
  }
}

function handleBatchPing() {
  bookSourceStore.pingSourcesBatch([...selectedIds.value]);
}

function handleBatchDisable() {
  emit("disable-batch", [...selectedIds.value]);
}

function handleBatchDelete() {
  if (!selectedIds.value.size) return;
  if (!window.confirm(`确定删除选中的 ${selectedIds.value.size} 个书源？内置书源不会被删除。`)) {
    return;
  }
  emit("delete-batch", [...selectedIds.value]);
  clearSelection();
}
</script>

<style scoped>
.source-table-wrap {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.table-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.table-search {
  flex: 1;
  min-width: 160px;
  padding: 8px 12px;
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  background: var(--color-surface);
}

.table-filter {
  padding: 8px 10px;
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
  font-size: 0.82rem;
  background: var(--color-surface);
  color: var(--color-text);
}

.batch-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--color-primary-soft);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
}

.table-scroll {
  overflow-x: auto;
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
}

.source-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.84rem;
}

.source-table th,
.source-table td {
  padding: 10px 12px;
  text-align: left;
  border-bottom: 1px solid var(--color-border-light);
}

.source-table th {
  font-weight: 600;
  color: var(--color-muted);
  background: var(--color-bg);
  white-space: nowrap;
}

.source-table tbody tr:hover {
  background: var(--color-hover);
}

.row--off {
  opacity: 0.65;
}

.col-check {
  width: 36px;
}

.col-health,
.col-ping,
.col-time {
  white-space: nowrap;
}

.col-switch {
  width: 52px;
}

.col-actions {
  width: 56px;
}

.cell-name {
  font-weight: 600;
}

.cell-id {
  font-size: 0.72rem;
  color: var(--color-muted);
  margin-top: 2px;
  word-break: break-all;
}

.origin-badge {
  font-size: 0.72rem;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--color-surface);
  border: 1px solid var(--color-border-light);
}

.origin-badge--subscription {
  color: var(--color-primary);
  border-color: var(--color-primary-soft);
  background: var(--color-primary-soft);
}

.health-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
  vertical-align: middle;
}

.health-dot--online {
  background: var(--color-success);
  box-shadow: 0 0 5px var(--color-success);
}

.health-dot--slow {
  background: #d4a373;
}

.health-dot--offline {
  background: var(--color-danger);
}

.health-dot--unknown {
  background: var(--color-muted);
  opacity: 0.45;
}

.mini-toggle {
  position: relative;
  display: inline-block;
  width: 32px;
  height: 18px;
  cursor: pointer;
}

.mini-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.mini-toggle span {
  position: absolute;
  inset: 0;
  background: var(--color-border);
  border-radius: 9px;
  transition: background 0.2s;
}

.mini-toggle span::after {
  content: "";
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
}

.mini-toggle input:checked + span {
  background: var(--color-primary);
}

.mini-toggle input:checked + span::after {
  transform: translateX(14px);
}

.link-btn {
  background: none;
  border: none;
  color: var(--color-primary);
  font-size: 0.82rem;
  cursor: pointer;
  padding: 0;
}

.link-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.muted {
  color: var(--color-muted);
}

.empty-hint {
  text-align: center;
  padding: 24px;
  color: var(--color-muted);
  font-size: 0.88rem;
}

.ghost-btn {
  padding: 6px 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-pill);
  background: var(--color-surface);
  font-size: 0.82rem;
  cursor: pointer;
  color: var(--color-text);
}

.ghost-btn--danger {
  color: var(--color-danger);
  border-color: var(--color-danger-bg);
}
</style>
