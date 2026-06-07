import { ref, onMounted, onUnmounted } from "vue";

/**
 * 响应式断点：监听 matchMedia，用于桌面/移动布局切换
 * @param {string} query 媒体查询，默认 ≥1024px 视为桌面
 */
export function useBreakpoint(query = "(min-width: 1024px)") {
  const matches = ref(false);
  let mql = null;

  function sync() {
    if (mql) matches.value = mql.matches;
  }

  onMounted(() => {
    mql = window.matchMedia(query);
    sync();
    mql.addEventListener("change", sync);
  });

  onUnmounted(() => {
    mql?.removeEventListener("change", sync);
  });

  return matches;
}
