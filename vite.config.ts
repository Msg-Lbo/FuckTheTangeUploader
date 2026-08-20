import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

/**
 * Vite 配置（Tauri 2 官方模板约定）
 */
export default defineConfig({
  plugins: [vue()],
  // Tauri 需要一个固定端口，保证窗口与 dev server 绑定稳定
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
