import { defineConfig } from "vite";

// Tauri expects a fixed dev server port and we don't need a browser opening.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "es2021",
    outDir: "dist",
    emptyOutDir: true,
  },
});
