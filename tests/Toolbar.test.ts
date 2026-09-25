import { describe, expect, it, vi, beforeEach } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { h } from "vue";
import { NDialogProvider, NMessageProvider } from "naive-ui";
import Toolbar from "../src/components/Toolbar.vue";
import { invoke, listeners } from "./mocks/tauri";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

function mountWithDialog(component: any) {
    return mount({
        render: () =>
            h(NDialogProvider, null, {
                default: () => h(NMessageProvider, null, { default: () => h(component) }),
            }),
    });
}

describe("Toolbar", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        invoke.mockImplementation(async (cmd: string) => {
            switch (cmd) {
                case "get_engine_config":
                    return {
                        depth: 20,
                        time: 5000,
                        threads: 4,
                        hash: 64,
                        chessdb_enabled: false,
                        chessdb_timeout: 5,
                    };
                case "list_windows":
                    return [];
                case "get_current_fen":
                    return ["rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w", "w"];
                default:
                    return null;
            }
        });
    });

    it("挂载后加载引擎配置并渲染关键控件", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        expect(wrapper.find(".n-card").exists()).toBe(true);
        expect(wrapper.text()).toContain("连线分析");
        expect(wrapper.text()).toContain("启");
        expect(wrapper.text()).toContain("配");
        expect(invoke).toHaveBeenCalledWith("get_engine_config");
    });

    it("点击启动且无可用窗口时弹出警告", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        const startBtn = wrapper.findAll("button").find((b) => b.text().includes("启"));
        expect(startBtn).toBeTruthy();
        await startBtn!.trigger("click");
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("list_windows");
        expect(document.body.textContent).toContain("没有找到可用的窗口");
    });

    it("复制局面按钮调用 get_current_fen 并写剪贴板", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        const copyBtn = wrapper.find('[data-testid="copy-fen"]');
        expect(copyBtn.exists()).toBe(true);
        await copyBtn.trigger("click");
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("get_current_fen");
        expect(await navigator.clipboard.readText()).toContain("rnbakabnr");
    });

    it("配置抽屉保存调用 6 个 setter 并关闭", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        await wrapper.find('[data-testid="open-config"]').trigger("click");
        await flushPromises();
        // 抽屉 teleport 到 body
        const saveBtn = document.body.querySelector('[data-testid="save-config"]');
        expect(saveBtn).toBeTruthy();
        (saveBtn as HTMLElement).click();
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("set_engine_depth", { depth: expect.any(Number) });
        expect(invoke).toHaveBeenCalledWith("set_engine_time", { time: expect.any(Number) });
        expect(invoke).toHaveBeenCalledWith("set_engine_threads", { num: expect.any(Number) });
        expect(invoke).toHaveBeenCalledWith("set_engine_hash", { size: expect.any(Number) });
        expect(invoke).toHaveBeenCalledWith("set_chessdb", {
            enabled: expect.any(Boolean),
            timeout: expect.any(Number),
        });
    });

    it("重载引擎按钮调用 reload_engine", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        await wrapper.find('[data-testid="open-config"]').trigger("click");
        await flushPromises();
        const reloadBtn = document.body.querySelector('[data-testid="reload-engine"]');
        expect(reloadBtn).toBeTruthy();
        (reloadBtn as HTMLElement).click();
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("reload_engine");
    });

    it("listen_state 事件更新状态徽标", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        const handler = listeners["listen_state"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: "running" });
        await flushPromises();
        expect(wrapper.find('[data-testid="listen-status"]').text()).toContain("运行中");
        await handler({ payload: "idle" });
        await flushPromises();
        expect(wrapper.find('[data-testid="listen-status"]').text()).toContain("空闲");
    });
});
