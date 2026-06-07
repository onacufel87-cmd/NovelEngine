<template>
  <section class="discover-view page page--wide page-stack">
    <header class="page-header">
      <h2 class="page-title">发现</h2>
      <p class="page-desc">搜索公版书库，或浏览书源榜单。</p>
    </header>

    <!-- 发现页 Tab：全网搜 / 榜单 -->
    <div class="discover-tabs">
      <button
        type="button"
        class="discover-tab"
        :class="{ active: tab === 'search' }"
        @click="tab = 'search'"
      >
        全网搜
      </button>
      <button
        type="button"
        class="discover-tab"
        :class="{ active: tab === 'rank' }"
        @click="tab = 'rank'"
      >
        榜单
      </button>
    </div>

    <BookSearchView v-show="tab === 'search'" embedded />
    <RankPanel v-show="tab === 'rank'" />
  </section>
</template>

<script setup>
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import BookSearchView from "./BookSearchView.vue";
import RankPanel from "../components/Discover/RankPanel.vue";

const route = useRoute();
const router = useRouter();
const tab = ref(route.query.tab === "rank" ? "rank" : "search");

watch(tab, (val) => {
  router.replace({ query: val === "rank" ? { tab: "rank" } : {} });
});

watch(
  () => route.query.tab,
  (q) => {
    tab.value = q === "rank" ? "rank" : "search";
  }
);
</script>

<style scoped>
.discover-tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--color-surface);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-pill);
  width: fit-content;
  box-shadow: var(--shadow-sm);
}

.discover-tab {
  padding: 8px 22px;
  border: none;
  border-radius: var(--radius-pill);
  background: transparent;
  color: var(--color-muted);
  font-size: 0.88rem;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, box-shadow 0.15s;
}

.discover-tab.active {
  background: var(--color-primary-soft);
  color: var(--color-primary);
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}
</style>
