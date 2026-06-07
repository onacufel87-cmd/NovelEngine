<template>
  <section class="page page--wide page-stack">
    <header class="page-header">
      <h2 class="page-title">书源</h2>
      <p class="page-desc">
        管理公版书源与远程订阅，或粘贴网址一键接入、手动检测选择器。
      </p>
    </header>

    <SourceHub />
  </section>
</template>

<script setup>
import { onMounted } from "vue";
import { useBookSourceStore } from "../stores/bookSourceStore";
import SourceHub from "../components/Source/SourceHub.vue";

const bookSourceStore = useBookSourceStore();

onMounted(() => {
  bookSourceStore.loadSources();
});
</script>
