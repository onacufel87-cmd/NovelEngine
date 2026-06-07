<template>

  <aside class="setting-panel" :class="{ 'setting-panel--page': mode === 'page' }">

    <h3 v-if="mode === 'page'">阅读与清洗</h3>

    <h3 v-else>设置</h3>



    <section class="group">

      <h4>中文显示</h4>

      <label class="field">

        简繁体

        <select

          :value="settingStore.chineseVariant"

          @change="settingStore.updateSetting('chineseVariant', $event.target.value)"

        >

          <option value="original">原文（不转换）</option>

          <option value="simplified">简体中文</option>

          <option value="traditional">繁体中文</option>

        </select>

      </label>

    </section>



    <section class="group">

      <h4>字体与排版</h4>

      <label class="field">

        字号

        <input

          type="range"

          min="14"

          max="32"

          :value="settingStore.fontSize"

          @input="settingStore.updateSetting('fontSize', Number($event.target.value))"

        />

        <span>{{ settingStore.fontSize }}px</span>

      </label>



      <label class="field">

        行距

        <input

          type="range"

          min="1.4"

          max="2.6"

          step="0.1"

          :value="settingStore.lineHeight"

          @input="settingStore.updateSetting('lineHeight', Number($event.target.value))"

        />

        <span>{{ settingStore.lineHeight }}</span>

      </label>



      <label class="field">

        字体

        <select

          :value="settingStore.fontFamily"

          @change="settingStore.updateSetting('fontFamily', $event.target.value)"

        >

          <option value="system">系统默认</option>

          <option value="pingfang">苹方 / 黑体</option>

          <option value="heiti">微软雅黑</option>

          <option value="serif">宋体 / Serif</option>

          <option value="songti">明宋体</option>

          <option value="kai">楷体</option>

          <option value="fangsong">仿宋</option>

        </select>

      </label>

    </section>



    <section class="group">

      <h4>背景主题</h4>

      <div class="theme-grid">

        <button

          v-for="item in READER_THEMES"

          :key="item.id"

          type="button"

          class="theme-chip"

          :class="{ active: settingStore.theme === item.id, [`chip-${item.id}`]: true }"

          :disabled="settingStore.followSystem"

          @click="settingStore.updateSetting('theme', item.id)"

        >

          {{ item.label }}

        </button>

      </div>

      <label class="field checkbox">

        <input

          type="checkbox"

          :checked="settingStore.followSystem"

          @change="settingStore.updateSetting('followSystem', $event.target.checked)"

        />

        夜间模式跟随系统

      </label>

    </section>



    <section class="group">

      <h4>正文清洗（全局）</h4>

      <p class="group-hint">抓取章节时自动应用，对所有书源生效。</p>

      <label class="field">

        广告关键词（每行一条）

        <textarea

          rows="3"

          :value="settingStore.globalAdKeywords"

          placeholder="请记住本站网址&#10;本章未完"

          @input="settingStore.updateSetting('globalAdKeywords', $event.target.value)"

        />

      </label>

      <label class="field">

        清洗正则（每行一条）

        <textarea

          rows="3"

          :value="settingStore.globalCleanPatterns"

          placeholder="自定义正则表达式"

          @input="settingStore.updateSetting('globalCleanPatterns', $event.target.value)"

        />

      </label>

      <label class="field checkbox">

        <input

          type="checkbox"

          :checked="settingStore.restoreCensoredPinyin"

          @change="settingStore.updateSetting('restoreCensoredPinyin', $event.target.checked)"

        />

        还原 censored 拼音（yindao → 阴道 等）

      </label>

      <label class="field">

        自定义拼音词典（每行一条）

        <textarea

          rows="4"

          class="textarea-mono"

          :disabled="!settingStore.restoreCensoredPinyin"

          :value="settingStore.customPinyinMappings"

          placeholder="# 覆盖内置或补充站点特有词&#10;yindao=阴道&#10;site_only=站点词"

          @input="settingStore.updateSetting('customPinyinMappings', $event.target.value)"

        />

      </label>

      <p class="group-hint">

        针对笔趣阁类源将敏感词替换成拼音的占位符，按内置词典与上下文自动还原；上方文本框可追加或覆盖映射（格式：拼音=汉字）。公版英文书可关闭开关。

      </p>

    </section>



    <section class="group">

      <h4>网络抓取</h4>

      <p class="group-hint">

        对仅校验 Cookie / 请求头的站点可能有效。在浏览器打开目标站并完成验证后，F12 → 网络 → 任选请求 → 复制 Cookie 粘贴 below。

      </p>

      <label class="field">

        全局 Cookie

        <textarea

          rows="2"

          :value="settingStore.fetchCookie"

          placeholder="name=value; name2=value2"

          @input="settingStore.updateSetting('fetchCookie', $event.target.value)"

        />

      </label>

      <label class="field">

        请求间隔（{{ settingStore.fetchMinIntervalMs }} ms，0 为不限速）

        <input

          type="range"

          min="0"

          max="5000"

          step="500"

          :value="settingStore.fetchMinIntervalMs"

          @input="settingStore.updateSetting('fetchMinIntervalMs', Number($event.target.value))"

        />

      </label>

      <p class="group-hint">

        书源 JSON 中可单独设置 <code>encoding: "gbk"</code>、<code>cookies</code>、<code>request_interval_ms</code>，优先级高于全局。

      </p>

    </section>

    <section class="group">
      <h4>本地书库</h4>
      <p class="group-hint">{{ libraryInfo?.hint || "下载/阅读过的书籍保存在本机，不在项目源码目录内。" }}</p>
      <p v-if="libraryInfo?.mode === 'custom'" class="library-tag">自定义位置</p>
      <code v-if="libraryInfo?.path" class="library-path">{{ libraryInfo.path }}</code>
      <p v-else class="group-hint">加载路径中…</p>
      <div class="library-actions">
        <BaseButton type="button" @click="pickLibraryFolder">更改书库文件夹</BaseButton>
        <BaseButton
          v-if="libraryInfo?.mode === 'custom'"
          type="button"
          @click="restoreDefaultLibrary"
        >
          恢复默认位置
        </BaseButton>
      </div>
      <p v-if="libraryNotice" class="library-notice">{{ libraryNotice }}</p>
    </section>

    <div class="panel-actions">

      <BaseButton @click="settingStore.resetToDefault">恢复默认</BaseButton>

      <BaseButton v-if="mode !== 'page'" @click="$emit('close')">关闭</BaseButton>

    </div>

  </aside>

