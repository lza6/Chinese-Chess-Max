# Spec V2：终局闭环审计增量规范（Spec Kit / spec-driven-development）

> 角色：项目主规格（本轮增量，覆盖 docs/SPEC.md 未覆盖的缺陷面）
> 版本：v0.2.3+ · 仓库：lza6/Chinese-Chess-Max · 交付分支：main
> 方法：本机 addy-spec-driven-development（SKILL.md，无需联网重装）

## 1. Objective
- 产品：中国象棋桌面分析工具（屏幕识别到 Pikafish 引擎到中文招法建议），已部署生产。
- 本轮目标：以即将交付真实用户的标准做终局审计，发现并修复真实缺陷，不伪造完成。
- 用户：在线象棋玩家；成功标准 = 核心链路真实可用 + 无假功能 + 错误可反馈可恢复 + 文档与实现一致。

## 2. 假设（ASSUMPTIONS，若有误请纠正）
1. 这是 Tauri 2 桌面应用（Windows/macOS/Linux）+ 独立 Web 构建；无云端后端。
2. 部署载体以桌面安装包为主；Web 构建静态托管即可用。
3. 引擎（Pikafish）与识别（YOLOv8/ONNX）为本机资源；无账号/订阅/云收费链路（chessdb.cn 为公共查询服务）。
4. libs/large.onnx（YOLOv8 模型，Rust 编译期 include_bytes 内嵌）是后端编译硬依赖——仓库当前缺失（阻塞项，需用户提供）。
5. 桌面 Camera/引擎原生链路与 GPU 推理性能需目标机型实测。

## 3. 本轮已知缺口（自审计初步发现，待子代理审计扩充）
| 严重级 | 缺口 | 位置 | 状态 |
|---|---|---|---|
| P1 | Analyse.vue 滚动失效：模板 ref 与 script 变量不匹配 | src/components/Analyse.vue | 待修 |
| P1 | analyse() pvs.first().unwrap() 云库 Success 但 pvs 空导致后台线程 panic | server/src/worker.rs | 待修 |
| P1 | analyze_board 持 config.read() 锁跨引擎搜索（最长 movetime）导致写锁阻塞 | server/src/worker.rs | 待修 |
| P1 | stop_listen join().unwrap() 无超时，引擎挂起卡死 | server/src/worker.rs | 待修 |
| P1 | start_listen / ListenWindow::new / predict 多处 unwrap 无错误路径 | worker.rs, listen.rs | 待修 |
| P2 | copy_fen() 空实现（假功能）；识别/复制按钮 disabled | src/components/Toolbar.vue | 待修/如实标注 |
| P2 | 窗口选择对话框搜索框可能非响应式（content 函数静态渲染） | src/components/Toolbar.vue | 待验证 |
| P2 | Config::load 解析失败直接删除用户配置；save 无原子写 | server/src/config.rs | 待修 |
| P2 | chessdb.rs 调试 println 残留 | server/src/engine/chessdb.rs | 待修 |
| P2 | 无 CHANGELOG / 无 ADR / 无文档索引 | docs/ | 待补 |

## 4. Commands（全量，与 package.json 一致）
- 安装：pnpm install
- 开发：pnpm dev（Vite 1420；Web 用 --port）
- 构建：pnpm build（vue-tsc --noEmit + vite build 到 dist/）
- 类型：pnpm typecheck
- Lint：pnpm lint（warnings=0 即失败）
- 格式：pnpm format:check / pnpm format
- 单测：pnpm test / pnpm test:coverage（阈值 Lines50/Branches40/Funcs40/Stmts50）
- E2E：pnpm exec playwright install chromium 然后 pnpm e2e
- 契约：pnpm check:contract
- Rust：cd server 然后 cargo fmt --check（缺 large.onnx 时 clippy/check 被 gate 跳过）

## 5. 代码风格
- 前端：Vue3 script setup lang=ts + Naive UI；命名 camelCase；新代码必须过 format:check。
- 后端：Rust edition 2024；错误用 Result 冒泡；日志用 tracing（debug/info/error），禁止 println 调试残留。
- 契约：前端 invoke 命令/事件必须与后端 generate_handler 一致（check-contract 强制）。

## 6. 测试策略
- 单测：Vitest + jsdom（tests/），覆盖工具/契约/状态；覆盖率阈值如上。
- E2E：Playwright Chromium（e2e/），mock Tauri IPC，覆盖首页渲染 + 核心用户路径。
- Rust：cargo test（缺 large.onnx 时部分测试 gate）。
- 验收：每条修复必须有验证命令/证据，记入 VERIFICATION-LOG。

## 7. 边界（Boundaries）
- Always：提交前跑 format/lint/typecheck/test；修复必加验证；改命令/事件同步 CONTRACT.md 与 check-contract；敏感信息不入库。
- Ask first：改 CI 结构、加依赖、改数据模型、改发布流程、动 libs/ 资源。
- Never：伪造测试通过；把 mock 当真实；删用户配置数据；强制推送；上传模型文件到公开仓库。

## 8. 成功标准（可测试）
1. 本轮所有 P0/P1 缺陷修复且附验证证据（命令输出）。
2. pnpm format:check / lint / typecheck / test:coverage / build / check:contract / e2e 全绿。
3. 无假功能：copy_fen 实现或如实标注不可用；README 与真实功能一致。
4. 文档同步：SPEC-V2 / VERIFICATION-LOG / workflow_status / CONTRACT 与代码一致。
5. 提交并推送 main，Release 产物 dist.zip 更新（如版本 bump）。

## 9. Open Questions
- libs/large.onnx 何时由用户提供？（阻塞 Rust 编译/安装包）
- 连线对战/人机对弈是否列为后续里程碑？（当前 disabled，需产品决策）
