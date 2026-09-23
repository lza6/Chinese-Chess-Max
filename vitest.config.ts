import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
    plugins: [vue()],
    test: {
        environment: "jsdom",
        globals: true,
        setupFiles: ["tests/setup.ts"],
        include: ["tests/**/*.test.ts"],
        coverage: {
            provider: "v8",
            include: ["src/**/*.{ts,vue}"],
            exclude: ["src/main.ts", "src/vite-env.d.ts", "src/**/*.test.ts"],
            reporter: ["text", "json", "html"],
            thresholds: {
                lines: 50,
                functions: 40,
                branches: 40,
                statements: 50,
            },
        },
    },
});
