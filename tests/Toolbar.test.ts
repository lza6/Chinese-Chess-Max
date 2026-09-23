import { describe, expect, it, vi } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { h } from "vue";
import { NDialogProvider } from "naive-ui";
import Toolbar from "../src/components/Toolbar.vue";
import { invoke } from "./mocks/tauri";

vi.mock("@tauri-apps/api/core", () => import("./mocks/tauri"));
vi.mock("@tauri-apps/api/event", () => import("./mocks/tauri"));

function mountWithDialog(component: any) {
    return mount({
        render: () => h(NDialogProvider, null, { default: () => h(component) }),
    });
}

describe("Toolbar", () => {
    it("挂载后加载引擎配置并渲染关键控件", async () => {
        const wrapper = mountWithDialog(Toolbar);
        await flushPromises();
        // 工具栏卡片存在
        expect(wrapper.find(".n-card").exists()).toBe(true);
        // 模式选择默认“连线分析”
        expect(wrapper.text()).toContain("连线分析");
        // 关键按钮渲染
        expect(wrapper.text()).toContain("启");
        expect(wrapper.text()).toContain("配");
        // 挂载时拉取引擎配置
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
});
