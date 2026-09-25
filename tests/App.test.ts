import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { h } from "vue";
import { NDialogProvider, NMessageProvider } from "naive-ui";
import App from "../src/App.vue";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

describe("App", () => {
    it("渲染完整布局：工具栏 + 棋盘 + 局面分析 + 品牌页脚", async () => {
        const wrapper = mount({
            render: () =>
                h(NDialogProvider, null, {
                    default: () => h(NMessageProvider, null, { default: () => h(App) }),
                }),
        });
        expect(wrapper.find("#chessboard").exists()).toBe(true);
        expect(wrapper.text()).toContain("局面分析");
        expect(wrapper.text()).toContain("连线分析");
        const footerLink = wrapper.find('a[href*="github.com/lza6/Chinese-Chess-Max"]');
        expect(footerLink.exists()).toBe(true);
        expect(footerLink.attributes("href")).toContain("github.com/lza6/Chinese-Chess-Max");
        expect(wrapper.findAll(".piece-wrap").length).toBe(90);
    });
});
