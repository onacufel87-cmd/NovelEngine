<template>
  <section class="comment-list page page--wide page-stack">
    <header class="page-header">
      <h2 class="page-title">我的评论</h2>
      <p class="page-desc">共 {{ commentStore.totalCount }} 条评论，按书籍分组管理。</p>
    </header>

    <div class="search-row">
      <input
        v-model="keyword"
        type="search"
        class="search-input"
        placeholder="搜索评论内容或原文摘录…"
      />
    </div>

    <p v-if="commentStore.error" class="msg msg--error">{{ commentStore.error }}</p>
    <LoadingSpinner v-if="commentStore.loading" text="加载评论中…" />

    <div v-else-if="filteredGroups.length === 0" class="card empty-state">
      <p class="empty-title">还没有评论</p>
      <p class="empty-hint">在阅读页划词选中文字，点击「评论」即可添加</p>
      <RouterLink to="/" class="empty-cta">去书架选书 →</RouterLink>
    </div>

    <div v-else class="comment-groups">
      <section
        v-for="group in filteredGroups"
        :key="group.book_id"
        class="book-group card"
      >
        <button
          type="button"
          class="group-header"
          :aria-expanded="isExpanded(group.book_id)"
          @click="toggleBook(group.book_id)"
        >
          <span class="group-title">{{ group.book_title }}</span>
          <span class="group-count">{{ group.notes.length }} 条</span>
        </button>

        <ul v-show="isExpanded(group.book_id)" class="comment-items">
          <li v-for="item in group.notes" :key="item.note.id" class="comment-item">
            <div class="comment-item__main">
              <p class="comment-chapter">{{ item.chapter_title }}</p>
              <blockquote v-if="item.note.quote" class="comment-quote">
                {{ item.note.quote }}
              </blockquote>
              <p
                class="comment-body"
                :class="{ 'comment-body--empty': !item.note.body?.trim() }"
              >
                {{ item.note.body?.trim() || "（已标记，暂无评论内容）" }}
              </p>
              <time class="comment-time">{{ formatTime(item.note.updated_at) }}</time>
            </div>
            <div class="comment-item__actions">
              <button type="button" class="action-btn" @click="jumpToComment(item)">
                跳转阅读
              </button>
              <button type="button" class="action-btn" @click="editComment(item)">编辑</button>
              <button type="button" class="action-btn action-btn--danger" @click="removeComment(item.note.id)">
                删除
              </button>
            </div>
          </li>
        </ul>
      </section>
    </div>

    <CommentEditor
      layout="modal"
      :visible="editor.visible"
      :quote="editor.quote"
      :initial-body="editor.body"
      :is-new="false"
      @close="editor.visible = false"
      @save="saveEdit"
    />
  </section>
</template>

<script setup>
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useCommentStore } from "../stores/commentStore";
import { formatAppError } from "../utils/appError";
import CommentEditor from "../components/Reader/CommentEditor.vue";
import LoadingSpinner from "../components/Common/LoadingSpinner.vue";

const route = useRoute();
const router = useRouter();
const commentStore = useCommentStore();

const keyword = ref("");
const expandedBookId = ref(null);
const editor = ref({
  visible: false,
  noteId: null,
  quote: "",
  body: "",
});

onMounted(async () => {
  await commentStore.loadGroupedComments();
  const bookQuery = Number(route.query.book);
  if (bookQuery) expandedBookId.value = bookQuery;
});

const filteredGroups = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return commentStore.groupedComments;

  return commentStore.groupedComments
    .map((group) => ({
      ...group,
      notes: group.notes.filter((item) => {
        const hay = [
          item.note.body,
          item.note.quote,
          item.chapter_title,
          item.book_title,
        ]
          .filter(Boolean)
          .join(" ")
          .toLowerCase();
        return hay.includes(kw);
      }),
    }))
    .filter((g) => g.notes.length > 0);
});

function isExpanded(bookId) {
  if (filteredGroups.value.length === 1) return true;
  return expandedBookId.value === bookId;
}

function toggleBook(bookId) {
  expandedBookId.value = expandedBookId.value === bookId ? null : bookId;
}

function formatTime(ts) {
  if (!ts) return "";
  return new Date(ts * 1000).toLocaleString("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function jumpToComment(item) {
  router.push({
    path: `/read/${item.note.book_id}`,
    query: {
      chapter: String(item.note.chapter_id),
      note: String(item.note.id),
    },
  });
}

function editComment(item) {
  editor.value = {
    visible: true,
    noteId: item.note.id,
    quote: item.note.quote || "",
    body: item.note.body || "",
  };
}

async function saveEdit(body) {
  try {
    await commentStore.saveCommentBody(editor.value.noteId, body);
    editor.value.visible = false;
    await commentStore.loadGroupedComments();
  } catch (err) {
    commentStore.error = formatAppError(err);
  }
}

async function removeComment(noteId) {
  if (!confirm("确定删除这条评论？")) return;
  try {
    await commentStore.removeComment(noteId);
  } catch (err) {
    commentStore.error = formatAppError(err);
  }
}
</script>

<style scoped>
.search-row {
  margin-bottom: 4px;
}

.search-input {
  width: 100%;
  max-width: 420px;
}

.comment-groups {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.book-group {
  padding: 0;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 16px 20px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  text-align: left;
}

.group-title {
  font-weight: 600;
  font-size: 0.98rem;
}

.group-count {
  font-size: 0.78rem;
  color: var(--color-muted);
  background: var(--color-bg);
  padding: 2px 10px;
  border-radius: var(--radius-pill);
}

.comment-items {
  list-style: none;
  border-top: 1px solid var(--color-border-light);
  padding-bottom: 12px;
}

.comment-item {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-light);
}

.comment-item:last-child {
  border-bottom: none;
  padding-bottom: 20px;
}

.comment-item__main {
  flex: 1;
  min-width: 0;
}

.comment-chapter {
  font-size: 0.78rem;
  color: var(--color-primary);
  margin-bottom: 8px;
  font-weight: 600;
}

.comment-quote {
  margin: 0 0 10px;
  padding: 10px 12px;
  font-size: 0.84rem;
  color: var(--color-muted);
  background: var(--color-bg);
  border-left: 3px solid var(--color-primary);
  border-radius: var(--radius-sm);
  line-height: 1.65;
}

.comment-quote::before {
  content: "「";
}

.comment-quote::after {
  content: "」";
}

.comment-body {
  margin: 0 0 8px;
  padding: 8px 10px;
  font-size: 0.92rem;
  font-weight: 500;
  line-height: 1.65;
  color: var(--color-text);
  background: var(--color-primary-soft, rgba(212, 163, 115, 0.1));
  border-radius: var(--radius-sm);
  white-space: pre-wrap;
}

.comment-body--empty {
  color: var(--color-muted);
  font-weight: 400;
  font-style: italic;
  font-size: 0.82rem;
  background: transparent;
  padding-left: 0;
}

.comment-time {
  display: block;
  font-size: 0.72rem;
  color: var(--color-muted);
  margin-top: 4px;
}

.comment-item__actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
  width: 88px;
}

.action-btn {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-muted);
  font-size: 0.78rem;
  line-height: 1.2;
  text-align: center;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.action-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
  border-color: var(--color-border);
}

.action-btn--danger {
  color: var(--color-danger);
  border-color: transparent;
  background: transparent;
}

.action-btn--danger:hover {
  background: var(--color-danger-bg);
  color: var(--color-danger);
  border-color: transparent;
}

.empty-title {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.empty-cta {
  display: inline-block;
  margin-top: 12px;
}
</style>
