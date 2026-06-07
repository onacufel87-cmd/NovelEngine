import { invoke } from "@tauri-apps/api/core";

/** 书源管理、规则校验、目录/正文解析 */

export async function validateSourceRule(rule) {
  return invoke("validate_source_rule", {
    ruleJson: JSON.stringify(rule),
  });
}

export async function fetchChapters(url, rule) {
  return invoke("fetch_chapters", {
    url,
    ruleJson: JSON.stringify(rule),
  });
}

export async function getBookContent(chapterUrl, rule) {
  return invoke("get_book_content", {
    chapterUrl,
    ruleJson: JSON.stringify(rule),
  });
}

export async function listBookSources() {
  return invoke("list_book_sources");
}

export async function toggleBookSource(sourceId, enabled) {
  return invoke("toggle_book_source", { sourceId, enabled });
}

export async function subscribeRemoteSource(url) {
  return invoke("subscribe_remote_source", { url });
}

export async function importBookSourcesBatch(ruleJson) {
  return invoke("import_book_sources_batch", { ruleJson });
}

export async function importBookSourceJson(ruleJson) {
  return invoke("import_book_source_json", { ruleJson });
}

/** 列出已订阅的书源仓库 */
export async function listSourceSubscriptions() {
  return invoke("list_source_subscriptions");
}

/** 重新同步订阅仓库 */
export async function syncSourceSubscription(subId) {
  return invoke("sync_source_subscription", { subId });
}

/** 探测单书源健康状态 */
export async function pingBookSource(sourceId) {
  return invoke("ping_book_source", { sourceId });
}

/** 批量探测书源 */
export async function pingBookSources(sourceIds) {
  return invoke("ping_book_sources", { sourceIds });
}

/** 批量删除非内置书源 */
export async function deleteBookSources(sourceIds) {
  return invoke("delete_book_sources", { sourceIds });
}

/** 批量启用/禁用书源 */
export async function setBookSourcesEnabled(sourceIds, enabled) {
  return invoke("set_book_sources_enabled", { sourceIds, enabled });
}
