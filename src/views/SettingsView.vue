<template>
  <section class="settings-view page">
    <header class="page-header">
      <h2 class="page-title">设置</h2>
      <p class="page-desc">阅读外观、简繁体、正文清洗与网络抓取（Cookie / 限速）。</p>
    </header>
    <div class="card">
      <SettingPanel mode="page" />
    </div>
  </section>
</template>

<script setup>
import SettingPanel from "../components/Reader/SettingPanel.vue";
</script>

<style scoped>
.settings-view :deep(.setting-panel--page) {
  border: none;
  padding: 0;
  box-shadow: none;
}

.settings-view :deep(.setting-panel h3) {
  display: none;
}
</style>
