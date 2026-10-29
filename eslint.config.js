import js from "@eslint/js"
import pluginVue from "eslint-plugin-vue"
import globals from "globals"
import prettier from "eslint-config-prettier"

export default [
  { ignores: ["dist/**", "node_modules/**", "src-tauri/**"] },
  js.configs.recommended,
  ...pluginVue.configs["flat/recommended"],
  prettier,
  {
    languageOptions: {
      globals: { ...globals.browser },
    },
    rules: {
      "no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      // Компоненты в проекте именуются по месту использования (TaskActions, HeaderLayout…).
      "vue/multi-word-component-names": "off",
      // Пропсы часто описываются как { task: Object } — без required/default.
      "vue/require-default-prop": "off",
      // Слоты Vuetify вида #item.actions и #header.data-table-select.
      "vue/valid-v-slot": ["error", { allowModifiers: true }],
    },
  },
  {
    files: ["vite.config.js", "eslint.config.js"],
    languageOptions: { globals: { ...globals.node } },
  },
]
