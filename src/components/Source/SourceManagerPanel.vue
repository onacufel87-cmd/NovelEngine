<template>

  <section class="card source-manager">

    <h3 class="card-title">书源管理</h3>

    <p class="callout callout--warn">

      官方不含盗版书源。可订阅社区维护的书源<strong>仓库索引</strong>（JSON 链接），一键导入数百条规则。

    </p>



    <div class="repo-subscribe card-inner">

      <h4 class="card-section-title">书源仓库订阅</h4>

      <p class="card-desc repo-desc">

        粘贴第三方书源列表 URL（JSON 数组或 <code>{ "sources": [...] }</code> 格式），自动批量导入。

      </p>

      <form class="form-row" @submit.prevent="handleSubscribe">

        <label class="form-field">

          <span>仓库 URL</span>

          <input

            v-model="remoteUrl"

            type="url"

            placeholder="https://example.com/book-sources.json"

          />

        </label>

        <div class="form-actions">

          <BaseButton type="submit" :disabled="subscribing || !remoteUrl.trim()">

            {{ subscribing ? "拉取中…" : "一键订阅仓库" }}

          </BaseButton>

        </div>

      </form>

      <div v-if="batchResult" class="batch-result">

        <p class="msg msg--ok">

          导入 {{ batchResult.imported }} · 更新 {{ batchResult.updated }}

          <span v-if="batchResult.failed"> · 失败 {{ batchResult.failed }}</span>

        </p>

        <ul v-if="batchResult.errors?.length" class="error-list">

          <li v-for="(err, i) in batchResult.errors.slice(0, 5)" :key="i">{{ err }}</li>

        </ul>

      </div>



      <!-- 已订阅仓库列表 + 同步按钮 -->

      <ul v-if="bookSourceStore.subscriptions.length" class="subscription-list">

        <li

          v-for="sub in bookSourceStore.subscriptions"

          :key="sub.id"

          class="subscription-item"

        >

          <div class="subscription-info">

            <span class="subscription-label">{{ sub.label || "书源仓库" }}</span>

            <span class="subscription-url" :title="sub.url">{{ sub.url }}</span>

            <span class="subscription-meta">

              {{ sub.source_count }} 个书源 · 上次同步 {{ formatRelativeTime(sub.last_synced_at) }}

            </span>

          </div>

          <BaseButton

            :disabled="bookSourceStore.syncingSubscriptionId === sub.id"

            @click="handleSync(sub.id)"

          >

            {{ bookSourceStore.syncingSubscriptionId === sub.id ? "同步中…" : "同步更新" }}

          </BaseButton>

        </li>

      </ul>

    </div>



    <LoadingSpinner v-if="bookSourceStore.loadingSources" text="加载书源…" />



    <template v-else>

      <SourceCardGrid

        :sources="bookSourceStore.sources"

        :pinging="bookSourceStore.pingingSources"

        @toggle="(id, enabled) => bookSourceStore.setEnabled(id, enabled)"

        @ping-batch="(ids) => bookSourceStore.pingSourcesBatch(ids)"

      />



      <section v-if="bookSourceStore.managedSources.length" class="managed-section">

        <h4 class="card-section-title">

          订阅 / 自定义书源

          <span class="section-count">{{ bookSourceStore.managedSources.length }} 个</span>

        </h4>

        <SourceLibraryTable
          :sources="bookSourceStore.managedSources"
          :pinging="bookSourceStore.pingingSources"
          @toggle="(id, enabled) => bookSourceStore.setEnabled(id, enabled)"
          @delete-batch="(ids) => bookSourceStore.deleteSourcesBatch(ids)"
          @disable-batch="(ids) => bookSourceStore.disableSourcesBatch(ids)"
        />

      </section>

    </template>



    <details class="card-collapse import-section">

      <summary>手动添加 · JSON 粘贴</summary>

      <div class="card-collapse-body">

        <form class="form-row" @submit.prevent="handleImportJson">
          <label class="form-field form-field--full">
            <span>书源 JSON（单条 / 数组 / 仓库格式）</span>
            <JsonRuleEditor
              v-model="jsonText"
              min-height="220px"
              :validate-rule="false"
              hint="支持单条书源、数组或仓库格式 · Ctrl+S 校验 JSON 语法"
            />
          </label>
          <div class="form-actions">
            <BaseButton type="submit" :disabled="importing || !jsonText.trim()">
              {{ importing ? "导入中…" : "导入 JSON" }}
            </BaseButton>
          </div>
        </form>

      </div>

    </details>



    <p v-if="sourceMessage" class="msg msg--ok source-msg">{{ sourceMessage }}</p>

  </section>

