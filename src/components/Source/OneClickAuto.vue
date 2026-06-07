<template>
  <section class="card one-click-auto">
    <header>
      <h3 class="card-title">一键全自动接入</h3>
      <p class="card-desc">
        粘贴任意小说网站 URL（首页、目录页、正文页均可），系统自动发现目录/正文并生成书源规则。
      </p>
    </header>

    <form class="form-row" @submit.prevent="handleAutoAdd">
      <label class="form-field">
        <span>网站 URL</span>
        <input
          v-model="url"
          type="url"
          required
          placeholder="https://example.com 或具体小说目录/正文页"
        />
      </label>
      <label class="form-field checkbox-row">
        <input v-model="useRendered" type="checkbox" />
        <span>使用浏览器渲染（较慢，可绕过部分 JS 动态站）</span>
      </label>
      <div class="form-actions">
        <BaseButton type="submit" :disabled="loading">
          {{ loading ? "分析中…" : useRendered ? "渲染并接入…" : "开始接入" }}
        </BaseButton>
        <button v-if="isDev" type="button" class="btn-link" @click="fillLocalTest">本地测试 URL</button>
      </div>
    </form>

    <p v-if="error" class="msg msg--error" style="margin-top: 14px">
      {{ error }}
      <button type="button" class="fallback-link" @click="$emit('fallback')">
        改用手动检测 →
      </button>
    </p>

    <div v-if="result" class="result-box">
      <p class="success-head">
        接入成功
        <span class="tag badge--primary">置信度 {{ result.confidence }}%</span>
      </p>
      <ul class="meta-list">
        <li><span>书源名称</span><strong>{{ result.source.name }}</strong></li>
        <li><span>目录页</span><code class="code-block">{{ result.toc_url }}</code></li>
        <li><span>正文页</span><code class="code-block">{{ result.content_url }}</code></li>
        <li v-if="result.source.search_url">
          <span>搜索接口</span><code class="code-block">{{ result.source.search_url }}</code>
        </li>
        <li>
          <span>章节选择器</span><code class="code-block">{{ result.source.chapter_list_selector }}</code>
        </li>
        <li>
          <span>正文选择器</span><code class="code-block">{{ result.source.content_selector }}</code>
        </li>
      </ul>

      <div class="form-actions">
        <BaseButton @click="saveSource">保存书源</BaseButton>
        <button type="button" class="btn-ghost" @click="applyAndParse">保存并解析目录</button>
      </div>
      <p v-if="statusMessage" class="msg msg--ok" style="margin-top: 10px">{{ statusMessage }}</p>
    </div>
  </section>
</template>

<script setup>
import { ref } from "vue";
import { autoDetectFromUrl } from "../../api/detect";
import { importBookSourceJson } from "../../api";
import { useBookSourceStore } from "../../stores/bookSourceStore";
import { useSourceWizardStore } from "../../stores/sourceWizardStore";
import BaseButton from "../Common/BaseButton.vue";

const emit = defineEmits([
  "fallback",
  "detect-start",
  "detect-complete",
  "detect-fail",
  "trial-start",
]);

const bookSourceStore = useBookSourceStore();
const wizard = useSourceWizardStore();

const url = ref("");
const useRendered = ref(false);
const loading = ref(false);
const error = ref("");
const result = ref(null);
const statusMessage = ref("");
const isDev = import.meta.env.DEV;

function fillLocalTest() {
  url.value = `${window.location.origin}/test-catalog.html`;
}

async function handleAutoAdd() {
  if (!url.value.trim()) return;
  loading.value = true;
  error.value = "";
  statusMessage.value = "";
  result.value = null;
  emit("detect-start");

  try {
    const { result: data, logs } = await autoDetectFromUrl(url.value.trim(), {
      useRendered: useRendered.value,
    });
    wizard.setDetectLogs(logs, false);
    result.value = data;
    emit("detect-complete", { catalogUrl: data.toc_url });
  } catch (err) {
    wizard.setDetectLogs(err.logs || [], true);
    error.value =
      String(err.message || err) ||
      "自动检测失败，可能该网站结构特殊。请尝试提供目录页与正文页 URL 进行手动检测。";
    emit("detect-fail");
  } finally {
    loading.value = false;
  }
}

function sourceToJson(source) {
  return JSON.stringify({
    ...source,
    ad_keywords: source.ad_keywords ?? [],
    clean_patterns: source.clean_patterns ?? [],
  });
}

async function saveSource() {
  if (!result.value) return;
  try {
    const record = await importBookSourceJson(sourceToJson(result.value.source));
    await bookSourceStore.loadSources();
    statusMessage.value = `书源已保存：${record.name}`;
  } catch (err) {
    error.value = String(err);
  }
}

async function applyAndParse() {
  if (!result.value) return;
  try {
    await bookSourceStore.loadParseRule(result.value.source);
    emit("trial-start", { catalogUrl: result.value.toc_url });
    await bookSourceStore.parseCatalog(result.value.toc_url);
    statusMessage.value = bookSourceStore.parseSession.statusMessage || "规则已加载，目录解析完成";
  } catch (err) {
    error.value = String(err);
  }
}
</script>

<style scoped>
.success-head {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  margin-bottom: 14px;
  color: var(--color-success);
}

.result-box {
  margin-top: 18px;
  padding-top: 18px;
  border-top: 1px solid var(--color-border-light);
}

.meta-list {
  list-style: none;
  margin-bottom: 16px;
}

.meta-list li {
  margin-bottom: 10px;
  font-size: 0.84rem;
}

.meta-list li > span {
  display: block;
  color: var(--color-muted);
  margin-bottom: 3px;
}

.fallback-link {
  display: inline-block;
  margin-left: 8px;
  background: none;
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  font-size: inherit;
  text-decoration: underline;
}

.checkbox-row {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--color-muted);
}
</style>
