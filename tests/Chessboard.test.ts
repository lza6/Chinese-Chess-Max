import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import Chessboard from "../src/components/Chessboard.vue";
import { listeners } from "./mocks/tauri";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

let wrapper: ReturnType<typeof mount> | null = null;

afterEach(() => {
    wrapper?.unmount();
    wrapper = null;
    document.body.innerHTML = "";
});

function mountBoard() {
    wrapper = mount(Chessboard, { attachTo: document.body });
    return wrapper;
}

describe("Chessboard", () => {
    it("渲染 90 个棋位并挂载时发出初始局面", () => {
        const w = mountBoard();
        expect(w.find("#chessboard").exists()).toBe(true);
        expect(w.findAll(".piece-wrap").length).toBe(90);
        expect(w.findAll(".piece").length).toBe(90);
    });

    it("position 事件调用 setPiecesOnBoard 添加棋子", async () => {
        mountBoard();
        const handler = listeners["position"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({
            payload: [
                { piece: "R", pos: "a0" },
                { piece: "k", pos: "e9" },
            ],
        });
        expect(document.getElementById("a0")).toBeTruthy();
    });

    it("mirror 事件翻转棋盘坐标顺序", async () => {
        const w = mountBoard();
        expect(w.findAll(".piece-wrap")[0].attributes("id")).toBe("a9");
        const handler = listeners["mirror"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: true });
        await nextTick();
        expect(w.findAll(".piece-wrap")[0].attributes("id")).toBe("i0");
    });

    it("move 事件移动棋子样式", async () => {
        mountBoard();
        const handler = listeners["move"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: { piece: "r", from: "b7", to: "c7" } });
        expect(document.getElementById("c7")).toBeTruthy();
    });
});