</template>



<script setup>

import { ref } from "vue";

import { useBookSourceStore } from "../../stores/bookSourceStore";

import { importBookSourcesBatch } from "../../api";

import { formatRelativeTime } from "../../utils/relativeTime";

import BaseButton from "../Common/BaseButton.vue";

import LoadingSpinner from "../Common/LoadingSpinner.vue";

import SourceCardGrid from "./SourceCardGrid.vue";

import SourceLibraryTable from "./SourceLibraryTable.vue";

import JsonRuleEditor from "./JsonRuleEditor.vue";

const bookSourceStore = useBookSourceStore();

const remoteUrl = ref("");

const jsonText = ref("");

const subscribing = ref(false);

const importing = ref(false);

const sourceMessage = ref("");

const batchResult = ref(null);



async function handleSubscribe() {

  if (!remoteUrl.value.trim()) return;

  subscribing.value = true;

  sourceMessage.value = "";

  batchResult.value = null;

  try {

    batchResult.value = await bookSourceStore.subscribe(remoteUrl.value);

    sourceMessage.value = `仓库订阅完成：${batchResult.value.names.length} 个书源可用`;

    remoteUrl.value = "";

  } catch (err) {

    bookSourceStore.error = String(err);

  } finally {

    subscribing.value = false;

  }

}



async function handleSync(subId) {

  sourceMessage.value = "";

  try {

    const result = await bookSourceStore.syncSubscription(subId);

    sourceMessage.value = `同步完成：导入 ${result.imported} · 更新 ${result.updated}`;

  } catch {

    /* error 已在 store 中设置 */

  }

}



async function handleImportJson() {

  if (!jsonText.value.trim()) return;

  importing.value = true;

  sourceMessage.value = "";

  batchResult.value = null;

  try {

    batchResult.value = await importBookSourcesBatch(jsonText.value);

    await bookSourceStore.loadSources();

    jsonText.value = "";

    sourceMessage.value = `已导入 ${batchResult.value.imported} 条，更新 ${batchResult.value.updated} 条`;

  } catch (err) {

    bookSourceStore.error = String(err);

  } finally {

    importing.value = false;

  }

}

</script>



<style scoped>

.card-inner {

  background: var(--color-bg);

  border-radius: var(--radius-sm);

  padding: 14px 16px;

  margin-bottom: 20px;

}



.repo-desc {

  margin-bottom: 12px;

}



.subscription-list {

  list-style: none;

  margin: 16px 0 0;

  padding: 0;

  display: flex;

  flex-direction: column;

  gap: 10px;

}



.subscription-item {

  display: flex;

  align-items: center;

  justify-content: space-between;

  gap: 12px;

  padding: 10px 12px;

  border: 1px solid var(--color-border-light);

  border-radius: var(--radius-sm);

  background: var(--color-surface);

}



.subscription-info {

  flex: 1;

  min-width: 0;

  display: flex;

  flex-direction: column;

  gap: 2px;

}



.subscription-label {

  font-size: 0.85rem;

  font-weight: 600;

}



.subscription-url {

  font-size: 0.75rem;

  color: var(--color-muted);

  overflow: hidden;

  text-overflow: ellipsis;

  white-space: nowrap;

}



.subscription-meta {

  font-size: 0.72rem;

  color: var(--color-muted);

}



.managed-section {

  margin-top: 24px;

  padding-top: 20px;

  border-top: 1px solid var(--color-border-light);

}



.section-count {

  font-weight: 400;

  font-size: 0.78rem;

  color: var(--color-muted);

  margin-left: 8px;

}



.import-section {

  margin-top: 20px;

  padding-top: 16px;

  border-top: 1px solid var(--color-border-light);

}



.error-list {

  margin: 8px 0 0;

  padding-left: 18px;

  font-size: 0.8rem;

  color: var(--color-muted);

}



.source-msg {

  margin-top: 12px;

}

.form-field--full {
  width: 100%;
}

</style>


