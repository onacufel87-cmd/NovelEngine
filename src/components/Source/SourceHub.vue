<template>
  <section class="source-hub page-stack">
    <div class="hub-tabs">
      <button
        type="button"
        class="hub-tab"
        :class="{ active: tab === 'repo' }"
        @click="switchTab('repo')"
      >
        书源仓库
      </button>
      <button
        type="button"
        class="hub-tab"
        :class="{ active: tab === 'oneclick' }"
        @click="switchTab('oneclick')"
      >
        一键接入
      </button>
      <button
        type="button"
        class="hub-tab"
        :class="{ active: tab === 'manual' }"
        @click="switchTab('manual')"
      >
        手动高级
      </button>
    </div>

    <SourceManagerPanel v-show="tab === 'repo'" />

    <div v-show="tab === 'oneclick'" class="tab-panel">
      <WizardStepBar :step="wizard.step" />
      <div v-if="wizard.detecting" class="wizard-loading card">
        <LoadingSpinner :text="wizard.loadingHint" />
        <p class="wizard-loading__sub">Rust 后台正在启发式解析页面结构…</p>
      </div>
      <OneClickAuto
        @fallback="switchTab('manual')"
        @detect-start="wizard.startDetect()"
        @detect-complete="onDetectComplete"
        @detect-fail="wizard.failDetect()"
        @trial-start="onTrialStart"
      />
      <DetectLogConsole
        :logs="wizard.detectLogs"
        :failed="wizard.detectFailed"
      />
      <Transition name="wizard-slide">
        <ParseSessionPanel
          v-if="wizard.step >= 2"
          ref="oneclickSessionRef"
          :show-json-panel="false"
          :initial-catalog-url="wizard.catalogUrl"
        />
      </Transition>
    </div>

    <div v-show="tab === 'manual'" class="tab-panel">
      <WizardStepBar :step="wizard.step" />
      <div v-if="wizard.detecting" class="wizard-loading card">
        <LoadingSpinner :text="wizard.loadingHint" />
        <p class="wizard-loading__sub">正在对比目录页与正文页 DOM 结构…</p>
      </div>
      <AutoDetectPanel
        @detect-start="wizard.startDetect()"
        @detect-complete="onDetectComplete"
        @detect-fail="wizard.failDetect()"
        @trial-start="onTrialStart"
      />
      <DetectLogConsole
        :logs="wizard.detectLogs"
        :failed="wizard.detectFailed"
      />
      <Transition name="wizard-slide">
        <ParseSessionPanel
          v-if="wizard.step >= 2"
          ref="manualSessionRef"
          :show-json-panel="true"
          :initial-catalog-url="wizard.catalogUrl"
        />
      </Transition>
    </div>
  </section>
</template>

<script setup>
import { ref, watch, nextTick } from "vue";
import { useSourceWizardStore } from "../../stores/sourceWizardStore";
import SourceManagerPanel from "./SourceManagerPanel.vue";
import WizardStepBar from "./WizardStepBar.vue";
import OneClickAuto from "./OneClickAuto.vue";
import AutoDetectPanel from "./AutoDetectPanel.vue";
import ParseSessionPanel from "./ParseSessionPanel.vue";
import DetectLogConsole from "./DetectLogConsole.vue";
import LoadingSpinner from "../Common/LoadingSpinner.vue";

const tab = ref("oneclick");
const wizard = useSourceWizardStore();
const oneclickSessionRef = ref(null);
const manualSessionRef = ref(null);

function switchTab(next) {
  tab.value = next;
  wizard.reset();
}

function onDetectComplete(payload) {
  wizard.completeDetect(payload?.catalogUrl || "");
}

async function onTrialStart(payload) {
  const url = payload?.catalogUrl || wizard.catalogUrl;
  wizard.startTrial(url);
  await nextTick();
  const panel =
    tab.value === "oneclick" ? oneclickSessionRef.value : manualSessionRef.value;
  panel?.setCatalogUrl(url);
}

watch(
  () => wizard.catalogUrl,
  async (url) => {
    if (wizard.step < 2 || !url) return;
    await nextTick();
    const panel =
      tab.value === "oneclick" ? oneclickSessionRef.value : manualSessionRef.value;
    panel?.setCatalogUrl(url);
  }
);
</script>

<style scoped>
.hub-tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--color-bg);
  border-radius: var(--radius-md);
  width: fit-content;
  margin-bottom: 4px;
}

.hub-tab {
  padding: 8px 20px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-muted);
  font-size: 0.9rem;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.hub-tab.active {
  background: var(--color-surface);
  color: var(--color-primary);
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}

.tab-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.wizard-loading {
  text-align: center;
  padding: 28px 20px;
}

.wizard-loading__sub {
  margin-top: 8px;
  font-size: 0.78rem;
  color: var(--color-muted);
}

.wizard-slide-enter-active {
  transition: opacity 0.28s ease, transform 0.28s ease;
}

.wizard-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.wizard-slide-enter-from {
  opacity: 0;
  transform: translateY(16px);
}

.wizard-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

@media (prefers-reduced-motion: reduce) {
  .wizard-slide-enter-active,
  .wizard-slide-leave-active {
    transition: none;
  }
  .wizard-slide-enter-from,
  .wizard-slide-leave-to {
    transform: none;
  }
}
</style>
