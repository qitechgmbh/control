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
      "@ui": path.resolve(__dirname, "../ui/src"),
      "@root": path.resolve(__dirname, ".."),
    },
  },
  server: {
    fs: {
      allow: [path.resolve(__dirname), path.resolve(__dirname, "../ui")],
    },
  },
  clearScreen: false,
  plugins: [
    electron({
      main: {
        entry: "src/main.ts",
        onstart: ({ startup }) => {
          const args = ["./dist-electron/main.js"];
          if (process.env.ELECTRON_NO_SANDBOX) {
            args.push("--no-sandbox");
          }
          startup(args);
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
