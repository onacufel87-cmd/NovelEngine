<template>
  <div class="chapter-comments-wrap">
    <p v-if="comments.length" class="chapter-comment-banner">
      本章 {{ comments.length }} 条评论
    </p>

    <div
      ref="rootRef"
      class="chapter-text chapter-text--comments"
      @mouseup="onMouseUp"
      @click="onClick"
    >
      <div v-for="(para, pIdx) in paragraphs" :key="pIdx" class="para-row">
        <div class="para-text" :data-line-start="para.lineStart">
          <template v-if="para.text">
            <template v-for="(seg, sIdx) in para.segments" :key="sIdx">
              <span v-if="seg.type === 'text'">{{ seg.text }}</span>
              <span
                v-else
                class="text-comment"
                :class="{
                  'text-comment--empty': !seg.hasBody,
                  'text-comment--active': seg.commentId === activeCommentId,
                }"
                :data-note-id="seg.commentId"
                :title="seg.hasBody ? '查看评论' : '待补充评论'"
              >{{ seg.text }}</span>
            </template>
          </template>
          <br v-else />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from "vue";
import { buildHighlightSegments, notesStartingInLine } from "../../utils/textAnchor";

const props = defineProps({
  plainText: { type: String, required: true },
  comments: { type: Array, default: () => [] },
  activeCommentId: { type: Number, default: null },
});

const emit = defineEmits(["selection-change", "comment-click"]);

const rootRef = ref(null);

const paragraphs = computed(() => {
  const lines = props.plainText.split("\n");
  let offset = 0;
  const result = [];

  for (const line of lines) {
    const lineStart = offset;
    const lineEnd = offset + line.length;
    const lineNotes = notesStartingInLine(props.comments, lineStart, lineEnd);
    const relativeNotes = lineNotes.map((n) => ({
      ...n,
      start_offset: n.start_offset - lineStart,
      end_offset: n.end_offset - lineStart,
    }));
    const segments = buildHighlightSegments(line, relativeNotes).map((seg) =>
      seg.type === "highlight"
        ? { ...seg, type: "comment", commentId: seg.noteId }
        : seg
    );

    result.push({ text: line, segments, lineStart });
    offset = lineEnd + 1;
  }

  return result;
});

function onMouseUp() {
  emit("selection-change", rootRef.value);
}

function onClick(event) {
  const el = event.target.closest?.("[data-note-id]");
  if (!el) return;
  const commentId = Number(el.dataset.noteId);
  if (commentId) emit("comment-click", commentId);
}

defineExpose({ rootEl: rootRef });
</script>

<style scoped>
.chapter-comments-wrap {
  max-width: 720px;
  margin: 0 auto;
}

.chapter-text--comments {
  white-space: pre-wrap;
  text-indent: 2em;
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}

.chapter-comment-banner {
  text-indent: 0;
  font-size: 0.78rem;
  color: var(--color-muted);
  padding: 6px 10px;
  margin: 0 0 16px;
  background: var(--color-primary-soft, rgba(212, 163, 115, 0.12));
  border-radius: var(--radius-sm);
  border: 1px solid var(--reader-border, var(--color-border));
}

.para-row {
  margin-bottom: 0.35em;
}

.text-comment {
  color: inherit;
  cursor: pointer;
  border-bottom: 2px solid var(--color-primary);
  padding-bottom: 1px;
}

.text-comment--empty {
  border-bottom-style: dashed;
  opacity: 0.88;
}

.text-comment--active {
  background: var(--color-primary-soft, rgba(212, 163, 115, 0.15));
  border-radius: 2px;
}
</style>
