# workflow_status.md —— 终局闭环审计节点台账

> 状态：✅ done / 🕐 in-progress / ⛔ blocked / ⚠️ decided-not-now

| 节点              | 说明                 | 状态        | 验收证据                                             |
| ----------------- | -------------------- | ----------- | ---------------------------------------------------- |
| SPEC              | 规格化（Spec-Kit）   | ✅          | docs/SPEC.md                                         |
| 去上游化          | atopx 残留清除/品牌  | ✅          | rg=0; 版本0.2.0→0.2.1                                |
| 契约审计          | 命令/事件一致        | ✅          | docs/CONTRACT.md + check-contract + Contract.test.ts |
| 单位 Bug          | set_engine_time×1000 | ✅ 已修复   | Contract.test + Toolbar.vue                          |
| CI/CD             | ci/security/release  | ✅          | workflows + docs/CI-CD.md                            |
| 测试基建          | Vitest/E2E/lint      | ✅          | 门禁全绿（见验证日志）                               |
| 文档              | SPEC/SOP/…           | ✅          | docs/*.md                                            |
| 后端编译          | cargo check          | ⛔ blocked  | 缺 `libs/large.onnx`；已 gate 防伪造                 |
| 桌面安装包        | release installers   | ⛔ blocked  | 挂 `libs/large.onnx`+onnxruntime`                    |
| 人机对弈/连线对战 | disabled 功能        | ⚠️ 不宜现做 | 需求产品确认                                         |
| 规模项            | Redis/Kafka/sharding | ⚠️ 不适用   | docs/SCALABILITY-REVIEW.md                           |

## Next

1. 人工提供 `libs/large.onnx` → 解锁 cargo 编译 + 出安装包。
2. 如启用 Web 静态托管：把 `dist/` 一键发到托管 → CDN。
3. 想加“人机对弈/连线对战”需产品功能拆解（先出 SPEC 再用 spec-kit 流程）。

## 最新：GitHub 线上验证（2026-09-23）

- CI / Security / Release 三条 workflow 全绿（commit `0d94b5e`）。
- Security 曾两连红已修复：pnpm.overrides(postcss/nanoid) + cargo 定向升级 + osv 官方 CLI 与 CVSS>=7 门禁 + release 幂等 + windows shell:bash。

## 最新：2026-09-23 第二轮终局审计（主控代理复跑）

- 本地真实全量重跑全绿：format/lint/typecheck/test:coverage(14用例)/build/e2e(Chromium)/cargo fmt/check:contract。
- 新修复 3 项：check-contract 脏变量+错误符号、.gitattributes LF 行尾契约、CI e2e 显式契约步骤；CodeQL v3->v4。
- 线上 job 级证据 @364e991：CI/Security/Release 全 success；v0.2.0/v0.2.1 Release 携带 dist.zip。
- 唯一硬阻塞不变：缺 libs/large.onnx（YOLOv8 40-50MB，需用户提供，compile-time include_bytes）。

## 最新：2026-09-24 第三轮终局闭环审计（Orchestrator 主控代理复跑）

- Spec Kit 前置：docs/SPEC-V2.md（增量规范）+ REQUIREMENT-MATRIX.md（需求追踪矩阵）+ SELF-REVIEW.md（反向审判）+ AUDIT-V2.md（4 Agent 综合清单）。
- 4 并行子代理深度审计（Bacon 前端/Galileo Rust/Heisenberg 工程/Raman 产品）→ 交叉印证缺陷清单。
- 本地真实解锁 Rust 编译（占位模型）→ cargo check/clippy/test 全绿 10/10；前端全绿 + E2E 真跑。
- 修复 A/B/C 组 20+ 项（详见 CHANGELOG 0.2.3）。
- 资源阻塞：large.onnx 真实模型缺失；仓库自带 pikafish 引擎损坏（已工作区替换官方版验证可用，未入库）。

| 0.2.4 发布 | YOLOv11 模型契约修复 + 版本 bump | ✅ | 8acfac6 + 20eb522 |
| 0.2.5（进行中） | Rust 可靠性 + 历史/导出/复盘/人机命令 + 前端交互 | 🕐 | CONTRACT 16 命令/6 事件；CHANGELOG 草稿 |

## 最新：2026-09-26 第四轮（v0.2.4 + v0.2.5 进行中）

- v0.2.4：YOLOv11 真实模型输出契约适配（output0 + stride19 无 objectness），版本 bump。
- v0.2.5 后端已落地：新增 get_current_fen / export_game / load_history / clear_history / review_step / human_move 命令 + listen_state / review_state 事件；reload_engine 接通；listen/engine/worker 全链 Result 化（cargo fmt/clippy/test 10/10 全绿）。
- 文档同步：CHANGELOG / CONTRACT / REQUIREMENT-MATRIX / AUDIT-V2 / VERIFICATION-LOG / docs/INDEX / README / .gitignore。
- 待落地：前端复盘/导出/人机交互接线与 E2E；release.yml 双轨统一；D1-D4 外部阻塞。

## 最新：2026-09-26 第五轮（v0.2.6）

- release.yml 资源检查统一为 scripts/ci/check-resources.{sh,ps1}（C4 落地）
- security.yml CodeQL Rust 加 large.onnx 资源 gate（C3 落地）
- 版本 0.2.5 → 0.2.6

## 最新：2026-09-26 第六轮（v0.2.7）

- 终局闭环总审计：P0/P1 修复 6 类（非法输入/锁中毒/FEN 错乱/初始局面方向/config panic/yolo panic）
- Rust 14 测试、前端 24 测试、E2E 6 条全绿；契约 16/15/6
