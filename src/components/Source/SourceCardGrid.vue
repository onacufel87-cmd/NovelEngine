<template>

  <div class="source-grid-wrap">

    <section v-if="builtinSources.length" class="source-section">

      <div class="section-head">

        <h4 class="section-title">内置公版书源</h4>

        <button

          type="button"

          class="link-btn"

          :disabled="pingingBuiltin"

          @click="pingAllBuiltin"

        >

          {{ pingingBuiltin ? "检测中…" : "检测全部" }}

        </button>

      </div>

      <div class="source-grid">

        <article

          v-for="src in builtinSources"

          :key="src.id"

          class="source-card"

          :class="{ 'source-card--off': !src.enabled }"

        >

          <div class="source-card__head">

            <span

              class="status-dot"

              :class="healthDotClass(src)"

              :title="healthTitle(src)"

            />

            <h5 class="source-card__name">{{ src.name }}</h5>

            <label class="source-toggle" :title="src.enabled ? '禁用' : '启用'">

              <input

                type="checkbox"

                :checked="src.enabled"

                @change="$emit('toggle', src.id, $event.target.checked)"

              />

              <span class="toggle-ui" />

            </label>

          </div>

          <div class="source-card__tags">

            <span v-for="tag in tagsFor(src)" :key="tag.label" class="tag" :class="`tag--${tag.type}`">

              {{ tag.label }}

            </span>

          </div>

          <p class="source-card__meta">{{ healthMeta(src) }}</p>

          <p class="source-card__desc">{{ src.description || "无描述" }}</p>

        </article>

      </div>

    </section>



    <p v-if="!sources.length" class="empty-hint">暂无书源，请订阅仓库或手动导入。</p>

  </div>

</template>



<script setup>

import { computed } from "vue";

import { formatRelativeTime } from "../../utils/relativeTime";



const props = defineProps({

  sources: { type: Array, default: () => [] },

  pinging: { type: Boolean, default: false },

});



const emit = defineEmits(["toggle", "ping-batch"]);



const pingingBuiltin = computed(() => props.pinging);



const builtinSources = computed(() => props.sources.filter((s) => s.is_builtin));



/** 根据 health_status 决定指示灯样式 */

function healthDotClass(src) {

  const status = src.health_status || "unknown";

  if (!src.enabled) return "status-dot--off";

  return `status-dot--${status}`;

}



function healthTitle(src) {

  const map = {

    online: "在线",

    slow: "响应较慢",

    offline: "离线",

    unknown: "未检测",

  };

  return map[src.health_status] || "未检测";

}



function healthMeta(src) {

  const parts = [];

  if (src.ping_ms != null) parts.push(`Ping ${src.ping_ms} ms`);

  parts.push(`上次检测 ${formatRelativeTime(src.last_verified)}`);

  return parts.join(" · ");

}



/** 优先读 DB tags JSON，否则按 id/名称推断 */

function tagsFor(src) {

  if (src.tags) {

    try {

      const parsed = JSON.parse(src.tags);

      if (Array.isArray(parsed)) {

        return parsed.map((label) => ({

          label,

          type: label === "EN" || label === "ZH" ? "lang" : label === "公版" ? "public" : "builtin",

        }));

      }

    } catch {

      /* 忽略非法 JSON */

    }

  }



  const tags = [{ label: "内置", type: "builtin" }];

  const id = src.id || "";

  const name = src.name || "";



  if (id.includes("gutenberg") || name.includes("Gutenberg")) {

    tags.push({ label: "EN", type: "lang" });

  }

  if (id.includes("wikisource") || name.includes("维基")) {

    tags.push({ label: "ZH", type: "lang" });

  }

  if (

    id.includes("openlibrary") ||

    name.includes("Open Library") ||

    name.includes("公版")

  ) {

    tags.push({ label: "公版", type: "public" });

  }

  return tags;

}



function pingAllBuiltin() {

  const ids = builtinSources.value.map((s) => s.id);

  emit("ping-batch", ids);

}

