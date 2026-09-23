import eslint from "@eslint/js";
import pluginVue from "eslint-plugin-vue";
import tseslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";

export default [
    {
        ignores: [
            "node_modules/**",
            "dist/**",
            "coverage/**",
            "playwright-report/**",
            "test-results/**",
            "public/**",
            "src/assets/**",
            "server/**",
            "docs/**",
            "graft/**",
            "libs/**",
            ".agents/**",
            ".claude/**",
            ".cline/**",
            ".codebuddy/**",
            ".codegraph/**",
            ".continue/**",
            ".junie/**",
            ".kiro/**",
            "AGENTS.md",
            "*.ttf",
            "*.png",
            "*.jpg",
            "*.jpeg",
            "*.gif",
            "*.svg",
            "pnpm-lock.yaml",
        ],
    },
    eslint.configs.recommended,
    ...tseslint.configs.recommended,
    ...pluginVue.configs["flat/recommended"],
    {
        files: ["**/*.vue"],
        languageOptions: {
            parserOptions: {
                parser: tseslint.parser,
                extraFileExtensions: [".vue"],
            },
        },
    },
    {
        rules: {
            // 现有组件为单单词命名（Toolbar/Analyse/Chessboard），属命名风格而非 bug
            "vue/multi-word-component-names": "off",
            // 属性顺序是纯风格约定，上游遗留代码未遵循，交给 Prettier 管理即可
            "vue/attributes-order": "off",
            // 模板 v-for 的 "_" 占位参数
            "vue/no-unused-vars": ["error", { ignorePattern: "^_" }],
            // TypeScript 编译器已负责未定义变量检查
            "no-undef": "off",
            "no-unused-vars": "off",
            "@typescript-eslint/no-unused-vars": [
                "error",
                { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
            ],
            // vite-env.d.ts 的 Vue SFC shim 需要使用 {} 占位类型
            "@typescript-eslint/no-empty-object-type": ["error", { allowObjectTypes: "always" }],
            // 与业务代码现网风格一致的宽松项（必要时按真实问题收紧）
            "@typescript-eslint/no-explicit-any": "off",
        },
    },
    eslintConfigPrettier,
];
