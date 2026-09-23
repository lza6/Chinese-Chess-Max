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
