import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@ui": path.resolve(__dirname, "../ui/src"),
    },
  },
  test: {
    dir: "../ui/src/tests/unit",
    globals: true,
    environment: "jsdom",
    setupFiles: "../ui/src/tests/unit/setup.ts",
    css: true,
    reporters: ["verbose"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["src/**/*", "../ui/src/**/*"],
      exclude: [],
    },
  },
});
