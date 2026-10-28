import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import eslintPluginPrettierRecommended from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import path from "node:path";
import { includeIgnoreFile } from "@eslint/compat";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const prettierIgnorePath = path.resolve(__dirname, ".prettierignore");

/** @type {import('eslint').Linter.Config[]} */
export default [
  includeIgnoreFile(prettierIgnorePath),
  {
    ignores: ["src/components/ui/**/*"],
  },
  {
    files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
    plugins: {
      "react-hooks": reactHooks,
    },
    // React Compiler rules only (successor of eslint-plugin-react-compiler)
    rules: {
      ...Object.fromEntries(
        Object.entries(reactHooks.configs.flat.recommended.rules).filter(
          ([rule]) =>
            rule !== "react-hooks/rules-of-hooks" &&
            rule !== "react-hooks/exhaustive-deps",
        ),
      ),
      // Not caught by the old plugin; existing violations still to be fixed
      "react-hooks/refs": "warn",
      "react-hooks/set-state-in-effect": "warn",
      "react-hooks/purity": "warn",
      "react-hooks/immutability": "warn",
      "react-hooks/static-components": "warn",
      "react-hooks/preserve-manual-memoization": "warn",
    },
  },
  { languageOptions: { globals: globals.browser } },
  pluginJs.configs.recommended,
  pluginReact.configs.flat.recommended,
  eslintPluginPrettierRecommended,
  ...tseslint.configs.recommended,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": "warn",
      "react/no-unknown-property": "warn",
    },
  },
];
