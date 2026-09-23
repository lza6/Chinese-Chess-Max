# 验证日志 / 记忆点（VERIFICATION-LOG）

> 用途：记录已跑通的检查与结论，避免每次重复空跑；改动某块时先看“范围”列。

## 最近一次全量（date: 手写提交日）
| 范围 | 命令 | 结果 | 备注 |
|---|---|---|---|
| 前端质量 | format:check / lint / typecheck | ✅ | ESLint warnings=0；Prettier 4sp/endOfLine auto |
| 单元覆盖 | test:coverage | ✅ | Lines 59.9 · Branches 43.1 · Funcs 48.5 · Stmts 60%（8+4 用例） |
| 构建 | build | ✅ | vite 6.4.3，dist gzipJS≈144KB |
| E2E | e2e | ✅ | Chromium 真渲染，含无未捕获异常 |
| 契约 | check:contract / Contract.test.ts | ✅ | 10 命令 / 4 事件 |
| Rust 格式 | cargo fmt --check | ✅ | 已 cargo fmt 规范化 |
| Rust 编译 | cargo check/clippy | ⛔ 阻断 | 缺 `libs/large.onnx`（include_bytes） |
| 上游信息 | rg -i atopx | 0 命中 | — |
| 版本 | 4 处 version | 0.2.x 一致 | package/Cargo/tauri×3 |

## 已修好、别重复踩
- 子代理写盘不可靠 → 关键代码我自写自验（本会话）。
- Vitest 5 ↔ vite<6.4 解析炸 → 已锁 vite@^6.4。
- Playwright config 必须放根目录（不是 e2e/）。
- CRLF 源文件导致 Prettier 全库红 → `.prettierrc endOfLine:auto`。
- naive-ui footer 不渲 `<footer>` → 测试用 `a[href*=…]` 选择器。

## 改到这块时优先读
- 引擎/识别的性能与行为 → 关注 `server/src/yolo.rs`、`chess.rs`、`worker.rs`。
- 前端 UI/UX → `src/components/`（三组件在 .prettierignore，注意别被格式门禁误杀）。
- 命令/事件 → `docs/CONTRACT.md` 单测 Contract.test.ts。
## CI 实测踩坑（2026-09-23 线上 runner 验证）
1. Windows runner 默认 shell=pwsh：bash/compgen if [ -f .. ] 步骤必须显式 shell: bash。
2. YAML 里  易被转义搞坏 → 用 V= 再拼 tag。
3. softprops/action-gh-release 对已存在 tag 会报错 → 加 gh release view 幂等门。
4. Playwright 本地残留 dev server 会被 reuse → 明显卡死/旧配置时 Stop-Process 清 5173 再跑。
