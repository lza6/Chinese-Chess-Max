<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, ref, h, computed } from "vue";
import { useDialog, useMessage } from "naive-ui";
import {
    NButton,
    NCard,
    NFlex,
    NForm,
    NFormItem,
    NInputNumber,
    NDrawer,
    NDrawerContent,
    NSpace,
    NTooltip,
    NDivider,
    NEmpty,
    NScrollbar,
    NTag,
    NInput,
    NSwitch,
    NText,
} from "naive-ui";

interface EngineConfig {
    depth: number;
    time: number; // 秒（展示/编辑）
    threads: number;
    hash: number;
    chessdb_enabled: boolean;
    chessdb_timeout: number;
}


const config = ref<EngineConfig>({
    depth: 20,
    time: 5,
    threads: 4,
    hash: 64,
    chessdb_enabled: false,
    chessdb_timeout: 5,
});
// 上次成功保存的配置（失败时还原）
const savedConfig = ref<EngineConfig>({ ...config.value });

const showEngineConfig = ref(false);
const isEngineRunning = ref(false);
const listenStatus = ref<"idle" | "running" | "error">("idle");
const listenError = ref("");
// 防重复操作：启动/停止中锁按钮
const isPending = ref(false);

const dialog = useDialog();
const message = useMessage();

// 宽容解析 listen_state（Rust ListenState lowercase 序列化）
function parseListenState(payload: unknown): { status: "idle" | "running" | "error"; error?: string } {
    if (typeof payload === "string") {
        const lower = payload.toLowerCase();
        if (lower === "running") return { status: "running" };
        if (lower === "error") return { status: "error" };
        return { status: "idle" };
    }
    if (payload && typeof payload === "object") {
        const p = payload as Record<string, unknown>;
        if (p.error !== undefined || p.Error !== undefined) {
            return { status: "error", error: String(p.error ?? p.Error) };
        }
        if (p.running !== undefined && p.running !== false) return { status: "running" };
        if (p.idle !== undefined && p.idle !== false) return { status: "idle" };
        if (p.Running !== undefined) return { status: "running" };
        if (p.Idle !== undefined) return { status: "idle" };
    }
    return { status: "idle" };
}

let unlistenState: (() => void) | null = null;

onMounted(async () => {
    await getEngineConfig();
    unlistenState = await listen("listen_state", (event) => {
        const st = parseListenState(event.payload);
        listenStatus.value = st.status;
        if (st.error) listenError.value = st.error;
        isEngineRunning.value = st.status === "running";
    });
});

onUnmounted(() => {
    unlistenState?.();
});

// 复制局面（get_current_fen -> 剪贴板）
async function copyFen() {
    try {
        const result = (await invoke("get_current_fen")) as [string, string] | string;
        const fen = Array.isArray(result) ? result[0] : result;
        // 非安全上下文（http 部署）clipboard 可能不可用：textarea + execCommand 降级
        if (navigator.clipboard?.writeText) {
            await navigator.clipboard.writeText(fen);
        } else {
            const ta = document.createElement("textarea");
            ta.value = fen;
            ta.style.position = "fixed";
            ta.style.opacity = "0";
            document.body.appendChild(ta);
            ta.select();
            document.execCommand("copy");
            ta.remove();
        }
        message.success("已复制 FEN 局面");
    } catch (e) {
        dialog.error({
            title: "复制失败",
            content: "复制局面失败: " + String(e),
            positiveText: "确定",
        });
    }
}

async function stopListen() {
    if (isPending.value) return;
    isPending.value = true;
    try {
        await invoke("stop_listen");
        isEngineRunning.value = false;
        listenStatus.value = "idle";
    } catch (e) {
        console.error("停止监听失败:", e);
        dialog.error({
            title: "错误",
            content: "停止监听失败: " + String(e),
            positiveText: "确定",
        });
    } finally {
        isPending.value = false;
    }
}

interface WindowItem {
    id: number;
    title: string;
    app_name: string;
    width: number;
    height: number;
}

