import { test, expect } from "@playwright/test";

// E2E 说明：本测试在真实 Chromium 浏览器里启动 Vite 应用，验证前端渲染。
// Tauri 原生 IPC 桥（window.__TAURI_INTERNALS__）在普通浏览器中不存在，
// 因此注入最小 mock，仅覆盖“前端 UI 渲染 + 无未捕获异常”这一层。
test.describe("中国象棋Max 前端渲染 E2E（Tauri 桥 mock）", () => {
    test("首页渲染：标题、棋盘、品牌页脚、无未捕获异常", async ({ page }) => {
        const pageErrors: string[] = [];
        page.on("pageerror", (err) => pageErrors.push(err.message));

        await page.addInitScript(() => {
            (window as any).__TAURI_INTERNALS__ = {
                invoke: (cmd: string, _args?: unknown) => {
                    if (cmd === "get_engine_config") {
                        return Promise.resolve({
                            depth: 20,
                            time: 5000,
                            threads: 4,
                            hash: 64,
                            chessdb_enabled: false,
                            chessdb_timeout: 5,
                        });
                    }
                    if (cmd === "list_windows") return Promise.resolve([]);
                    return Promise.resolve(null);
                },
                listen: () => Promise.resolve(() => {}),
                emit: () => Promise.resolve(),
                transformCallback: (cb: () => void) => cb,
                metadata: { currentWindow: { label: "main" } },
            };
        });

        await page.goto("/");
        await expect(page).toHaveTitle(/中国象棋/);
        await expect(page.locator("#chessboard")).toBeVisible();
        await expect(page.locator(".piece-wrap")).toHaveCount(90);
        await expect(page.getByText("连线分析").first()).toBeVisible();
        await expect(page.getByText("局面分析").first()).toBeVisible();

        // naive-ui 的 footer 不渲染为 <footer>，直接按品牌链接定位
        const footer = page.locator('a[href*="lza6/Chinese-Chess-Max"]');
        await expect(footer).toHaveAttribute("href", /lza6\/Chinese-Chess-Max/);

        expect(pageErrors, `捕获到未处理异常: ${pageErrors.join(" | ")}`).toEqual([]);
    });
});
