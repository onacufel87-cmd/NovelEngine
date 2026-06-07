<template>
  <div class="json-editor" :class="{ 'json-editor--invalid': !!errorMsg }">
    <div v-if="showToolbar" class="json-editor__toolbar">
      <button type="button" class="tool-btn" @click="formatJson">格式化</button>
      <button type="button" class="tool-btn" @click="minifyJson">压缩</button>
      <button
        type="button"
        class="tool-btn tool-btn--primary"
        :disabled="validating"
        @click="validateJson"
      >
        {{ validating ? "校验中…" : "校验规则" }}
      </button>
      <span v-if="validOk" class="valid-badge">✓ 规则有效</span>
    </div>
    <div ref="hostRef" class="json-editor__host" :style="{ minHeight }" />
    <p v-if="errorMsg" class="json-editor__error">{{ errorMsg }}</p>
    <p v-else-if="hint" class="json-editor__hint">{{ hint }}</p>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { EditorState } from "@codemirror/state";
import {
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
  placeholder as cmPlaceholder,
} from "@codemirror/view";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter, lintGutter } from "@codemirror/lint";
import { defaultKeymap, indentWithTab } from "@codemirror/commands";
import { validateSourceRule } from "../../api";

const props = defineProps({
  modelValue: { type: String, default: "" },
  placeholder: {
    type: String,
    default: '{"name":"我的书源","chapter_list_selector":"#list a",...}',
  },
  minHeight: { type: String, default: "180px" },
  readonly: { type: Boolean, default: false },
  showToolbar: { type: Boolean, default: true },
  /** 是否调用 Rust validate_source_rule 做书源 schema 校验 */
  validateRule: { type: Boolean, default: true },
  hint: { type: String, default: "Ctrl+S 快速校验 · Tab 缩进" },
});

const emit = defineEmits(["update:modelValue", "valid", "invalid"]);

const hostRef = ref(null);
const viewRef = shallowRef(null);
const errorMsg = ref("");
const validOk = ref(false);
const validating = ref(false);

/** 暖纸主题，与应用 CSS 变量一致 */
const editorTheme = EditorView.theme({
  "&": {
    fontSize: "0.84rem",
    backgroundColor: "var(--color-surface)",
    color: "var(--color-text)",
    borderRadius: "var(--radius-sm)",
  },
  ".cm-scroller": {
    fontFamily: '"Cascadia Code", "Fira Code", Consolas, monospace',
    lineHeight: "1.55",
  },
  ".cm-gutters": {
    backgroundColor: "var(--color-bg)",
    color: "var(--color-muted)",
    border: "none",
    borderRight: "1px solid var(--color-border-light)",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "var(--color-primary-soft)",
  },
  ".cm-activeLine": {
    backgroundColor: "rgba(212, 163, 115, 0.08)",
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
    backgroundColor: "rgba(212, 163, 115, 0.25) !important",
  },
  ".cm-cursor": {
    borderLeftColor: "var(--color-primary)",
  },
  ".cm-lintRange-error": {
    backgroundImage: "none",
    textDecoration: "underline wavy var(--color-danger)",
  },
  "&.cm-focused": {
    outline: "2px solid var(--color-primary-soft)",
    outlineOffset: "-1px",
  },
});

/** 从编辑器读取全文 */
function getText() {
  return viewRef.value?.state.doc.toString() ?? props.modelValue;
}

/** 写入编辑器内容并同步 v-model */
function setText(text) {
  if (!viewRef.value) return;
  const current = viewRef.value.state.doc.toString();
  if (current === text) return;
  viewRef.value.dispatch({
    changes: { from: 0, to: viewRef.value.state.doc.length, insert: text },
  });
  emit("update:modelValue", text);
}

/** 格式化 JSON（2 空格缩进） */
function formatJson() {
  errorMsg.value = "";
  validOk.value = false;
  try {
    const parsed = JSON.parse(getText());
    setText(JSON.stringify(parsed, null, 2));
  } catch (err) {
    errorMsg.value = `JSON 语法错误: ${err.message}`;
    emit("invalid", err);
  }
}

/** 压缩为单行 JSON */
function minifyJson() {
  errorMsg.value = "";
  validOk.value = false;
  try {
    const parsed = JSON.parse(getText());
    setText(JSON.stringify(parsed));
  } catch (err) {
    errorMsg.value = `JSON 语法错误: ${err.message}`;
    emit("invalid", err);
  }
}

/** 本地语法 + 可选后端书源规则校验 */
async function validateJson() {
  errorMsg.value = "";
  validOk.value = false;
  validating.value = true;
  const text = getText().trim();
  if (!text) {
    errorMsg.value = "请输入书源 JSON";
    validating.value = false;
    emit("invalid", new Error("empty"));
    return false;
  }
  try {
    const rule = JSON.parse(text);
    if (props.validateRule) {
      const msg = await validateSourceRule(rule);
      validOk.value = true;
      emit("valid", { rule, message: msg });
    } else {
      validOk.value = true;
      emit("valid", { rule, message: "JSON 语法有效" });
    }
    return true;
  } catch (err) {
    errorMsg.value = String(err.message || err);
    emit("invalid", err);
    return false;
  } finally {
    validating.value = false;
  }
}

onMounted(() => {
  if (!hostRef.value) return;

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      lintGutter(),
      json(),
      linter(jsonParseLinter()),
      editorTheme,
      cmPlaceholder(props.placeholder),
      EditorView.editable.of(!props.readonly),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          validOk.value = false;
          errorMsg.value = "";
          emit("update:modelValue", update.state.doc.toString());
        }
      }),
      keymap.of([
        ...defaultKeymap,
        indentWithTab,
        {
          key: "Mod-s",
          preventDefault: true,
          run: () => {
            validateJson();
            return true;
          },
        },
      ]),
    ],
  });

  viewRef.value = new EditorView({ state, parent: hostRef.value });
});

onBeforeUnmount(() => {
  viewRef.value?.destroy();
  viewRef.value = null;
});

watch(
  () => props.modelValue,
  (val) => {
    if (viewRef.value && val !== viewRef.value.state.doc.toString()) {
      setText(val);
    }
  }
);

defineExpose({ formatJson, minifyJson, validateJson, getText, setText });
</script>

<style scoped>
.json-editor {
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--color-surface);
}

.json-editor--invalid {
  border-color: var(--color-danger);
}

.json-editor__toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--color-border-light);
  background: var(--color-bg);
}

.tool-btn {
  padding: 4px 12px;
  font-size: 0.78rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-pill);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
}

.tool-btn:hover {
  background: var(--color-hover);
}

.tool-btn--primary {
  color: var(--color-primary);
  border-color: var(--color-primary-soft);
  background: var(--color-primary-soft);
}

.tool-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.valid-badge {
  font-size: 0.78rem;
  color: var(--color-success);
  margin-left: 4px;
}

.json-editor__host {
  text-align: left;
}

.json-editor__host :deep(.cm-editor) {
  min-height: inherit;
}

.json-editor__host :deep(.cm-content) {
  min-height: inherit;
  padding: 8px 0;
}

.json-editor__error {
  margin: 0;
  padding: 8px 12px;
  font-size: 0.78rem;
  color: var(--color-danger);
  background: var(--color-danger-bg);
  border-top: 1px solid var(--color-danger-bg);
}

.json-editor__hint {
  margin: 0;
  padding: 6px 12px;
  font-size: 0.72rem;
  color: var(--color-muted);
  border-top: 1px solid var(--color-border-light);
}
</style>