</script>



<style scoped>

.source-grid-wrap {

  display: flex;

  flex-direction: column;

  gap: 20px;

}



.section-head {

  display: flex;

  align-items: center;

  justify-content: space-between;

  margin-bottom: 12px;

}



.section-title {

  font-size: 0.88rem;

  font-weight: 600;

  color: var(--color-muted);

  margin: 0;

}



.link-btn {

  background: none;

  border: none;

  color: var(--color-primary);

  font-size: 0.82rem;

  cursor: pointer;

}



.link-btn:disabled {

  opacity: 0.5;

  cursor: not-allowed;

}



.source-grid {

  display: grid;

  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));

  gap: 14px;

}



.source-card {

  padding: 14px 16px;

  border: 1px solid var(--color-border-light);

  border-radius: var(--radius-md);

  background: var(--color-bg);

  transition: border-color 0.15s, box-shadow 0.15s;

}



.source-card:hover {

  border-color: var(--color-border);

  box-shadow: var(--shadow-sm);

}



.source-card--off {

  opacity: 0.72;

}



.source-card__head {

  display: flex;

  align-items: flex-start;

  gap: 8px;

  margin-bottom: 8px;

}



.status-dot {

  width: 8px;

  height: 8px;

  border-radius: 50%;

  margin-top: 6px;

  flex-shrink: 0;

}



.status-dot--online {

  background: var(--color-success);

  box-shadow: 0 0 6px var(--color-success);

}



.status-dot--slow {

  background: #d4a373;

}



.status-dot--offline {

  background: var(--color-danger);

}



.status-dot--unknown {

  background: var(--color-muted);

  opacity: 0.45;

}



.status-dot--off {

  background: var(--color-muted);

  opacity: 0.5;

}



.source-card__name {

  flex: 1;

  font-size: 0.9rem;

  font-weight: 600;

  line-height: 1.4;

  margin: 0;

  min-width: 0;

}



.source-toggle {

  position: relative;

  flex-shrink: 0;

  cursor: pointer;

}



.source-toggle input {

  position: absolute;

  opacity: 0;

  width: 0;

  height: 0;

}



.toggle-ui {

  display: block;

  width: 36px;

  height: 20px;

  border-radius: 10px;

  background: var(--color-border);

  transition: background 0.2s;

  position: relative;

}



.toggle-ui::after {

  content: "";

  position: absolute;

  top: 2px;

  left: 2px;

  width: 16px;

  height: 16px;

  border-radius: 50%;

  background: #fff;

  transition: transform 0.2s;

  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);

}



.source-toggle input:checked + .toggle-ui {

  background: var(--color-primary);

}



.source-toggle input:checked + .toggle-ui::after {

  transform: translateX(16px);

}



.source-card__tags {

  display: flex;

  flex-wrap: wrap;

  gap: 6px;

  margin-bottom: 6px;

}



.tag {

  font-size: 0.68rem;

  padding: 2px 8px;

  border-radius: var(--radius-pill);

  background: var(--color-surface);

  color: var(--color-muted);

  border: 1px solid var(--color-border-light);

}



.tag--public {

  color: var(--color-success);

  border-color: var(--color-success-bg);

  background: var(--color-success-bg);

}



.tag--lang {

  color: var(--color-primary);

  border-color: var(--color-primary-soft);

  background: var(--color-primary-soft);

}



.source-card__meta {

  font-size: 0.72rem;

  color: var(--color-muted);

  margin: 0 0 6px;

}



.source-card__desc {

  font-size: 0.78rem;

  color: var(--color-muted);

  line-height: 1.55;

  margin: 0;

  display: -webkit-box;

  -webkit-line-clamp: 2;

  -webkit-box-orient: vertical;

  overflow: hidden;

}



.empty-hint {

  font-size: 0.88rem;

  color: var(--color-muted);

  text-align: center;

  padding: 24px;

}

</style>


