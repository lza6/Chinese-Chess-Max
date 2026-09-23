import { vi } from "vitest";

/** 事件处理器注册表：用于在测试里手动触发 listen 回调以覆盖组件逻辑 */
export const listeners: Record<string, Array<(payload: any) => any>> = {};

export const invoke = vi.fn(async (cmd: string, _args?: unknown): Promise<any> => {
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
        case "stop_listen":
        case "set_engine_depth":
        case "set_engine_time":
        case "set_engine_threads":
        case "set_engine_hash":
        case "set_chessdb":
        case "start_listen":
            return null;
        default:
            return null;
    }
});

export const convertFileSrc = vi.fn((p: string) => p);

export const listen = vi.fn(async (event: string, handler: (payload: any) => any) => {
    (listeners[event] ??= []).push(handler);
    return () => {};
});

export const emit = vi.fn(async (_event: string, _payload?: unknown) => undefined);

export const once = vi.fn(async () => () => {});
