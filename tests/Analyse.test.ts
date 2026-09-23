import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import Analyse from "../src/components/Analyse.vue";
import { listeners } from "./mocks/tauri";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

describe("Analyse", () => {
    it("初始状态显示默认最佳走法", () => {
        const wrapper = mount(Analyse);
        expect(wrapper.text()).toContain("局面分析");
        expect(wrapper.text()).toContain("----");
    });

    it("收到 analyse 事件后更新最佳走法与日志", async () => {
        const wrapper = mount(Analyse);
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
    });

    it("大量事件时日志滚动与裁剪分支被覆盖", async () => {
        const wrapper = mount(Analyse);
        const handler = listeners["analyse"]?.at(-1);
        expect(handler).toBeTruthy();
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
