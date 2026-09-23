// 前后端 Tauri 契约回归测试（仓库级）：防止命令/事件改名时前后端静默脱节。
import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();

function listSrc(dir: string): string[] {
    const out: string[] = [];
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, e.name);
        if (e.isDirectory()) {
            if (["node_modules", "dist"].includes(e.name)) continue;
            out.push(...listSrc(full));
        } else if (/\.(vue|ts|tsx)$/.test(e.name)) {
            out.push(full);
        }
    }
    return out;
}

function listRs(dir: string): string[] {
    const out: string[] = [];
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, e.name);
        if (e.isDirectory()) out.push(...listRs(full));
        else if (e.name.endsWith(".rs")) out.push(full);
    }
    return out;
}

function matchesToSet(files: string[], re: RegExp, out = new Set<string>()) {
    for (const f of files) {
        const text = fs.readFileSync(f, "utf8");
        for (const m of text.matchAll(re)) out.add(m[1]);
    }
    return out;
}

describe("前后端 Tauri 契约", () => {
    const lib = fs.readFileSync(path.join(root, "server/src/lib.rs"), "utf8");
    const block = lib.match(/generate_handler\s*!?\s*\[([\s\S]*?)\]/)?.[1] ?? "";
    const backendCommands = new Set(
        block
            .split(/[,\n]+/)
            .map((s) => s.trim())
            .filter(Boolean)
            .map((s) => s.split(":").pop()!),
    );

    const srcFiles = listSrc(path.join(root, "src"));
    const frontendInvoke = matchesToSet(srcFiles, /invoke\(\s*["']([^"']+)["']/g);
    const frontendListen = matchesToSet(srcFiles, /\blisten\(\s*["']([^"']+)["']/g);
    const frontendEmit = matchesToSet(srcFiles, /\bemit\(\s*["']([^"']+)["']/g);

    const backendEvents = matchesToSet(
        listRs(path.join(root, "server/src")),
        /\.emit\(\s*["']([^"']+)["']/g,
    );

    it("前端 invoke 的命令均已注册到后端 generate_handler", () => {
        const missing = [...frontendInvoke].filter((c) => !backendCommands.has(c));
        expect(missing, `前端调用后端未注册命令: ${missing.join(", ")}`).toEqual([]);
    });

    it("前端监听的事件必须有发射方（后端 emit 或前端自身 emit）", () => {
        const orphan = [...frontendListen].filter(
            (ev) => !backendEvents.has(ev) && !frontendEmit.has(ev),
        );
        expect(orphan, `前端监听无发射方的事件: ${orphan.join(", ")}`).toEqual([]);
    });

    it("后端发射的事件应被前端消费", () => {
        const unconsumed = [...backendEvents].filter((ev) => !frontendListen.has(ev));
        expect(unconsumed, `后端发射但前端未监听: ${unconsumed.join(", ")}`).toEqual([]);
    });

    it("set_engine_time 单位契约：前端传秒，后端 ×1000 存毫秒", () => {
        const tb = fs.readFileSync(path.join(root, "src/components/Toolbar.vue"), "utf8");
        expect(tb).toContain('invoke("set_engine_time", { time: config.value.time })');
        const cfg = fs.readFileSync(path.join(root, "server/src/config.rs"), "utf8");
        expect(cfg).toMatch(/time\s*\*\s*1000\.0/);
    });
});
