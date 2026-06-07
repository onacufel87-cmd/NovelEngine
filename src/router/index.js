import { createRouter, createWebHistory } from "vue-router";

import ShelfView from "../views/ShelfView.vue";
import CommentListView from "../views/CommentListView.vue";
import DiscoverView from "../views/DiscoverView.vue";
import ImportView from "../views/ImportView.vue";
import ReadView from "../views/ReadView.vue";
import SettingsView from "../views/SettingsView.vue";

const routes = [
  { path: "/", name: "bookshelf", component: ShelfView },
  { path: "/notes", name: "comments", component: CommentListView },
  { path: "/discover", name: "discover", component: DiscoverView },
  { path: "/import", name: "sources", component: ImportView },
  { path: "/settings", name: "settings", component: SettingsView },
  { path: "/read/:bookId?", name: "read", component: ReadView },
  { path: "/search", redirect: "/discover" },
  { path: "/rank", redirect: { path: "/discover", query: { tab: "rank" } } },
  { path: "/parse", redirect: "/import" },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
