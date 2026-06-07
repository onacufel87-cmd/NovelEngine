<template>



  <section class="shelf-view page page--wide page-stack">



    <header class="page-header shelf-header">



      <div>



        <h2 class="page-title">我的书架</h2>



        <p class="page-desc">本地阅读，数据仅存于你的电脑。</p>



      </div>



      <div class="header-actions">



        <!-- 列表 / 网格切换 -->

        <div class="view-toggle" role="group" aria-label="书架视图">

          <button

            type="button"

            class="view-btn"

            :class="{ active: settingStore.shelfViewMode === 'grid' }"

            title="网格视图"

            @click="setViewMode('grid')"

          >

            ⊞

          </button>

          <button

            type="button"

            class="view-btn"

            :class="{ active: settingStore.shelfViewMode === 'list' }"

            title="列表视图"

            @click="setViewMode('list')"

          >

            ☰

          </button>

        </div>



        <button type="button" class="btn-ghost" @click="shelfStore.loadFromDB">刷新</button>



        <RouterLink to="/discover" class="add-link">+ 发现书籍</RouterLink>



      </div>



    </header>







    <LocalImportPanel />







    <p v-if="shelfStore.error" class="msg msg--error">{{ shelfStore.error }}</p>







    <LoadingSpinner v-if="shelfStore.loading" text="加载书架中…" />







    <div v-else-if="shelfStore.books.length === 0" class="card empty-state">



      <p class="empty-title">书架空空如也</p>



      <p class="empty-hint">导入 EPUB/TXT，或从发现页搜索公版书</p>



      <RouterLink to="/discover" class="empty-cta">去发现 →</RouterLink>



    </div>







    <div

      v-else

      class="book-collection"

      :class="settingStore.shelfViewMode === 'list' ? 'book-collection--list' : 'book-collection--grid'"

    >



      <template v-if="settingStore.shelfViewMode === 'list'">

        <BookListRow v-for="book in shelfStore.books" :key="book.id" :book="book" />

      </template>

      <template v-else>

        <BookCard v-for="book in shelfStore.books" :key="book.id" :book="book" />

      </template>



    </div>



  </section>



</template>







<script setup>



import { onMounted } from "vue";



import { useShelfStore } from "../stores/shelfStore";



import { useSettingStore } from "../stores/settingStore";



import BookCard from "../components/Shelf/BookCard.vue";



import BookListRow from "../components/Shelf/BookListRow.vue";



import LocalImportPanel from "../components/Shelf/LocalImportPanel.vue";



import LoadingSpinner from "../components/Common/LoadingSpinner.vue";







const shelfStore = useShelfStore();



const settingStore = useSettingStore();







onMounted(() => {



  shelfStore.loadFromDB();



});







/** 切换书架视图并持久化 */

function setViewMode(mode) {

  settingStore.updateSetting("shelfViewMode", mode);

}



</script>







<style scoped>



.shelf-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.shelf-header {



  display: flex;



  align-items: flex-start;



  justify-content: space-between;



  gap: 16px;



  flex-wrap: wrap;



}







.header-actions {



  display: flex;



  align-items: center;



  gap: 12px;



  flex-shrink: 0;



  flex-wrap: wrap;



  padding-top: 4px;



  margin-left: auto;



}







.view-toggle {



  display: flex;



  border: 1px solid var(--color-border-light);



  border-radius: var(--radius-pill);



  overflow: hidden;



  background: var(--color-bg);



}







.view-btn {



  border: none;



  background: transparent;



  color: var(--color-muted);



  width: 36px;



  height: 32px;



  cursor: pointer;



  font-size: 0.95rem;



  transition: background 0.15s, color 0.15s;



}







.view-btn.active {



  background: var(--color-primary-soft);



  color: var(--color-primary);



}







.view-btn:hover:not(.active) {



  background: var(--color-hover);



}







.add-link {



  color: var(--color-primary);



  text-decoration: none;



  font-size: 0.88rem;



  font-weight: 500;



  white-space: nowrap;



  padding: 8px 16px;



  border-radius: var(--radius-pill);



  background: var(--color-primary-soft);



  transition: background 0.15s;



}







.add-link:hover {



  background: var(--color-hover);



}







.book-collection--grid {
  --shelf-card-width: 108px;
  display: grid;
  width: 100%;
  /* 固定列宽，保证每张卡片尺寸一致 */
  grid-template-columns: repeat(auto-fill, var(--shelf-card-width));
  gap: 14px;
  justify-content: start;
  align-items: start;
}







.book-collection--list {



  display: flex;



  flex-direction: column;



  gap: 10px;



  width: 100%;



}







@media (min-width: 768px) {
  .book-collection--grid {
    --shelf-card-width: 112px;
  }
}







.empty-title {



  font-size: 1rem;



  font-weight: 600;



  margin-bottom: 8px;



  color: var(--color-text);



}







.empty-cta {



  display: inline-block;



  margin-top: 12px;



  font-size: 0.9rem;



  font-weight: 500;



}



</style>


