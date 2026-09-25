import { test, expect } from "@playwright/test";

// E2E 说明：真实 Chromium 浏览器启动 Vite 应用，注入 Tauri IPC mock，
// 覆盖真实用户路径：渲染 → 启动 → 分析事件 → 停止 → 配置保存 → 复制 → 复盘/导出 → 响应式。
const MOCK = `
    (window).__TAURI_INTERNALS__ = {
        invoke: (cmd, args) => {
            if (cmd === "plugin:event|listen") {
                (window.__EVENT_HANDLERS__ ??= {})[args.event] = args.handler;
                return Promise.resolve(1);
            }
            if (cmd === "plugin:event|unlisten") return Promise.resolve(null);
            switch (cmd) {
                case "get_engine_config":
                    return Promise.resolve({ depth: 20, time: 5000, threads: 4, hash: 64, chessdb_enabled: false, chessdb_timeout: 5 });
                case "list_windows":
                    return Promise.resolve([
                        { id: 2, title: "QQ 游戏大厅 - 中国象棋", app_name: "QQGame", width: 1024, height: 768 },
                        { id: 1, title: "Chrome", app_name: "chrome", width: 1280, height: 800 },
                    ]);
                case "get_current_fen":
                    return Promise.resolve(["rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w", "w"]);
                case "export_game":
                    return Promise.resolve("C:/mock/chess-game-1.json");
                case "load_history":
                    return Promise.resolve([{ seq: 1, from: "b7", to: "c7", piece: "c", camp: "b", iccs: "b7c7", fen: "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b", source: "human" }]);
                case "stop_listen":
                case "start_listen":
                case "set_engine_depth":
                case "set_engine_time":
                case "set_engine_threads":
                case "set_engine_hash":
                case "set_chessdb":
                case "clear_history":
                case "review_step":
                case "reload_engine":
                    return Promise.resolve(null);
                default:
                    return Promise.resolve(null);
            }
        },
        emit: () => Promise.resolve(),
        transformCallback: (cb) => cb,
        metadata: { currentWindow: { label: "main" } },
    };
    window.__TAURI__ = true;
    // 剪贴板 mock（无权限环境，强制覆盖）
    let clip = "";
    Object.defineProperty(navigator, "clipboard", {
        value: {
            writeText: async (t) => { clip = t; },
            readText: async () => clip,
        },
        configurable: true,
    });
`;

async function gotoApp(page: import("@playwright/test").Page) {
    await page.addInitScript(MOCK as any);
    await page.goto("/");
    await expect(page.locator("#chessboard")).toBeVisible();
    await expect(page.locator(".piece-wrap")).toHaveCount(90);
}

test.describe("中国象棋Max 前端 E2E（Tauri 桥 mock，真实浏览器）", () => {
    test("首页渲染 + 空状态引导 + 无未捕获异常", async ({ page }) => {
        const pageErrors: string[] = [];
        page.on("pageerror", (err) => pageErrors.push(err.message));
        await gotoApp(page);
        await expect(page).toHaveTitle(/中国象棋/);
        await expect(page.getByText("连线分析").first()).toBeVisible();
        await expect(page.getByText("局面分析").first()).toBeVisible();
        await expect(page.locator('[data-testid="listen-status"]')).toContainText("空闲");
        await expect(page.getByText("启动引擎后将自动识别棋盘并显示最佳招法")).toBeVisible();
        expect(pageErrors, `捕获到未处理异常: ${pageErrors.join(" | ")}`).toEqual([]);
    });

    test("真实路径：选窗→启动→analyse 事件→最佳招法/日志/高亮→停止", async ({ page }) => {
        await gotoApp(page);

        // 点击启动 → 弹出窗口选择对话框
        await page.locator('[data-testid="toggle-engine"]').click();
        await expect(page.getByText("选择要监听的窗口")).toBeVisible();
        // 窗口列表排序后第一项应为 Chrome（自然排序）
        await expect(page.getByText("Chrome").first()).toBeVisible();

        // 键盘选择 + 确定
        const chromeCard = page.locator(".n-card").filter({ hasText: "Chrome" }).first();
        await chromeCard.focus();
        await page.keyboard.press("Enter");
        await page.getByRole("button", { name: "确定" }).click();

        // 状态徽标变为运行中（start_listen 后前端本地置 running）
        await expect(page.locator('[data-testid="listen-status"]')).toContainText("运行中");

        // 触发 analyse 事件（模拟后端）

        await page.evaluate(() => {
            (window as any).__EVENT_HANDLERS__?.["analyse"]?.({
                payload: {
                    depth: 12,
                    score: 35,
                    time: 120,
                    pvs: ["e2e4"],
                    moves: ["炮二平五"],
                    state: "normal",
                    source: "pikafish",
                },
            });
        });
        await expect(page.locator('[data-testid="best-move"]')).toContainText("炮二平五");
        await expect(page.locator('[data-testid="analyse-log"]')).toContainText(
            /\[\d{2}:\d{2}:\d{2}\]/,
        );

        // 停止
        await page.locator('[data-testid="toggle-engine"]').click();
        await expect(page.locator('[data-testid="listen-status"]')).toContainText("空闲");
    });

    test("配置抽屉保存 + 重载引擎", async ({ page }) => {
        await gotoApp(page);
        await page.locator('[data-testid="open-config"]').click();
        await expect(page.getByText("引擎配置").first()).toBeVisible();
        await page.locator('[data-testid="save-config"]').click();
        // 保存成功（message 出现）且抽屉关闭
        await expect(page.getByText("配置已保存").first()).toBeVisible();
        await expect(page.locator('[data-testid="open-config"]')).toBeVisible();

        await page.locator('[data-testid="open-config"]').click();
        await expect(page.locator('[data-testid="reload-engine"]')).toBeVisible();
        await page.locator('[data-testid="reload-engine"]').click();
        await expect(page.getByText("引擎已重载").first()).toBeVisible();
    });

    test("复制局面 FEN + 复盘 + 导出", async ({ page }) => {
        await gotoApp(page);
        // 复制
        await page.locator('[data-testid="copy-fen"]').click();
        await expect(page.getByText("已复制 FEN 局面").first()).toBeVisible();
        const clip = await page.evaluate(() => navigator.clipboard.readText());
        expect(clip).toContain("rnbakabnr");

        // 复盘列表 + 下一步
        await expect(page.locator('[data-testid="history-1"]')).toContainText("b7→c7");
        await page.locator('[data-testid="review-next"]').click();
        await expect(page.locator('[data-testid="review-next"]')).toBeVisible();

        // 导出 FEN
        await page.locator('[data-testid="export-fen"]').click();
        await expect(page.locator('[data-testid="export-path"]')).toContainText(
            "C:/mock/chess-game-1.json",
        );
    });

    test("响应式：1280×800 无横向滚动", async ({ page }) => {
        await page.setViewportSize({ width: 1280, height: 800 });
        await gotoApp(page);
        const overflow = await page.evaluate(
            () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(1);
    });

    test("响应式：390×844 移动端无横向滚动", async ({ page }) => {
        await page.setViewportSize({ width: 390, height: 844 });
        await gotoApp(page);
        const overflow = await page.evaluate(
            () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(1);
        await expect(page.locator("#chessboard")).toBeVisible();
    });
});
