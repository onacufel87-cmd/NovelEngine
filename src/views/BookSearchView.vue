<template>

  <section

    class="book-search-view page-stack"

    :class="{ 'book-search-view--embedded': embedded }"

  >

    <header v-if="!embedded" class="page-header">

      <h2 class="page-title">搜索</h2>

      <p class="page-desc">

        在已启用书源中并行搜索，默认含维基文库、Open Library 等公版源。

      </p>

    </header>



    <!-- 大圆角搜索栏 -->

    <form class="search-bar" @submit.prevent="handleSearch">

      <span class="search-bar__icon" aria-hidden="true">

        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">

          <circle cx="11" cy="11" r="8" />

          <path d="m21 21-4.3-4.3" />

        </svg>

      </span>

      <input

        v-model="keyword"

        type="search"

        required

        class="search-bar__input"

        placeholder="搜索书名、作者…"

        autocomplete="off"

      />

      <BaseButton type="submit" class="search-bar__btn" :disabled="bookSourceStore.loadingSearch">

        {{ bookSourceStore.loadingSearch ? "搜索中…" : "搜索" }}

      </BaseButton>

    </form>



    <!-- 热门词快捷入口 -->

    <div class="hot-keywords">

      <button

        v-for="kw in hotKeywords"

        :key="kw"

        type="button"

        class="hot-kw"

        @click="quickSearch(kw)"

      >

        {{ kw }}

      </button>

    </div>



    <p v-if="bookSourceStore.error" class="msg msg--error">{{ bookSourceStore.error }}</p>



    <div

      v-if="bookSourceStore.hasSearched && !bookSourceStore.loadingSearch && !bookSourceStore.searchResults.length && !bookSourceStore.error"

      class="card empty-results"

    >

      <h3 class="card-title">未找到相关书籍</h3>

      <p class="card-desc">未检测到可用结果，试试更换关键词，或确认书源已启用。</p>

      <ul class="tips-list">

        <li>中文：<code>红楼梦</code>、<code>西游记</code></li>

        <li>英文：<code>Alice</code>、<code>Pride</code></li>

        <li>前往 <RouterLink to="/import">书源页</RouterLink> 启用公版源</li>

      </ul>

    </div>



    <div v-if="bookSourceStore.searchResults.length" class="card">

      <h3 class="card-title">搜索结果 · {{ bookSourceStore.searchResults.length }}</h3>

      <ul class="result-list">

        <li v-for="(item, idx) in bookSourceStore.searchResults" :key="idx" class="result-item">

          <div class="result-item__main">

            <strong class="result-title">{{ item.title }}</strong>

            <p class="result-meta">

              <span v-if="item.author">{{ item.author }}</span>

              <span class="dot" v-if="item.author">·</span>

              <span class="source-tag">{{ item.source_name }}</span>

            </p>

          </div>

          <div class="result-item__actions">

            <button

              type="button"

              class="btn-ghost"

              :disabled="previewingId === idx"

              @click="openPreview(item, idx)"

            >

              试读

            </button>

            <BaseButton :disabled="addingId === idx" @click="handleAdd(item, idx)">

              {{ addingId === idx ? "解析中…" : "加入书架" }}

            </BaseButton>

          </div>

        </li>

      </ul>

      <p v-if="shelfMessage" class="msg msg--ok" style="margin-top: 12px">{{ shelfMessage }}</p>

      <RouterLink v-if="addedBookId" :to="`/read/${addedBookId}`" class="read-link">

        前往阅读 →

      </RouterLink>

    </div>



    <BookPreviewPanel

      :visible="previewVisible"

      :book="previewBook"

      @close="closePreview"

    />

  </section>

</template>



<script setup>

import { onMounted, ref } from "vue";

import { useBookSourceStore } from "../stores/bookSourceStore";

import { useShelfStore } from "../stores/shelfStore";

import { formatAppError } from "../utils/appError";

import BookPreviewPanel from "../components/Source/BookPreviewPanel.vue";

import BaseButton from "../components/Common/BaseButton.vue";



defineProps({

  /** 嵌入发现页时隐藏页头 */

  embedded: { type: Boolean, default: false },

});



