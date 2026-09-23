# Spec：Chinese-Chess-Max 终端闭环审计基线

> 角色：项目主规格（Spec-Kit / spec-driven-development）
> 版本：v0.2.3 基线 · 仓库：lza6/Chinese-Chess-Max·交付分支：main

## 1. Objective
- 产品：中国象棋桌面分析工具（实时识别屏幕棋盘 → 引擎分析 → 中文招法建议）。
- 用户：在线平台象棋玩家；目标：提升识别的准确性、分析响应与上手效率。
- Success：全部 Keep/Maintain 验收门禁通过；本地可复现构建/测试；前后端契约无漂移；新环境按 SOP 能跑起来。

### 假设（ASSUMPTIONS，若有误请纠正）
1. 这是一个 **Tauri 2 桌面应用**（Windows/macOS/Linux）+ 可独立分发的 **静态 Web 构建**，不存在云端后端服务器。
2. 部署载体以桌面安装包为主；Web 构建可在任意静态托管/CDN 直接上线。
3. 引擎（Pikafish）与识别（YOLOv8/ONNX）为本机资源；**无任何账号/订阅/云收费链路**。
4. `libs/large.onnx`（YOLOv8 模型，Rust 编译期 `include_bytes!` 内嵌）是后端编译硬依赖——仓库当前缺失。

## 2. Tech Stack
- 前端：Vue 3.5 / TypeScript / Naive UI / Vite 6.4（含 vue-tsc 2.2）
- 桌面：Tauri 2.5 + Rust（edition 2024）· ONNX Runtime ort 2.0-rc9 · xcap 截屏 · reqwest+tokio · pikafish
- 质量：ESLint 10 (flat) · Prettier 3.9 · Vitest 5 + jsdom + @vue/test-utils · Playwright 1.63
- 包管理：pnpm 10

## 3. Commands
```bash
pnpm install                          # 安装依赖（写入 pnpm-lock.yaml）
pnpm dev                              # Vite 开发服务（Tauri 用 1420；Web 用 --port）
pnpm build                            # vue-tsc 类型检查 + vite 生产构建 -> dist/
pnpm typecheck                        # vue-tsc --noEmit
pnpm lint                             # ESLint（warnings=0 即失败）
pnpm format:check                     # Prettier 校验
pnpm test                             # Vitest 单测
pnpm test:coverage                    # Vitest + 覆盖率阈值（Lines 50/Branches 40/Funcs 40/Stmts 50）
pnpm e2e                              # Playwright 真浏览器（webServer 自动起 dev）
pnpm check:contract                   # 前后端 Tauri 命令/事件契约校验（防漂移）
pnpm exec playwright install chromium # 首次跑 e2e 前安装浏览器
cd server && cargo fmt --check        # Rust 格式门禁
cd server && cargo clippy --all-targets -- -D warnings   # 需 libs/large.onnx 存在
cd server && cargo check --all-targets                    # 需 libs/large.onnx 存在
```

## 4. Project Structure
```
src/                Vue 前端（App.vue / components/）
src/components/     Toolbar(工具栏) · Chessboard(棋盘) · Analyse(局面分析)
server/             Rust/Tauri 后端（commands / engine / listen / worker / yolo / chess / config）
libs/               运行与模型资源（pikafish、onnx）
tests/              Vitest 单测 + Tauri IPC mock
e2e/                Playwright E2E（frontend + 桥 mock）
scripts/ci/         check-resources · check-contract
docs/               SPEC / CONTRACT / SOP / SCALABILITY-REVIEW / VERIFICATION-LOG / CI-CD
```

## 5. Code Style
- 前端：4 空间缩进、双引号、分号、尾逗号（见 `.prettierrc.json`）。
- ESLint：vue/typescript-eslint 推荐集保留真实问题规则；命名类/风格类按需豁免（详见 eslint.config.js 注释）。
- Rust：`cargo fmt` 规范（上游遗留 3 个 Vue 大组件在 .prettierignore 豁免，避免大范围重排）。

## 6. Testing Strategy
- 单元/组件：`tests/`（Vitest + jsdom，Tauri API mock），覆盖 App/Toolbar/Analyse/Chessboard。
- 仓库级契约回归：`tests/Contract.test.ts`（命令/事件集合 + 时间单位规则）。
- E2E：`e2e/`（Playwright，注入 `__TAURI_INTERNALS__` mock）。
- 覆盖率阈值：Lines 50 / Branches 40 / Funcs 40 / Stmts 50（可上调）。

## 7. Boundaries
- 始终：提交前跑 `pnpm format:check lint typecheck test`；遵循契约；输入校验。
- 先问：改 DB/依赖/CI 配置；删除现有测试；改 tauri conf 标识。
- 永不：提交密钥；改动后才补 types；丢弃 RELEASED 资源。

## 8. Success Criteria
- [x] 全门禁绿（format/lint/typecheck/coverage/build/e2e/cargo fmt/contract）。
- [x] 关键契约 bug（set_engine_time 单位）已修复并有回归测试。
- [x] 文档（SPEC/CONTRACT/SOP/SCALABILITY/VERIFICATION/CI-CD）齐备且与代码一致。
- [ ] Rust 编译可验证：需 `libs/large.onnx` 入库（阻断项，见 CONTRACT/SCALABILITY）。

## 9. Open Questions（需人工拍板）
1. 是否补充 y简 `libs/large.onnx`（YOLOv8 模型）到仓库？（当前缺失导致 Rust 无法编译/发布安装包）
2. 是否需要真正的“人机对弈/连线对战”？（前端两个模式当前为 disabled）
3. 是否需要多云/高可用（如 Web 托管+CDN）？（当前是桌面优先+静态 Web）