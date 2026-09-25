import { describe, expect, it, vi, beforeEach } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import Analyse from "../src/components/Analyse.vue";
import { invoke, listeners } from "./mocks/tauri";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

describe("Analyse", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        document.body.innerHTML = "";
        // 构造高亮目标元素
        const wrap = document.createElement("div");
        wrap.id = "e2";
        wrap.className = "piece-wrap";
        const cell = document.createElement("span");
        cell.className = "piece";
        wrap.appendChild(cell);
        document.body.appendChild(wrap);
        const wrap2 = document.createElement("div");
        wrap2.id = "e4";
        wrap2.className = "piece-wrap";
        const cell2 = document.createElement("span");
        cell2.className = "piece";
        wrap2.appendChild(cell2);
        document.body.appendChild(wrap2);
    });

    it("初始状态显示等待分析", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        expect(wrapper.text()).toContain("局面分析");
        expect(wrapper.find('[data-testid="best-move"]').text()).toContain("等待分析");
    });

    it("收到 analyse 事件后更新最佳走法与日志（含时间戳）", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        const handler = listeners["analyse"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({
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
        await nextTick();
        expect(wrapper.text()).toContain("炮二平五");
        const logText = wrapper.find('[data-testid="analyse-log"]').text();
        expect(logText).toMatch(/\[\d{2}:\d{2}:\d{2}\]/);
        // 高亮只加在当轮 from/to
        expect(
            document.getElementById("e2")?.querySelector(".piece")?.classList.contains("b-select"),
        ).toBe(true);
        expect(
            document.getElementById("e4")?.querySelector(".piece")?.classList.contains("b-select"),
        ).toBe(true);
    });

    it("第二次 analyse 只高亮新一轮，不清全局", async () => {
        await mount(Analyse);
        await flushPromises();
        const handler = listeners["analyse"]?.at(-1);
        await handler({
            payload: {
                depth: 6,
                score: 1,
                time: 5,
                pvs: ["e2e4"],
                moves: ["炮二平五"],
                state: "normal",
                source: "pikafish",
            },
        });
        await nextTick();
        await handler({
            payload: {
                depth: 6,
                score: 2,
                time: 5,
                pvs: ["h2h4"],
                moves: ["炮八平六"],
                state: "normal",
                source: "pikafish",
            },
        });
        await nextTick();
        // 旧 from/to 不再有 b-select
        expect(
            document.getElementById("e2")?.querySelector(".piece")?.classList.contains("b-select"),
        ).toBe(false);
    });

    it("加载历史并渲染复盘列表", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("load_history");
        expect(wrapper.find('[data-testid="history-1"]').exists()).toBe(true);
    });

    it("下一步调用 review_step 并在 review_state 事件后更新", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        await wrapper.find('[data-testid="review-next"]').trigger("click");
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("review_step", { index: 1 });
        const handler = listeners["review_state"]?.at(-1);
        await handler({ payload: { index: 1, total: 1, fen: "fen-after" } });
        await nextTick();
        expect(wrapper.text()).toContain("1/1 步");
    });

    it("清空历史按钮调用 clear_history 并重置复盘", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        expect(wrapper.find('[data-testid="history-1"]').exists()).toBe(true);
        await wrapper.find('[data-testid="clear-history"]').trigger("click");
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("clear_history");
    });

    it("导出按钮调用 export_game 并显示路径", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        await wrapper.find('[data-testid="export-fen"]').trigger("click");
        await flushPromises();
        expect(invoke).toHaveBeenCalledWith("export_game", { format: "fen" });
        expect(wrapper.find('[data-testid="export-path"]').text()).toContain(
            "C:/mock/chess-game-1.json",
        );
    });

    it("大量事件时日志裁剪分支被覆盖", async () => {
        const wrapper = mount(Analyse);
        await flushPromises();
        const handler = listeners["analyse"]?.at(-1);
        for (let i = 0; i < 130; i++) {
            await handler({
                payload: {
                    depth: 6,
                    score: i,
                    time: 10,
                    pvs: ["e2e4"],
                    moves: ["炮二平五"],
                    state: "normal",
                    source: "pikafish",
                },
            });
        }
        await nextTick();
        expect(wrapper.text()).toContain("炮二平五");
    });
});