const bookSourceStore = useBookSourceStore();

const shelfStore = useShelfStore();

const keyword = ref("");

const addingId = ref(null);

const previewingId = ref(null);

const previewVisible = ref(false);

const previewBook = ref(null);

const shelfMessage = ref("");

const addedBookId = ref(null);



const hotKeywords = ["红楼梦", "西游记", "Alice", "科幻"];



onMounted(() => {

  bookSourceStore.loadSources();

});



async function handleSearch() {

  shelfMessage.value = "";

  addedBookId.value = null;

  closePreview();

  await bookSourceStore.search(keyword.value);

}



function quickSearch(kw) {

  keyword.value = kw;

  handleSearch();

}



function openPreview(item, idx) {

  previewingId.value = idx;

  previewBook.value = item;

  previewVisible.value = true;

  previewingId.value = null;

}



function closePreview() {

  previewVisible.value = false;

  previewBook.value = null;

}



async function handleAdd(item, idx) {

  addingId.value = idx;

  shelfMessage.value = "";

  addedBookId.value = null;



  try {

    const book = await shelfStore.addBookFromSearchResult({

      title: item.title,

      author: item.author,

      catalogUrl: item.catalog_url,

      sourceId: item.source_id,

    });

    shelfMessage.value = `已加入书架：《${book.title}》（${book.chapter_count} 章）`;

    addedBookId.value = book.id;

  } catch (err) {

    bookSourceStore.error = formatAppError(err);

  } finally {

    addingId.value = null;

  }

}

</script>



<style scoped>

/* 嵌入发现页时仍保留垂直间距 */
.book-search-view--embedded {

  gap: 16px;

}



.search-bar {

  display: flex;

  align-items: center;

  gap: 10px;

  padding: 8px 8px 8px 16px;

  background: var(--color-surface);

  border: 1px solid var(--color-border-light);

  border-radius: var(--radius-pill);

  box-shadow: var(--shadow-sm);

  transition: box-shadow 0.15s, border-color 0.15s;

}



.search-bar:focus-within {

  border-color: var(--color-primary);

  box-shadow: var(--shadow-md), 0 0 0 3px var(--color-primary-soft);

}



.search-bar__icon {

  color: var(--color-muted);

  display: flex;

}



.search-bar__icon svg {

  width: 18px;

  height: 18px;

}



.search-bar__input {

  flex: 1;

  border: none !important;

  background: transparent !important;

  padding: 8px 0 !important;

  box-shadow: none !important;

  font-size: 0.95rem;

}



.search-bar__input:focus {

  outline: none;

  box-shadow: none !important;

}



.search-bar__btn {

  flex-shrink: 0;

}



.hot-keywords {

  display: flex;

  flex-wrap: wrap;

  gap: 8px;

  margin-top: 4px;

}



.hot-kw {

  padding: 4px 12px;

  border: 1px solid var(--color-border-light);

  border-radius: var(--radius-pill);

  background: var(--color-surface);

  color: var(--color-muted);

  font-size: 0.78rem;

  cursor: pointer;

  transition: background 0.15s, color 0.15s, border-color 0.15s;

}



.hot-kw:hover {

  background: var(--color-primary-soft);

  color: var(--color-primary);

  border-color: transparent;

}



.result-list {

  list-style: none;

}



.result-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--color-border-light);
}

.result-item:last-child {
  border-bottom: none;
}

.result-item__main {
  flex: 1;
  min-width: 0;
}

.result-item__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding-top: 2px;
}

.result-title {
  font-size: 0.95rem;
  font-weight: 600;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.result-meta {
  margin: 4px 0 0;
  font-size: 0.82rem;
  color: var(--color-muted);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}



.source-tag {

  color: var(--color-primary);

  opacity: 0.9;

}



.dot {

  opacity: 0.5;

}



.read-link {

  display: inline-block;

  margin-top: 10px;

  font-size: 0.88rem;

}



.tips-list {

  margin: 0;

  padding-left: 18px;

  color: var(--color-muted);

  font-size: 0.86rem;

  line-height: 1.9;

}



.empty-results .card-title {

  margin-bottom: 8px;

}

</style>

