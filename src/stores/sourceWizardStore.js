import { defineStore } from "pinia";

/** 接入向导：0 输入 → 1 预览 → 2 试读 */
const LOADING_HINTS = [
  "抓取页面 HTML…",
  "分析链接与页面结构…",
  "启发式推断 CSS 选择器…",
  "生成书源规则…",
];

export const useSourceWizardStore = defineStore("sourceWizard", {
  state: () => ({
    step: 0,
    detecting: false,
    loadingHint: LOADING_HINTS[0],
    catalogUrl: "",
    /** Rust 检测流水线结构化日志 */
    detectLogs: [],
    detectFailed: false,
    _hintTimer: null,
  }),

  actions: {
    reset() {
      this.stopHintRotation();
      this.step = 0;
      this.detecting = false;
      this.loadingHint = LOADING_HINTS[0];
      this.catalogUrl = "";
      this.detectLogs = [];
      this.detectFailed = false;
    },

    setDetectLogs(logs, failed = false) {
      this.detectLogs = logs || [];
      this.detectFailed = failed;
    },

    startDetect() {
      this.detecting = true;
      this.step = 0;
      this.catalogUrl = "";
      this.detectLogs = [];
      this.detectFailed = false;
      this.startHintRotation();
    },

    completeDetect(catalogUrl = "") {
      this.stopHintRotation();
      this.detecting = false;
      this.step = 1;
      this.detectFailed = false;
      if (catalogUrl) this.catalogUrl = catalogUrl;
    },

    failDetect() {
      this.stopHintRotation();
      this.detecting = false;
      this.step = 0;
      this.detectFailed = true;
    },

    startTrial(catalogUrl = "") {
      this.step = 2;
      if (catalogUrl) this.catalogUrl = catalogUrl;
    },

    startHintRotation() {
      this.stopHintRotation();
      let idx = 0;
      this.loadingHint = LOADING_HINTS[0];
      this._hintTimer = setInterval(() => {
        idx = (idx + 1) % LOADING_HINTS.length;
        this.loadingHint = LOADING_HINTS[idx];
      }, 1400);
    },

    stopHintRotation() {
      if (this._hintTimer) {
        clearInterval(this._hintTimer);
        this._hintTimer = null;
      }
    },
  },
});