async function startListen() {
    if (isPending.value) return;
    isPending.value = true;
    try {
        const windows: WindowItem[] = await invoke("list_windows");

        if (windows.length === 0) {
            dialog.warning({
                title: "警告",
                content: "没有找到可用的窗口",
                positiveText: "确定",
                showIcon: true,
            });
            return;
        }

        const selectedWindowId = ref<number | null>(null);
        const searchQuery = ref("");

        // 按标题自然排序 + 搜索过滤
        const filteredWindows = computed(() => {
            const sorted = [...windows].sort((a, b) =>
                a.title.localeCompare(b.title, undefined, { numeric: true, sensitivity: "base" })
            );
            if (!searchQuery.value) return sorted;
            const query = searchQuery.value.toLowerCase();
            return sorted.filter(
                (w) => w.title.toLowerCase().includes(query) || w.app_name.toLowerCase().includes(query)
            );
        });

        // 键盘选择辅助
        const selectByKey = (e: KeyboardEvent, w: WindowItem) => {
            if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                selectedWindowId.value = w.id;
            }
        };

        dialog.info({
            title: "选择要监听的窗口",
            class: "window-select-dialog",
            content: () =>
                h(NFlex, { vertical: true, style: "gap: 16px" }, [
                    h(NInput, {
                        clearable: true,
                        placeholder: "搜索窗口...",
                        "onUpdate:value": (val) => (searchQuery.value = val),
                        style: "width: 100%",
                    }),
                    h(NScrollbar, { style: "max-height: 300px" }, [
                        filteredWindows.value.length > 0
                            ? h(
                                  NSpace,
                                  { vertical: true, size: "small" },
                                  filteredWindows.value.map((w) =>
                                      h(
                                          NCard,
                                          {
                                              hoverable: true,
                                              size: "small",
                                              bordered: true,
                                              tabindex: 0,
                                              role: "option",
                                              "aria-selected": selectedWindowId.value === w.id,
                                              class:
                                                  selectedWindowId.value === w.id ? "selected-window" : "",
                                              onClick: () => (selectedWindowId.value = w.id),
                                              onKeydown: (e: KeyboardEvent) => selectByKey(e, w),
                                          },
                                          {
                                              default: () => [
                                                  h(NFlex, { align: "center", justify: "space-between" }, [
                                                      h("div", [
                                                          h("div", { class: "window-title" }, w.title),
                                                          h("div", { class: "window-app" }, [
                                                              w.app_name,
                                                              h(
                                                                  NTag,
                                                                  {
                                                                      size: "tiny",
                                                                      type: "info",
                                                                      style: "margin-left: 8px",
                                                                  },
                                                                  { default: () => `${w.width}×${w.height}` }
                                                              ),
                                                          ]),
                                                      ]),
                                                      h(
                                                          NButton,
                                                          {
                                                              tertiary: true,
                                                              circle: true,
                                                              type:
                                                                  selectedWindowId.value === w.id
                                                                      ? "primary"
                                                                      : "default",
                                                              size: "small",
                                                          },
                                                          {
                                                              default: () =>
                                                                  selectedWindowId.value === w.id ? "✓" : "",
                                                          }
                                                      ),
                                                  ]),
                                              ],
                                          }
                                      )
                                  )
                              )
                            : h(NEmpty, { description: "没有找到匹配的窗口" }),
                    ]),
                ]),
            positiveText: "确定",
            negativeText: "取消",
            style: "max-width: 500px",
            maskClosable: false,
            onPositiveClick: async () => {
                if (!selectedWindowId.value) {
                    dialog.warning({
                        title: "提示",
                        content: "请先选择一个窗口",
                        positiveText: "确定",
                    });
                    return false;
                }

                const window = windows.find((w) => w.id === selectedWindowId.value);
                if (window) {
                    try {
                        await invoke("start_listen", { target: window });
                        isEngineRunning.value = true;
                        listenStatus.value = "running";
                        message.success(`已监听: ${window.title}`);
                    } catch (error) {
                        dialog.error({
                            title: "错误",
                            content: "启动监听失败: " + String(error),
                            positiveText: "确定",
                        });
                    } finally {
                        isPending.value = false;
                    }
                }
            },
        });
    } catch (error) {
        console.error("启动监听失败:", error);
        dialog.error({
            title: "错误",
            content: "启动监听失败: " + String(error),
            positiveText: "确定",
        });
    } finally {
        isPending.value = false;
    }
}

