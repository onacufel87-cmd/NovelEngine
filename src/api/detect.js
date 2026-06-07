import { invoke } from "@tauri-apps/api/core";

/** 书源零配置 / 一键自动接入 */

export async function autoDetectSelectors({ tocUrl, contentUrl, searchUrl = null }) {
  return invoke("auto_detect_selectors_cmd", {
    tocUrl,
    contentUrl,
    searchUrl: searchUrl || null,
  });
}

/**
 * 解析 DetectResponse：成功返回 { result, logs }，失败抛错但保留 logs
 */
function unwrapDetectResponse(resp) {
  if (resp?.error) {
    const err = new Error(resp.error);
    err.logs = resp.logs || [];
    throw err;
  }
  return {
    result: resp.result,
    logs: resp.logs || [],
  };
}

export async function autoDetectSourceRule({
  name,
  tocUrl,
  contentUrl,
  searchUrl = null,
}) {
  const resp = await invoke("auto_detect_source_rule", {
    name,
    tocUrl,
    contentUrl,
    searchUrl: searchUrl || null,
  });
  const { result, logs } = unwrapDetectResponse(resp);
  return { ruleJson: result, logs };
}

/** 一键全自动书源接入；useRendered 时用 WebView 渲染抓取 */
export async function autoDetectFromUrl(url, { useRendered = false } = {}) {
  const cmd = useRendered ? "auto_detect_from_url_rendered" : "auto_detect_from_url";
  const resp = await invoke(cmd, { url: url.trim() });
  const { result, logs } = unwrapDetectResponse(resp);
  return { result, logs };
}
