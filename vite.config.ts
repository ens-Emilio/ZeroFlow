import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed port; fail fast if it's taken instead of picking another.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    cssTarget: "es2022",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