// 配置统一保存：本地暂存 -> 批量 invoke -> 反馈/还原
async function saveEngineConfig() {
    try {
        await invoke("set_engine_depth", { depth: config.value.depth });
        await invoke("set_engine_time", { time: config.value.time });
        await invoke("set_engine_threads", { num: config.value.threads });
        await invoke("set_engine_hash", { size: config.value.hash });
        await invoke("set_chessdb", {
            enabled: config.value.chessdb_enabled,
            timeout: config.value.chessdb_timeout,
        });
        savedConfig.value = { ...config.value };
        if (isEngineRunning.value) {
            // 引擎运行中：提示需重载生效，抽屉保留以便一键重载
            message.info("配置已保存，正在监听中——点「重载引擎」后新参数才生效");
        } else {
            message.success("配置已保存");
            showEngineConfig.value = false;
        }
    } catch (e) {
        config.value = { ...savedConfig.value };
        dialog.error({
            title: "保存失败",
            content: "配置保存失败，已还原: " + String(e),
            positiveText: "确定",
        });
    }
}

// 重载引擎（配置热生效）
async function reloadEngine() {
    try {
        await invoke("reload_engine");
        message.success("引擎已重载");
    } catch (e) {
        dialog.error({
            title: "重载失败",
            content: "引擎重载失败: " + String(e),
            positiveText: "确定",
        });
    }
}

async function getEngineConfig() {
    try {
        const result: EngineConfig = await invoke("get_engine_config");
        config.value = {
            ...result,
            time: Number((result.time / 1000).toFixed(1)),
        };
        savedConfig.value = { ...config.value };
    } catch (e) {
        dialog.error({
            title: "错误",
            content: "读取引擎配置失败: " + String(e),
            positiveText: "确定",
        });
    }
}

async function toggleEngine() {
    if (isEngineRunning.value) {
        await stopListen();
    } else {
        await startListen();
    }
}
</script>

