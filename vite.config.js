import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));
const devFixturesDir = path.join(rootDir, "dev-fixtures");

/** 开发模式下将 dev-fixtures/ 映射到站点根路径，避免打进 release 包 */
function devFixturesPlugin() {
  return {
    name: "dev-fixtures",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const urlPath = req.url?.split("?")[0] ?? "";
        const fileName = path.basename(urlPath);
        if (!fileName.startsWith("test-")) {
          next();
          return;
        }
        const filePath = path.join(devFixturesDir, fileName);
        if (!fs.existsSync(filePath)) {
          next();
          return;
        }
        res.setHeader("Content-Type", fileName.endsWith(".json") ? "application/json" : "text/html; charset=utf-8");
        res.end(fs.readFileSync(filePath));
      });
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), devFixturesPlugin()],

  // Tauri 开发/构建专用配置
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
