<template>
  <section class="card auto-detect">
    <header>
      <h3 class="card-title">零配置自动检测</h3>
      <p class="card-desc">提供目录页与正文页 URL，自动推断 CSS 选择器并生成书源规则。</p>
    </header>

    <form class="form-row" @submit.prevent="handleDetect">
      <label class="form-field">
        <span>目录页 URL<em>必填</em></span>
        <input v-model="tocUrl" type="url" required placeholder="https://example.com/book/123/" />
      </label>
      <label class="form-field">
        <span>正文页 URL<em>必填</em></span>
        <input v-model="contentUrl" type="url" required placeholder="https://example.com/book/123/1.html" />
      </label>
      <label class="form-field">
        <span>搜索页 URL<em>可选</em></span>
        <input v-model="searchUrl" type="url" placeholder="https://example.com/search?q={keyword}" />
      </label>
      <label class="form-field">
        <span>书源名称</span>
        <input v-model="sourceName" type="text" placeholder="我的书源" />
      </label>

      <div class="form-actions">
        <BaseButton type="submit" :disabled="detecting">
          {{ detecting ? "检测中…" : "开始检测" }}
        </BaseButton>
        <button v-if="isDev" type="button" class="btn-link" @click="fillLocalTestUrls">填入本地测试 URL</button>
      </div>
    </form>

    <p v-if="error" class="msg msg--error" style="margin-top: 12px">{{ error }}</p>

    <div v-if="detected" class="result-box">
      <p class="confidence">
        置信度 <strong>{{ detected.confidence }}%</strong>
        <span v-if="detected.confidence < 60" class="warn">建议人工核对</span>
      </p>
      <ul class="selector-list">
        <li><span>章节列表</span><code class="code-block">{{ detected.chapter_list_item }}</code></li>
        <li><span>正文容器</span><code class="code-block">{{ detected.content_container }}</code></li>
        <li v-if="detected.search_result_item">
          <span>搜索结果</span><code class="code-block">{{ detected.search_result_item }}</code>
        </li>
      </ul>

      <details class="rule-json-panel" open>
        <summary>生成的书源规则 JSON</summary>
        <JsonRuleEditor
          v-model="ruleJsonText"
          min-height="240px"
          :validate-rule="true"
        />
      </details>

      <div class="form-actions">
        <BaseButton @click="applyAndParse">确认并解析目录</BaseButton>
        <button type="button" class="btn-ghost" @click="applyOnly">仅加载规则</button>
        <button type="button" class="btn-ghost" @click="importToLibrary">导入书源库</button>
      </div>
      <p v-if="statusMessage" class="msg msg--ok" style="margin-top: 10px">{{ statusMessage }}</p>
    </div>
  </section>
</template>

<script setup>
import { ref } from "vue";
import { autoDetectSourceRule } from "../../api/detect";
import { importBookSourceJson } from "../../api";
import { useBookSourceStore } from "../../stores/bookSourceStore";
import { useSourceWizardStore } from "../../stores/sourceWizardStore";
import BaseButton from "../Common/BaseButton.vue";
import JsonRuleEditor from "./JsonRuleEditor.vue";

const emit = defineEmits([
  "detect-start",
  "detect-complete",
  "detect-fail",
  "trial-start",
]);

const bookSourceStore = useBookSourceStore();
const wizard = useSourceWizardStore();

const tocUrl = ref("");
const contentUrl = ref("");
const searchUrl = ref("");
const sourceName = ref("自动检测书源");
const detecting = ref(false);
const error = ref("");
const detected = ref(null);
const statusMessage = ref("");
const ruleJsonText = ref("");
const isDev = import.meta.env.DEV;

function fillLocalTestUrls() {
  const origin = window.location.origin;
  tocUrl.value = `${origin}/test-catalog.html`;
  contentUrl.value = `${origin}/test-chapter.html`;
  searchUrl.value = `${origin}/test-search.html?q={keyword}`;
  sourceName.value = "本地测试书源";
}

async function handleDetect() {
  detecting.value = true;
  error.value = "";
  statusMessage.value = "";
  detected.value = null;
  emit("detect-start");

  try {
    const { ruleJson, logs } = await autoDetectSourceRule({
      name: sourceName.value.trim() || "自动检测书源",
      tocUrl: tocUrl.value.trim(),
      contentUrl: contentUrl.value.trim(),
      searchUrl: searchUrl.value.trim() || null,
    });
    wizard.setDetectLogs(logs, false);
    ruleJsonText.value = ruleJson;
    const rule = JSON.parse(ruleJson);
    detected.value = {
      chapter_list_item: rule.chapter_list_selector,
      content_container: rule.content_selector,
      search_result_item: rule.search_result_selector || "",
      result_title: rule.search_title_selector || "a",
      result_url_attr: rule.search_link_attr || "href",
      confidence: rule._confidence ?? 0,
    };
    emit("detect-complete", { catalogUrl: tocUrl.value.trim() });
  } catch (err) {
    wizard.setDetectLogs(err.logs || [], true);
    error.value = String(err.message || err);
    emit("detect-fail");
  } finally {
    detecting.value = false;
  }
}

async function applyOnly() {
  if (!ruleJsonText.value) return;
  try {
    const rule = JSON.parse(ruleJsonText.value);
    delete rule._confidence;
    await bookSourceStore.loadParseRule(rule);
    statusMessage.value = "书源规则已加载。";
  } catch (err) {
    error.value = String(err);
  }
}

async function applyAndParse() {
  await applyOnly();
  if (bookSourceStore.hasParseRule && tocUrl.value.trim()) {
    emit("trial-start", { catalogUrl: tocUrl.value.trim() });
    await bookSourceStore.parseCatalog(tocUrl.value.trim());
    statusMessage.value = bookSourceStore.parseSession.statusMessage || "目录解析完成";
  }
}

async function importToLibrary() {
  if (!ruleJsonText.value) return;
  try {
    const rule = JSON.parse(ruleJsonText.value);
    delete rule._confidence;
    const record = await importBookSourceJson(JSON.stringify(rule));
    statusMessage.value = `已导入书源库：${record.name}`;
  } catch (err) {
    error.value = String(err);
  }
}
</script>

<style scoped>
.result-box {
  margin-top: 18px;
  padding-top: 18px;
  border-top: 1px solid var(--color-border-light);
}

.confidence {
  font-size: 0.88rem;
  color: var(--color-muted);
}

.confidence strong {
  color: var(--color-primary);
  font-size: 1rem;
}

.confidence .warn {
  margin-left: 8px;
  color: #d97706;
  font-size: 0.82rem;
}

.selector-list {
  list-style: none;
  margin: 14px 0;
}

.selector-list li {
  margin-bottom: 10px;
  font-size: 0.84rem;
  color: var(--color-muted);
}

.rule-json-panel {
  margin: 14px 0 16px;
}

.rule-json-panel summary {
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-muted);
  margin-bottom: 10px;
}
</style>
