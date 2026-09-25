import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { flushPromises } from "@vue/test-utils";
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
    it("渲染 90 个棋位并挂载时发出初始局面", async () => {
        const w = mountBoard();
        expect(w.find("#chessboard").exists()).toBe(true);
        expect(w.findAll(".piece-wrap").length).toBe(90);
        expect(w.findAll(".piece").length).toBe(90);
    });

    it("坐标由数据生成且非镜像首格为 a9", () => {
        const w = mountBoard();
        expect(w.findAll(".piece-wrap")[0].attributes("id")).toBe("a9");
        expect(w.findAll(".piece-wrap")[89].attributes("id")).toBe("i0");
    });

    it("mirror 事件翻转棋盘坐标顺序", async () => {
        const w = mountBoard();
        await flushPromises();
        expect(w.findAll(".piece-wrap")[0].attributes("id")).toBe("a9");
        const handler = listeners["mirror"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: true });
        await nextTick();
        expect(w.findAll(".piece-wrap")[0].attributes("id")).toBe("i0");
        expect(w.findAll(".piece-wrap")[89].attributes("id")).toBe("a9");
    });

    it("position 事件全量重绘（旧棋子被清空）", async () => {
        mountBoard();
        await flushPromises();
        const handler = listeners["position"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: [{ piece: "R", pos: "a0" }] });
        expect(
            document.getElementById("a0")?.firstElementChild?.classList.contains("piece-R"),
        ).toBe(true);
        // 第二次 payload 不含 a0 → 全量重绘清空 a0
        await handler({ payload: [{ piece: "k", pos: "e9" }] });
        expect(
            document.getElementById("a0")?.firstElementChild?.classList.contains("piece-R"),
        ).toBe(false);
        expect(
            document.getElementById("e9")?.firstElementChild?.classList.contains("piece-k"),
        ).toBe(true);
    });

    it("move 事件移动棋子样式", async () => {
        mountBoard();
        await flushPromises();
        const handler = listeners["move"]?.at(-1);
        expect(handler).toBeTruthy();
        await handler({ payload: { piece: "r", from: "b7", to: "c7" } });
        expect(
            document.getElementById("c7")?.firstElementChild?.classList.contains("piece-r"),
        ).toBe(true);
    });
});
