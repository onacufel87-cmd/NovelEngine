<template>
  <section class="card local-import">
    <h3 class="card-title">导入本地书籍</h3>
    <p class="card-desc">支持 EPUB / TXT，导入后离线阅读。</p>
    <BaseButton :disabled="importing" @click="handlePickFile">
      {{ importing ? "导入中…" : "选择文件" }}
    </BaseButton>
    <p v-if="message" class="msg msg--ok" style="margin-top: 12px">{{ message }}</p>
    <p v-if="error" class="msg msg--error" style="margin-top: 12px">{{ error }}</p>
  </section>
</template>

<script setup>
import { ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { useShelfStore } from "../../stores/shelfStore";
import BaseButton from "../Common/BaseButton.vue";

const shelfStore = useShelfStore();
const router = useRouter();
const importing = ref(false);
const message = ref("");
const error = ref("");

async function handlePickFile() {
  error.value = "";
  message.value = "";

  const selected = await open({
    multiple: false,
    filters: [
      { name: "EPUB 电子书", extensions: ["epub"] },
      { name: "纯文本", extensions: ["txt"] },
    ],
  });

  if (!selected || Array.isArray(selected)) return;

  importing.value = true;
  try {
    const book = await shelfStore.importLocal(selected);
    message.value = `已导入：《${book.title}》（${book.chapter_count} 章）`;
    router.push(`/read/${book.id}`);
  } catch (err) {
    error.value = String(err);
  } finally {
    importing.value = false;
  }
}
</script>
