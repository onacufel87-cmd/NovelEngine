/** 将 Unix 秒时间戳格式化为相对时间文案 */
export function formatRelativeTime(unixSec) {
  if (!unixSec) return "从未";
  const diff = Math.floor(Date.now() / 1000) - unixSec;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} 天前`;
  return new Date(unixSec * 1000).toLocaleDateString("zh-CN");
}