</template>



<script setup>
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingStore, READER_THEMES } from "../../stores/settingStore";
import { getLibraryPath, resetLibraryPath, setLibraryPath } from "../../api/settings";
import BaseButton from "../Common/BaseButton.vue";

const libraryInfo = ref(null);
const libraryNotice = ref("");

async function loadLibraryInfo() {
  try {
    libraryInfo.value = await getLibraryPath();
  } catch {
    libraryInfo.value = null;
  }
}

onMounted(loadLibraryInfo);

async function pickLibraryFolder() {
  libraryNotice.value = "";
  const selected = await open({ directory: true, multiple: false, title: "选择书库保存文件夹" });
  if (!selected || Array.isArray(selected)) return;
  try {
    libraryNotice.value = await setLibraryPath(selected);
  } catch (err) {
    libraryNotice.value = `保存失败：${err}`;
  }
}

async function restoreDefaultLibrary() {
  libraryNotice.value = "";
  try {
    libraryNotice.value = await resetLibraryPath();
    await loadLibraryInfo();
  } catch (err) {
    libraryNotice.value = `恢复失败：${err}`;
  }
}



defineProps({

  /** popover：阅读器浮层；page：独立设置页 */

  mode: { type: String, default: "popover" },

});



defineEmits(["close"]);



const settingStore = useSettingStore();

</script>



<style scoped>

.setting-panel {

  position: absolute;

  right: 16px;

  top: 48px;

  width: 300px;

  max-height: calc(100vh - 120px);

  overflow-y: auto;

  background: var(--color-surface);

  border: 1px solid var(--color-border);

  border-radius: 8px;

  padding: 16px;

  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);

  z-index: 100;

}



.setting-panel--page {

  position: static;

  width: 100%;

  max-height: none;

  box-shadow: none;

}



.setting-panel h3 {

  margin-bottom: 12px;

  font-size: 1rem;

}



.group {

  margin-bottom: 16px;

  padding-bottom: 12px;

  border-bottom: 1px solid var(--color-border);

}



.group:last-of-type {

  border-bottom: none;

}



.group h4 {

  font-size: 0.85rem;

  color: #666;

  margin-bottom: 10px;

}



.group-hint {

  font-size: 0.75rem;

  color: #999;

  margin: -4px 0 8px;

}

.library-path {
  display: block;
  font-size: 0.72rem;
  padding: 8px 10px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.04);
  word-break: break-all;
  line-height: 1.45;
}

.library-tag {
  margin: 0 0 6px;
  font-size: 0.72rem;
  color: var(--color-primary);
}

.library-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.library-notice {
  margin: 8px 0 0;
  font-size: 0.75rem;
  color: #27ae60;
  line-height: 1.45;
}



.field {

  display: flex;

  flex-direction: column;

  gap: 4px;

  margin-bottom: 10px;

  font-size: 0.85rem;

}



.field.checkbox {

  flex-direction: row;

  align-items: center;

  gap: 8px;

}



.field select,

.field textarea {

  padding: 6px 8px;

  border: 1px solid var(--color-border);

  border-radius: 4px;

  font-size: 0.85rem;

  font-family: inherit;

}



.textarea-mono {

  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;

  font-size: 0.82rem;

}



.theme-grid {

  display: grid;

  grid-template-columns: repeat(2, 1fr);

  gap: 8px;

  margin-bottom: 10px;

}



.theme-chip {

  padding: 8px 6px;

  border: 2px solid transparent;

  border-radius: 6px;

  cursor: pointer;

  font-size: 0.78rem;

  transition: border-color 0.15s;

}



.theme-chip:disabled {

  opacity: 0.5;

  cursor: not-allowed;

}



.theme-chip.active {

  border-color: var(--color-primary);

}



.chip-light { background: #fff; color: #333; }

.chip-sepia { background: #f4ecd8; color: #5c4b37; }

.chip-green { background: #c7edcc; color: #2d5016; }

.chip-parchment { background: #f5e6c8; color: #5a4632; }

.chip-bluegray { background: #e3eaf2; color: #3a4a5c; }

.chip-warm { background: #faf0e4; color: #5c4033; }

.chip-dark { background: #1e1e1e; color: #d4d4d4; }

.chip-nightblue { background: #1a2332; color: #c8d4e8; }



.panel-actions {

  display: flex;

  gap: 8px;

  margin-top: 8px;

}

</style>

