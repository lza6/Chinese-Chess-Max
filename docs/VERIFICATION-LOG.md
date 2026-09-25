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

## 2026-09-26 第四轮终局闭环审计（v0.2.4 已发布 + v0.2.5 进行中）
- 版本：v0.2.4（8acfac6 YOLOv11 契约修复 + 20eb522 版本 bump）；v0.2.5 进行中。
- Rust 新增命令：get_current_fen / export_game / load_history / clear_history / review_step / human_move / reload_engine（接通）；新增事件 listen_state / review_state → 契约 **16 命令 / 6 事件** 双向对齐（check:contract + Contract.test）。
- Rust 可靠性：listen.rs 全链 Result 化（Window::new/ListenWindow::new/capture 不 panic）；engine/mod.rs Engine::new/reload/setoption/write_command Result 化 + bestmove 容错 + parse_line 坏数字容错；worker.rs start_listen predict.map_err、emit 失败不 panic、confirm_board 截屏失败兜底。
- 全门禁：cargo fmt / clippy(-D warnings) / test --lib **10/10 全绿**；pnpm format/lint/typecheck/test:coverage/build/check:contract 全绿；E2E（Chromium）覆盖窗口选择→启动→analyse→停止、配置保存反馈、复制局面、复盘/导出、响应式断点。
- 前端同步：复制局面（get_current_fen + 剪贴板）、配置保存反馈 + 失败还原、重载引擎、窗口排序 + 键盘可选、监听状态徽标、空状态引导、日志时间戳、高亮组件内状态、复盘面板、导出按钮、棋盘坐标数据生成、App Flex 响应式 + 触控 ≥44px。
- 文档同步：CHANGELOG v0.2.4/v0.2.5、CONTRACT 命令/事件补全、REQUIREMENT-MATRIX 刷新、AUDIT-V2 状态、docs/INDEX.md 新增、README 链接 + startup.png 改名、.gitignore 补 *.orig/server/AppData//server/gen/。
- 清理：删除 4 个 server/src/*.rs.orig 残留。
- 待落地：release.yml 双轨统一（C4）；D1-D4 外部阻塞不变（large.onnx 真实模型、onnxruntime DLL）。

## 2026-09-26 第五轮（v0.2.6：CI/CD 双轨统一 + CodeQL 资源 gate）
- **C4 落地**：release.yml 三处内联资源检查（check-linux/check-win-cpu/check-win-gpu）改为调用 `scripts/ci/check-resources.{sh,ps1}`，消除双轨漂移；脚本本地验证缺资源 exit 1 / 存在 exit 0。
- **C3 落地**：security.yml CodeQL job 增加 `check-model` gate，缺 libs/large.onnx 时 Rust CodeQL 跳过并输出 notice（不假绿、不失败）。
- YAML 语法验证：release.yml / security.yml 均通过 pyyaml 解析。
- 阻塞不变：libs/large.onnx 真实模型缺失；onnxruntime DLL 未入库。

## 2026-09-26 第六轮（v0.2.7：终局闭环总审计）
- 4 Agent 深挖（Rust 健壮性/前端真实路径/生产可交付性）+ 主控复验 → 修复 P0/P1 共 6 类：
  1. parse_iccs 严格校验（非法 ICCS 不 panic，human_move 安全）
  2. analyse 非法 pv 不锁中毒
  3. 历史 FEN 双重应用修复（record_move 传走子前局面）+ 引擎走子单次记录
  4. 初始局面统一 red_startpos（标准方向，复盘/复制/人机不再倒置）
  5. config save/load 防 panic（RwLock 不中毒）
  6. yolo session 惰性 Result + NaN 安全
- 前端：clear_history 接入 + clipboard 降级 + 复盘显示一致性
- 验收：cargo fmt/clippy/test 14/14；pnpm lint/typecheck/test 24/24/build/e2e 6/6；契约 16/15/6
- 版本 0.2.6 → 0.2.7

## 2026-09-26 第七轮（v0.2.8：架构资产 + 部署资源 SOP + 回归确认）
- 新增 docs/adr/ 4 个 ADR（模型 gate / 无后端架构 / FEN 单一事实 / 契约防漂移）
- docs/SOP.md 补「资源获取与放置」（large.onnx / onnxruntime 获取路径 + 放置说明）
- docs/starup.png → startup.png（C8 拼写修正）
- 全量回归确认 v0.2.7 修复无回归：cargo 14/14、前端 24/24、E2E 6/6、契约 16/15/6
- 版本 0.2.7 → 0.2.8

## 2026-09-26 第八轮（v0.2.9：引擎超时 + 云库重试 + 文档一致性）
- wait_until 加 10s 时间预算（防引擎持续输出时无限阻塞）
- chessdb 瞬态失败重试 1 次（200ms 间隔）再降级引擎
- README 功能矩阵修正（复制局面已实现、补复盘/导出行）；SPEC 版本刷新
- cargo fmt/clippy/test 14/14 全绿

## 2026-09-26 第九轮（v0.2.10：终局审计硬性产出 + 内部防御）
- 需求追踪矩阵刷新至 v0.2.9 + A-I 九层补位检查（新增章节）
- SELF-REVIEW 第七轮反向审判（攻击 v0.2.5-v0.2.9 伪闭环）
- board_move_chinese P/p 分支 position().unwrap() 兜底；fen_to_board 全防御化
- cargo fmt/clippy/test 15/15 全绿

## 2026-09-26 第十轮（v0.2.11：前端 A 层补位）
- Analyse 复盘跳步失败可见错误反馈（review-error），不再仅 console.error
- 前端 25 测试 + E2E 6/6 + build 全绿
