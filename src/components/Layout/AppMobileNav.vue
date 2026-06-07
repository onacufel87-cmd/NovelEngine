<template>
  <nav class="mobile-nav" aria-label="主导航">
    <RouterLink
      v-for="item in navItems"
      :key="item.to"
      :to="item.to"
      class="mobile-nav__item"
    >
      <NavIcon :name="item.icon" />
      <span>{{ item.label }}</span>
    </RouterLink>
  </nav>
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
.mobile-nav {
  display: flex;
  justify-content: space-around;
  align-items: stretch;
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 100;
  background: var(--color-surface);
  border-top: 1px solid var(--color-border-light);
  padding: 6px 8px calc(6px + env(safe-area-inset-bottom, 0));
  box-shadow: 0 -2px 12px rgba(44, 62, 47, 0.06);
}

@media (min-width: 900px) {
  .mobile-nav {
    display: none;
  }
}

.mobile-nav__item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  min-height: 44px;
  padding: 4px 0;
  color: var(--color-muted);
  text-decoration: none;
  font-size: 0.68rem;
  border-radius: var(--radius-sm);
  transition: color 0.15s, background 0.15s;
}

.mobile-nav__item.router-link-active {
  color: var(--color-primary);
  font-weight: 600;
}

.mobile-nav__item :deep(.nav-icon) {
  width: 20px;
  height: 20px;
}
</style>
