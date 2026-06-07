import { defineStore } from "pinia";
import {
  listBookSources,
  toggleBookSource,
  subscribeRemoteSource,
  importBookSourceJson,
  listSourceSubscriptions,
  syncSourceSubscription,
  pingBookSource,
  pingBookSources,
  deleteBookSources,
  setBookSourcesEnabled,
  searchBooks,
  fetchRankings,
  getRankTypes,
  validateSourceRule,
  fetchChapters,
} from "../api";
import { formatAppError } from "../utils/appError";

/** 试读/解析会话初始状态（原 sourceStore） */
function emptyParseSession() {
  return {
    rule: null,
    chapters: [],
    catalogUrl: "",
    loading: false,
    error: "",
    statusMessage: "",
  };
}

/** 书源库 + 搜索/榜单 + 试读解析会话 */
export const useBookSourceStore = defineStore("bookSource", {
  state: () => ({
    // —— 书源库 ——
    sources: [],
    subscriptions: [],
    loadingSources: false,
    pingingSources: false,
    syncingSubscriptionId: "",
    // —— 搜索 ——
    searchKeyword: "",
    searchResults: [],
    loadingSearch: false,
    hasSearched: false,
    // —— 榜单 ——
    rankTypes: [],
    rankBooks: [],
    selectedSourceId: "",
    selectedRankType: "",
    loadingRank: false,
    // —— 共享错误 ——
    error: "",
    // —— 试读/手动解析会话 ——
    parseSession: emptyParseSession(),
  }),

  getters: {
    enabledSources: (state) => state.sources.filter((s) => s.enabled),
    /** 非内置书源（订阅 + 自定义） */
    managedSources: (state) => state.sources.filter((s) => !s.is_builtin),
    selectedSource: (state) =>
      state.sources.find((s) => s.id === state.selectedSourceId) ?? null,
    /** 是否已加载试读书源规则 */
    hasParseRule: (state) => state.parseSession.rule !== null,
    parseRuleName: (state) => state.parseSession.rule?.name ?? "未加载",
  },

  actions: {
    // —— 书源库 ——

    async loadSources() {
      this.loadingSources = true;
      this.error = "";
      try {
        const [sources, subscriptions] = await Promise.all([
          listBookSources(),
          listSourceSubscriptions(),
        ]);
        this.sources = sources;
        this.subscriptions = subscriptions;
        if (!this.selectedSourceId && this.sources.length) {
          this.selectedSourceId = this.sources[0].id;
        }
      } catch (err) {
        this.error = formatAppError(err);
        this.sources = [];
        this.subscriptions = [];
      } finally {
        this.loadingSources = false;
      }
    },

    /** 将健康检测结果合并回 sources 列表 */
    mergeHealthResults(results) {
      for (const h of results) {
        const idx = this.sources.findIndex((s) => s.id === h.source_id);
        if (idx >= 0) {
          this.sources[idx] = {
            ...this.sources[idx],
            ping_ms: h.ping_ms ?? null,
            health_status: h.health_status,
            last_verified: h.last_verified,
          };
        }
      }
    },

    async pingSource(sourceId) {
      this.error = "";
      const health = await pingBookSource(sourceId);
      this.mergeHealthResults([health]);
      return health;
    },

    async pingSourcesBatch(sourceIds) {
      if (!sourceIds.length) return [];
      this.pingingSources = true;
      this.error = "";
      try {
        const results = await pingBookSources(sourceIds);
        this.mergeHealthResults(results);
        return results;
      } catch (err) {
        this.error = formatAppError(err);
        return [];
      } finally {
        this.pingingSources = false;
      }
    },

    async deleteSourcesBatch(sourceIds) {
      this.error = "";
      const deleted = await deleteBookSources(sourceIds);
      await this.loadSources();
      return deleted;
    },

    async disableSourcesBatch(sourceIds) {
      this.error = "";
      await setBookSourcesEnabled(sourceIds, false);
      await this.loadSources();
    },

    async syncSubscription(subId) {
      this.syncingSubscriptionId = subId;
      this.error = "";
      try {
        const result = await syncSourceSubscription(subId);
        await this.loadSources();
        return result;
      } catch (err) {
        this.error = formatAppError(err);
        throw err;
      } finally {
        this.syncingSubscriptionId = "";
      }
    },

    async setEnabled(sourceId, enabled) {
      const updated = await toggleBookSource(sourceId, enabled);
      const idx = this.sources.findIndex((s) => s.id === sourceId);
      if (idx >= 0) this.sources[idx] = updated;
    },

    async subscribe(url) {
      this.error = "";
      const result = await subscribeRemoteSource(url.trim());
      await this.loadSources();
      return result;
    },

    async importJson(jsonText) {
      this.error = "";
      const record = await importBookSourceJson(jsonText.trim());
      await this.loadSources();
      return record;
    },

    // —— 搜索 / 榜单 ——

    async search(keyword) {
      this.searchKeyword = keyword.trim();
      if (!this.searchKeyword) {
        this.error = "请输入书名或关键词";
        return;
      }

      this.loadingSearch = true;
      this.error = "";
      this.searchResults = [];
      this.hasSearched = false;

      try {
        const origin = window.location.origin;
        this.searchResults = await searchBooks(this.searchKeyword, origin);
        this.hasSearched = true;
      } catch (err) {
        this.hasSearched = true;
        this.error = formatAppError(err);
      } finally {
        this.loadingSearch = false;
      }
    },

    async loadRankMeta(sourceId) {
      this.selectedSourceId = sourceId;
      this.rankTypes = await getRankTypes(sourceId);
      if (this.rankTypes.length) {
        await this.loadRank(this.rankTypes[0]);
      } else {
        this.rankBooks = [];
        this.selectedRankType = "";
      }
    },

    async loadRank(rankType) {
      if (!this.selectedSourceId || !rankType) return;

      this.selectedRankType = rankType;
      this.loadingRank = true;
      this.error = "";

      try {
        const origin = window.location.origin;
        this.rankBooks = await fetchRankings(
          this.selectedSourceId,
          rankType,
          origin
        );
      } catch (err) {
        this.error = formatAppError(err);
        this.rankBooks = [];
      } finally {
        this.loadingRank = false;
      }
    },

    // —— 试读/解析会话 ——

    async loadParseRule(ruleObject) {
      this.parseSession.error = "";
      this.parseSession.statusMessage = "";
      const msg = await validateSourceRule(ruleObject);
      this.parseSession.rule = ruleObject;
      this.parseSession.statusMessage = msg;
      this.parseSession.chapters = [];
    },

    async loadTestParseRule() {
      const res = await fetch("/test-rule.json");
      const rule = await res.json();
      await this.loadParseRule(rule);
    },

    async parseCatalog(url) {
      if (!this.parseSession.rule) {
        this.parseSession.error = "请先加载书源规则 JSON";
        return;
      }

      this.parseSession.loading = true;
      this.parseSession.error = "";
      this.parseSession.catalogUrl = url.trim();

      try {
        this.parseSession.chapters = await fetchChapters(
          this.parseSession.catalogUrl,
          this.parseSession.rule
        );
        this.parseSession.statusMessage = `成功解析 ${this.parseSession.chapters.length} 个章节`;
      } catch (err) {
        this.parseSession.error = formatAppError(err);
        this.parseSession.chapters = [];
      } finally {
        this.parseSession.loading = false;
      }
    },

    clearParseChapters() {
      this.parseSession.chapters = [];
      this.parseSession.catalogUrl = "";
    },

    setParseError(message) {
      this.parseSession.error = message;
    },
  },
});
