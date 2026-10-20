import { defineConfig } from "vite";

export default defineConfig({
  build: {
    outDir: "dist",
  },
  server: {
    port: 5174, // endmill-sandbox(5173) 와 동시에 띄워도 부딪치지 않게
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  },
});