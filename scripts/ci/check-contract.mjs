// scripts/ci/check-contract.mjs
// 前后端契约防坑：校验前端 invoke(命令名) 与后端 generate_handler 注册一致，
// 并校验事件 listen/emit 与后端 app.emit 双向对齐。退出码非 0 表示契约漂移。
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const issues = [];

function walk(dir, ext, out = []) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, e.name);
        if (e.isDirectory()) {
            if (["node_modules", "dist", ".git"].includes(e.name)) continue;
            walk(full, ext, out);
        } else if (e.name.endsWith(ext)) out.push(full);
    }
    return out;
}

// --- 后端命令（generate_handler 注册） ---
const lib = fs.readFileSync(path.join(root, "server/src/lib.rs"), "utf8");
const m = lib.match(/generate_handler\s*!?\s*\[([\s\S]*?)\]/);
if (!m) {
    console.error("FATAL: generate_handler! block not found in server/src/lib.rs");
    process.exit(1);
}
const backendCommands = new Set(
    m[1]
        .split(/[,\n]+/)
        .map((s) => s.trim())
        .filter(Boolean)
        .map((s) => s.split(":").pop()),
);

// --- 前端调用 ---
const frontFiles = walk(path.join(root, "src"), "", []);
let feInvoke = new Set();
let feListen = new Set();
let feEmit = new Set();
for (const f of frontFiles) {
    if (!/\.(vue|ts|tsx|js|jsx)$/.test(f)) continue;
    const text = fs.readFileSync(f, "utf8");
    for (const mm of text.matchAll(/invoke\(\s*["']([^"']+)["']/g)) feInvoke.add(mm[1]);
    for (const mm of text.matchAll(/\blisten\(\s*["']([^"']+)["']/g)) feListen.add(mm[1]);
    for (const mm of text.matchAll(/\bemit\(\s*["']([^"']+)["']/g)) feEmit.add(mm[1]);
}

// --- 后端事件（app.emit） ---
const backendEvents = new Set();
for (const f of walk(path.join(root, "server/src"), "", [])) {
    if (!/\.rs$/.test(f)) continue;
    for (const mm of fs.readFileSync(f, "utf8").matchAll(/\.emit\(\s*["']([^"']+)["']/g))
        backendEvents.add(mm[1]);
}

// 1) 前端调用命中后端命令
for (const cmd of [...feInvoke].sort()) {
    if (!backendCommands.has(cmd)) {
        issues.push(`FRONTEND-CALLS-MISSING: invoke("${cmd}") 在后端 generate_handler 中未注册`);
        files = 1;
    }
}
// 2) 后端注册但前端未调用（可能为保留/未来命令）
for (const c of [...backendCommands].sort()) {
    if (!feInvoke.has(c)) console.log(`WARN unused-backend-command: ${c}（前端未调用，保留）`);
}
// 3) 前端监听的事件应有来源（后端 emit 或前端自身 emit）
for (const ev of [...feListen].sort()) {
    if (!backendEvents.has(ev) && !feEmit.has(ev)) {
        issues.push(`FRONTEND-LISTENS-ORPHAN: listen("${ev}") 无任何发射方`);
    }
}
// 4) 后端发射的事件应被前端消费
for (const ev of [...backendEvents].sort()) {
    if (!feListen.has(ev)) console.log(`WARN backend-event-unconsumed: ${ev}`);
}

console.log("后端命令:", [...backendCommands].sort().join(", "));
console.log("前端调用:", [...feInvoke].sort().join(", "));
console.log("前端监听事件:", [...feListen].sort().join(", "));
console.log("后端发射事件:", [...backendEvents].sort().join(", "));
if (issues.length) {
    issues.forEach((i) => console.error("CONTRACT ✔  " + i));
    process.exit(1);
}
console.log(
    `✔ 契约校验通过（命令 ${backendCommands.size}，前端调用 ${feInvoke.size}，事件 ${backendEvents.size}）`,
);
process.exit(0);
