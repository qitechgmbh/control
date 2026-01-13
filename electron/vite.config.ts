import react from "@vitejs/plugin-react";
import electron from "vite-plugin-electron/simple";
import path from "path";

// https://vitejs.dev/config
export default {
  publicDir: path.resolve(__dirname, "public"),
  resolve: {
    preserveSymlinks: true,
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@root": path.resolve(__dirname, ".."),
    },
  },
  clearScreen: false,
  plugins: [
    electron({
      main: {
        entry: "src/main.ts",
        onstart: ({ startup }) => {
          startup(["./dist-electron/main.js"]);
        },
      },
      preload: {
        // Shortcut of `build.rollupOptions.input`
        input: "src/preload.ts",
      },
      // Optional: Use Node.js API in the Renderer process
      renderer: {},
    }),
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler"]],
      },
    }),
  ],
};
