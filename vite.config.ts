import { resolve } from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// The Tauri CLI sets TAURI_DEV_HOST when it drives the dev server.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host ?? false,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "chrome120",
    rollupOptions: {
      input: {
        flyout: resolve(import.meta.dirname, "index.html"),
        strip: resolve(import.meta.dirname, "strip.html"),
      },
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
  },
});
