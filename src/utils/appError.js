/**
 * 解析 Tauri 后端返回的结构化错误 JSON
 * 格式: { "code": "network", "message": "连接超时" }
 */

/** 错误码 → 中文友好提示（可扩展） */
const CODE_HINTS = {
  network: "网络请求失败",
  parse: "页面解析失败",
  database: "本地数据异常",
  not_found: "未找到资源",
  invalid_rule: "书源规则无效",
};

/**
 * 将 invoke 抛出的错误转为用户可读字符串
 * @param {unknown} err
 * @returns {string}
 */
export function formatAppError(err) {
  const raw = String(err ?? "未知错误");

  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && parsed.message) {
      const hint = CODE_HINTS[parsed.code];
      if (hint && !String(parsed.message).includes(hint)) {
        return `${hint}：${parsed.message}`;
      }
      return String(parsed.message);
    }
  } catch {
    // 非 JSON，兼容旧版纯文本错误
  }

  return raw;
}
