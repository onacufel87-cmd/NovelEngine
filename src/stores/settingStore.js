import { defineStore } from "pinia";

import { getReaderSettings, saveReaderSettings } from "../api";



/** 默认阅读与全局设置 */

export const DEFAULT_SETTINGS = {

  fontSize: 18,

  lineHeight: 1.8,

  theme: "green",

  fontFamily: "system",

  followSystem: false,

  /** 中文显示：original | simplified | traditional */

  chineseVariant: "original",

  /** 全局正文清洗：每行一条广告关键词 */

  globalAdKeywords: "",

  /** 全局正文清洗：每行一条正则 */

  globalCleanPatterns: "",

  /** 还原 censored 拼音占位符（yindao → 阴道） */
  restoreCensoredPinyin: true,

  /** 自定义拼音映射，每行一条（yindao=汉字） */
  customPinyinMappings: "",

  /** 全局 HTTP Cookie（从浏览器复制，对部分反爬站有效） */

  fetchCookie: "",

  /** 连续请求最小间隔（毫秒），0 表示不限速 */

  fetchMinIntervalMs: 1000,

  /** 是否已完成首次启动引导 */

  onboardingCompleted: false,

  /** 书架展示：grid | list */

  shelfViewMode: "grid",

  /** 桌面阅读页左侧目录是否折叠（已废弃，保留兼容旧设置） */

  readerSidebarCollapsed: true,

};



/** 字体映射表 */

export const FONT_FAMILIES = {

  system: '"Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif',

  serif: '"Noto Serif SC", "Source Han Serif SC", SimSun, serif',

  kai: 'KaiTi, "STKaiti", "FangSong", serif',

  heiti: '"Microsoft YaHei", "PingFang SC", "Heiti SC", sans-serif',

  songti: 'SimSun, "Songti SC", "Noto Serif SC", serif',

  fangsong: '"FangSong", "STFangsong", serif',

  pingfang: '"PingFang SC", "Helvetica Neue", sans-serif',

};



/** 阅读背景主题选项 */

export const READER_THEMES = [

  { id: "green", label: "护眼绿" },

  { id: "sepia", label: "牛皮纸" },

  { id: "light", label: "默认白" },

  { id: "parchment", label: "羊皮暖黄" },

  { id: "bluegray", label: "浅蓝灰" },

  { id: "warm", label: "暖杏色" },

  { id: "dark", label: "夜间黑" },

  { id: "nightblue", label: "深靛蓝" },

];



let saveTimer = null;



export const useSettingStore = defineStore("setting", {

  state: () => ({

    ...DEFAULT_SETTINGS,

    loaded: false,

    systemDark: false,

  }),



  getters: {

    fontFamilyCss: (state) =>

      FONT_FAMILIES[state.fontFamily] ?? FONT_FAMILIES.system,

    readerThemeClass: (state) => `reader-theme-${state.theme}`,

    effectiveReaderThemeClass(state) {

      if (state.followSystem) {

        return state.systemDark ? "reader-theme-dark" : "reader-theme-light";

      }

      return `reader-theme-${state.theme}`;

    },

    /** 解析全局广告关键词列表 */

    globalAdKeywordList(state) {

      return splitLines(state.globalAdKeywords);

    },

    /** 解析全局清洗正则列表 */

    globalCleanPatternList(state) {

      return splitLines(state.globalCleanPatterns);

    },

  },



  actions: {

    async loadFromDB() {

      try {

        const saved = await getReaderSettings();

        Object.assign(this, DEFAULT_SETTINGS, saved);

        delete this.pageMode;

      } catch {

        Object.assign(this, DEFAULT_SETTINGS);

      }

      this.loaded = true;

    },



    updateSetting(key, value) {

      if (!(key in DEFAULT_SETTINGS)) return;

      this[key] = value;

      this.scheduleSave();

    },



    scheduleSave() {

      if (saveTimer) clearTimeout(saveTimer);

      saveTimer = setTimeout(() => this.persist(), 500);

    },



    async persist() {

      const payload = {

        fontSize: this.fontSize,

        lineHeight: this.lineHeight,

        theme: this.theme,

        fontFamily: this.fontFamily,

        followSystem: this.followSystem,

        chineseVariant: this.chineseVariant,

        globalAdKeywords: this.globalAdKeywords,

        globalCleanPatterns: this.globalCleanPatterns,

        restoreCensoredPinyin: this.restoreCensoredPinyin,

        customPinyinMappings: this.customPinyinMappings,

        fetchCookie: this.fetchCookie,

        fetchMinIntervalMs: this.fetchMinIntervalMs,

        onboardingCompleted: this.onboardingCompleted,

        shelfViewMode: this.shelfViewMode,

        readerSidebarCollapsed: this.readerSidebarCollapsed,

      };

      try {

        await saveReaderSettings(payload);

      } catch {

        // 设置保存失败静默处理

      }

    },



    resetToDefault() {

      Object.assign(this, DEFAULT_SETTINGS);

      this.persist();

    },

  },

});



function splitLines(text) {

  return String(text || "")

    .split("\n")

    .map((s) => s.trim())

    .filter(Boolean);

}

