import { defineConfig } from "@playwright/test";

export default defineConfig({
    testDir: "./e2e",
    timeout: 30000,
    retries: 0,
    use: {
        baseURL: "http://localhost:5173",
        headless: true,
        viewport: { width: 1280, height: 800 },
    },
    webServer: {
        command: "pnpm dev --port 5173",
        url: "http://localhost:5173",
        reuseExistingServer: !process.env.CI,
        timeout: 60000,
    },
});