<template>
    <n-card class="toolbar" :bordered="false" size="small">
        <n-space vertical size="small">
            <n-flex align="center" justify="space-between" wrap>
                <n-text strong class="mode-label">连线分析</n-text>

                <n-space align="center">
                    <n-tag
                        :type="listenStatus === 'running' ? 'success' : listenStatus === 'error' ? 'error' : 'default'"
                        size="small"
                        data-testid="listen-status"
                    >
                        {{ listenStatus === "running" ? "运行中" : listenStatus === "error" ? "错误" : "空闲" }}
                    </n-tag>

                    <n-tooltip trigger="hover" placement="bottom">
                        <template #trigger>
                            <n-button
                                circle
                                size="small"
                                :type="isEngineRunning ? 'error' : 'primary'"
                                :loading="isPending"
                                :disabled="isPending"
                                data-testid="toggle-engine"
                                aria-label="启动或停止引擎"
                                @click="toggleEngine"
                            >
                                {{ isEngineRunning ? "停" : "启" }}
                            </n-button>
                        </template>
                        {{ isEngineRunning ? "停止引擎" : "启动引擎" }}
                    </n-tooltip>

                    <n-tooltip trigger="hover" placement="bottom">
                        <template #trigger>
                            <n-button
                                circle
                                size="small"
                                type="info"
                                data-testid="open-config"
                                aria-label="引擎配置"
                                @click="showEngineConfig = true"
                            >
                                配
                            </n-button>
                        </template>
                        引擎配置
                    </n-tooltip>

                    <n-divider vertical />

                    <n-tooltip trigger="hover" placement="bottom">
                        <template #trigger>
                            <n-button
                                circle
                                size="small"
                                type="success"
                                data-testid="open-image"
                                aria-label="图片识别（开发中，需模型）"
                                disabled
                            >
                                识
                            </n-button>
                        </template>
                        图片识别（开发中，需模型）
                    </n-tooltip>

                    <n-tooltip trigger="hover" placement="bottom">
                        <template #trigger>
                            <n-button
                                circle
                                size="small"
                                type="warning"
                                data-testid="copy-fen"
                                aria-label="复制局面 FEN"
                                @click="copyFen"
                            >
                                复
                            </n-button>
                        </template>
                        复制局面
                    </n-tooltip>
                </n-space>
            </n-flex>

            <n-text v-if="listenStatus === 'idle' && !isEngineRunning" depth="3" class="guide-text">
                启动引擎后将自动识别棋盘并显示最佳招法
            </n-text>
            <n-text v-if="listenStatus === 'error'" type="error" class="guide-text">
                监听错误: {{ listenError || "识别失败，请检查窗口与模型资源" }}
            </n-text>
        </n-space>

        <n-drawer v-model:show="showEngineConfig" :width="320" placement="right">
            <n-drawer-content title="引擎配置">
                <n-form :model="config" label-placement="left" label-width="90">
                    <n-form-item label="深度">
                        <n-input-number
                            v-model:value="config.depth"
                            button-placement="both"
                            :min="1"
                            :max="200"
                            style="width: 130px"
                        />
                    </n-form-item>
                    <n-form-item label="时间(秒)">
                        <n-input-number
                            v-model:value="config.time"
                            button-placement="both"
                            :step="0.5"
                            :precision="1"
                            :min="0.5"
                            :max="120"
                            style="width: 130px"
                        />
                    </n-form-item>
                    <n-form-item label="线程数">
                        <n-input-number
                            v-model:value="config.threads"
                            button-placement="both"
                            :min="1"
                            :max="64"
                            style="width: 130px"
                        />
                    </n-form-item>
                    <n-form-item label="哈希表(m)">
                        <n-input-number
                            v-model:value="config.hash"
                            button-placement="both"
                            :min="32"
                            :max="102400"
                            style="width: 130px"
                        />
                    </n-form-item>
                    <n-form-item label="启用云库">
                        <n-switch v-model:value="config.chessdb_enabled" />
                    </n-form-item>
                    <n-form-item label="云库超时(s)">
                        <n-input-number
                            v-model:value="config.chessdb_timeout"
                            :disabled="!config.chessdb_enabled"
                            :min="1"
                            :max="60"
                            :step="1"
                            style="width: 130px"
                        />
                    </n-form-item>
                </n-form>
                <n-space style="margin-top: 16px" justify="end">
                    <n-button data-testid="reload-engine" @click="reloadEngine">重载引擎</n-button>
                    <n-button type="primary" data-testid="save-config" @click="saveEngineConfig">
                        保存
                    </n-button>
                </n-space>
            </n-drawer-content>
        </n-drawer>
    </n-card>
</template>

<style scoped>
.toolbar {
    width: 100%;
    padding: 8px;
    border-radius: 8px;
}

.mode-select {
    width: 120px;
}

.guide-text {
    display: block;
    font-size: 12px;
    padding-left: 2px;
}

:deep(.n-button) {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 44px;
    min-height: 44px;
    transition: all 0.3s;
}

:deep(.n-button:hover) {
    transform: translateY(-2px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.window-title) {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 360px;
}

:deep(.window-app) {
    font-size: 12px;
    color: #595959;
    margin-top: 4px;
    display: flex;
    align-items: center;
}

:deep(.selected-window) {
    border-color: var(--primary-color) !important;
    background-color: rgba(var(--primary-color-rgb), 0.05);
}

:deep(.n-card:focus-visible) {
    outline: 2px solid var(--primary-color);
    outline-offset: 2px;
}
</style>



