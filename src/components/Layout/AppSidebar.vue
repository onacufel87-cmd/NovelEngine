<template>
  <aside class="app-sidebar">
    <RouterLink to="/" class="sidebar-brand" title="小说引擎">
      <img class="brand-mark" src="/app-icon.png?v=2" alt="" width="44" height="44" />
      <span class="brand-text">小说引擎</span>
    </RouterLink>

    <nav class="sidebar-nav">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="sidebar-link"
        :title="item.label"
      >
        <NavIcon :name="item.icon" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>

    <p class="sidebar-foot">本地运行 · 数据不上传</p>
  </aside>
</template>

<script setup>
import NavIcon from "./NavIcon.vue";

const navItems = [
  { to: "/", label: "书架", icon: "shelf" },
  { to: "/notes", label: "评论", icon: "notes" },
  { to: "/discover", label: "发现", icon: "discover" },
  { to: "/import", label: "书源", icon: "sources" },
  { to: "/settings", label: "设置", icon: "settings" },
];
</script>

<style scoped>
.app-sidebar {
  display: none;
  width: var(--sidebar-width);
  flex-shrink: 0;
  flex-direction: column;
  padding: 20px 14px;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border-light);
  min-height: 100vh;
}

@media (min-width: 900px) {
  .app-sidebar {
    display: flex;
  }
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  text-decoration: none;
  color: inherit;
  padding: 4px 8px 20px;
  margin-bottom: 8px;
}

.brand-mark {
  width: 44px;
  height: 44px;
  border-radius: 11px;
  object-fit: contain;
  flex-shrink: 0;
  background: transparent;
  box-shadow: none;
}

.brand-text {
  font-size: 0.95rem;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.sidebar-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  color: var(--color-muted);
  text-decoration: none;
  font-size: 0.9rem;
  transition: background 0.15s, color 0.15s;
}

.sidebar-link:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.sidebar-link.router-link-active {
  background: var(--color-primary-soft);
  color: var(--color-primary);
  font-weight: 600;
}

.sidebar-foot {
  font-size: 0.68rem;
  color: var(--color-muted);
  line-height: 1.5;
  padding: 12px 8px 0;
  opacity: 0.85;
}
</style>
