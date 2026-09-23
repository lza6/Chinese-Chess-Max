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
2. YAML 里 `$(node -p "require(...)")` 易被转义搞坏 → 用 `V=$(node -p "require(...)")` 再拼 tag。
3. softprops/action-gh-release 对已存在 tag 会报错 → 加 gh release view 幂等门。
4. Playwright 本地残留 dev server 会被 reuse → 明显卡死/旧配置时 Stop-Process 清 5173 再跑。

## 安全门禁策略（osv-scanner）
- 门禁：仅 CVSS>=7（High/Critical）阻断；低危/未知通告打印不阻断（与 pnpm audit --audit-level high 一致）。
- 2026-09-23：cargo update 定向升级 bytes/h2/anyhow/memmap2/event-listener/openssl 等，Cargo.lock 59 行小改动后 CVSS>=7=0；剩余 63 条为无 CVSS 的构建期/传递通告。
- 第三方 action google/osv-scanner-action 的 tag 损坏（v1 不存在、v2.6.0 缺 runs）→ 改用官方 CLI（setup-go + go install）。

## 2026-09-23 GitHub 线上三线全绿（本地勿重复跑）
- CI / Security / Release 均 success @ 0d94b5e；改流水线前先看本文档踩坑区。

## 2026-09-23 第二轮终局审计（本地真实重跑 + 新修复）
- 本机全量重跑：format:check / lint / typecheck / test:coverage（5 文件 14 用例，Lines 59.9 Branches 43.1 Funcs 48.5 Stmts 60）/ build（gzip JS 143.9KB）/ e2e（Chromium 1 passed）/ cargo fmt --check 全绿。
- 新发现并修复：check-contract.mjs 错误分支 iles = 1; 未声明变量（漂移时会抛 ReferenceError 而非清晰报错，错误前缀 ✔ 误用）；check-resources.sh CRLF 在 Windows bash 下语法炸；新增 .gitattributes 行尾契约（*.sh/*.mjs/*.js/*.ts/*.vue/*.json/*.yml/*.md 强制 LF）。
- 升级：CodeQL v3 -> v4（github/codeql-action 三处）；CI e2e job 增加显式 pnpm check:contract。
- 资源 gate 本地验证：pikafish 四件套 found；libs/large.onnx missing -> exit 1（正确）。
- GitHub 线上 @364e991 job 级证据：CI changes success（frontend/e2e/backend skipped 为路径过滤正确行为）；Security 五 job（osv-scan/secret-scan/dependency-audit/codeql×2）全 success；Release 三 job（build-linux/build-windows/publish）全 success。

## 2026-09-24 第三轮终局闭环审计（Spec Kit + 4 Agent + 本地真实验证解锁 Rust 编译）
- **重大能力突破**：放置占位 large.onnx 解锁 include_bytes 编译 → 本机真实跑通 cargo check / cargo clippy(-D warnings) / cargo test --lib = 10/10 全绿。
- **关键资源缺陷**：仓库自带 pikafish-windows.exe（2025-01-10, 1.4MB）连自带 nnue 也拒绝加载（引擎启动即退出，生产运行时引擎必死）。官方 2026-09-06 引擎+配套 nnue 验证可用（工作区已替换验证，未入库）。large.onnx 仍需真实模型。
- **Rust 修复（均真实验证）**：引擎 UCI 握手+isready；bestmove EOF 防死循环；read_line lossy 容忍非 UTF-8；analyse pvs 空兜底；config 读锁快照不阻塞写锁；start_listen 防重入(guard.is_some)；stop_listen detach 不 join；ListenWindow Result；chessdb https+tracing+防 panic；config 原子写+解析失败备份；logger try_init 防重复初始化。
- **前端修复（均验证）**：Analyse 滚动 ref/nextTick/高亮清理/空兜底/time 展示；Chessboard 监听 onMounted+onUnmounted 解绑；Toolbar pending 防重复+按钮 loading；假功能(copy_fen/图片识别)如实标注开发中；jsdom scrollTo polyfill。
- **工程修复**：README 诚实化（功能矩阵/下载说明）；CI 过滤补 scripts/ci+security.yml；.gitignore 补 .env*/.zip/.bak-*；CHANGELOG 建立。
- **验收矩阵全绿**：pnpm format/lint/typecheck/test:coverage(14用例)/build/e2e(Chromium)/check:contract + cargo fmt/clippy/test(10/10)。

- 版本 bump 0.2.2 -> 0.2.3（package/Cargo/tauri×3/docs/CHANGELOG）。
