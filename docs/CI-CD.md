# Chinese-Chess-Max CI/CD 流水线设计文档

> 仓库：https://github.com/lza6/Chinese-Chess-Max · 交付分支：`main`
> 技术栈：Vue 3 + TypeScript + Vite + Naive UI（前端） / Tauri 2 + Rust（YOLOv8 + ONNX Runtime + Pikafish 引擎）
> 平台：GitHub Actions

## 1. 流水线总览（ASCII）

```
feature/* ──PR──▶ ┌──────────────── CI (push: main/develop + PR) ────────────────┐
                 │  changes(路径过滤)                                            │
                 │   ├─ frontend: format:check → lint → typecheck → test:coverage │
                 │   │            → build → dist 产物                             │
                 │   ├─ backend (×3 OS): cargo fmt → clippy → cargo check        │
                 │   └─ e2e: playwright (chromium, Tauri 桥 mock)                 │
                 └───────────────────────────────────────────────────────────────┘
                                        │ 全绿
                                        ▼
                 ┌──────────────── Security (每周/PR/push) ──────────────────────┐
                 │  osv-scanner(pnpm + Cargo.lock) → pnpm audit → gitleaks → CodeQL │
                 └───────────────────────────────────────────────────────────────┘
                                        │ 全绿
   merge 到 develop ────────────────▶ staging 预发布（release 流程产物 draft）【可选】
                                        │
   merge 到 main ──────────────────▶ Release workflow
                                        │  environment: production（人工审批门）
                                        ▼
                 ┌──────────────── Release ──────────────────────────────────────┐
                 │  build-linux (deb, 资源 gate)                                  │
                 │  build-windows (CPU/GPU MSI, 资源 gate)                        │
                 │  publish: 打包 dist + 安装包 → GitHub Release（draft）          │
                 │  失败 → Slack 通知                                             │
                 └───────────────────────────────────────────────────────────────┘
```

## 2. 工作流文件与阶段

| 阶段 | 文件 | 触发 | 关键步骤 |
|---|---|---|---|
| Build | `.github/workflows/ci.yml` (frontend job) | push main/develop + PR | `pnpm install --frozen-lockfile` → `pnpm build`（vue-tsc 类型检查 + vite 构建）→ 上传 `dist/` |
| Build | `.github/workflows/ci.yml` (backend job) | 同上 | 三平台矩阵 cargo check --all-targets（Rust 编译验证） |
| Qualité | `ci.yml` (frontend) | 同上 | `pnpm format:check`（Prettier）→ `pnpm lint`（ESLint，warnings=0 即失败）→ `pnpm typecheck`（vue-tsc --noEmit） |
| Tests | `ci.yml` | 同上 | `pnpm test:coverage`（Vitest + jsdom 单测，阈值见 vitest.config.ts）→ `pnpm e2e`（Playwright 真实浏览器，Tauri IPC 桥 mock） |
| Sécurité | `.github/workflows/security.yml` | push main/develop + PR + 每周一 | OSV 依赖扫描（pnpm-lock.yaml + server/Cargo.lock）→ `pnpm audit --audit-level high` → Gitleaks 密钥扫描 → CodeQL（JS/TS + Rust） |
| Déploiement | `.github/workflows/release.yml` | push main + 手动 workflow_dispatch | 见第 3 节 |
| Notifications | `release.yml` 各 job | 失败时 | `rtCamp/action-slack-notify@v2`（可选，配 `SLACK_WEBHOOK_URL` 后启用） |

## 3. 部署与发布策略

- **main 分支**：生产发布唯一来源。`push main` 触发 `release.yml`。
- **人工审批**：`publish` job 声明 `environment: production`。在 GitHub 仓库
  `Settings → Environments → production` 中配置 **Required reviewers**（建议 1 人），
  之后每次发布需审批才创建 Release（draft），人工确认后在 Releases 页面点“Publish”。
- **产物**：
  - `dist.zip` —— 前端 Web 构建（始终产出）；
  - Linux `.deb` —— 需要 `libs/linux/libonnxruntime*.so` 资源存在；
  - Windows CPU/GPU `.msi` —— 需要 `libs/windows-cpu/*.dll` / `libs/windows-gpu/*.dll` 资源存在。
- **资源 gate（不伪造成功）**：`scripts/ci/check-resources.ps1` / `check-resources.sh`
  检查上述资源；缺失时输出 `::notice::` 并跳过对应打包步骤，Release 仍可携带 web 产物发布，
  补齐资源后自动启用安装包构建。
- **回滚策略**：
  1. GitHub Release 支持删除/编辑草稿，未点 Publish 前零影响；
  2. 已发布版本：在 Releases 页面将上一版本标为 Latest，或复用 tag 重新跑
     `workflow_dispatch`（同一 tag 需先删旧 Release 与 tag）；
  3. 代码回滚：`git revert`（推荐，保留历史）提交到 main，重跑发布。

## 4. 环境变量与 Secrets

| 名称 | 位置 | 必填 | 说明 |
|---|---|---|---|
| `GITHUB_TOKEN` | 内置 | 是 | 自动注入，无需配置；权限在 workflow `permissions` 中声明 |
| `SLACK_WEBHOOK_URL` | Actions secrets | 否 | 配了才启用 Slack 失败通知（workflow 顶部 `env` 透传，未配自动跳过） |
| `production` environment | Settings → Environments | 建议 | 配置 Required reviewers 实现人工审批门 |

> 说明：secrets 不能直接在 `if:` 中引用，本仓库做法是在 workflow 顶层
> `env: SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}`，步骤 `if` 用 `env.SLACK_WEBHOOK_URL != ''` 判断。

## 5. 分支管理

| 分支 | 用途 | CI | 发布 |
|---|---|---|---|
| `feature/*` | 新功能开发 | PR 触发 CI（路径过滤） | 无 |
| `develop` | 集成/预发（可选） | push 触发 CI | 可选预发布草稿 |
| `main` | 生产 | push 触发 CI + Security | push 触发 Release（人工审批） |
| `hotfix/*` | 紧急修复 | PR 到 main 触发 CI | 合并 main 后随 Release 发布 |

规范：
- 禁止直接 `git push --force`；禁止 `git add .` 全量提交。
- 提交按主题拆分：`chore(ci): ...` / `test(...): ...` / `docs: ...` 等。
- 合并 main 前先 fetch 远端并核对远端 SHA。

## 6. 性能优化

- **依赖缓存**：`actions/setup-node@v4` 的 `cache: pnpm`（复用 pnpm store）；
  `Swatinem/rust-cache@v2` 缓存 `server/target`（三平台各自独立缓存）。
- **并行**：frontend / backend（3 OS 矩阵）/ e2e 三个 job 并行；
  `fail-fast: false` 保证一个平台失败不影响其余平台产出诊断信息。
- **路径过滤**：`dorny/paths-filter@v3` —— 只改 `server/**` 时前端 job/e2e 不跑，只改 `src/**` 时后端 job 不跑。
- **并发取消**：`concurrency.cancel-in-progress: true`，同分支新 push 自动取消旧 CI，省排队。
- **Release 并发**：`cancel-in-progress: false`，避免发布互相打断。

## 7. 排障指南

| 现象 | 原因 | 处理 |
|---|---|---|
| `ERR_PNPM_OUTDATED_LOCKFILE` / frozen-lockfile 失败 | lockfile 与 package.json 不一致 | 本地 `pnpm install` 后提交更新后的 `pnpm-lock.yaml` |
| `pnpm lint` 报大量格式/风格错 | ESLint 规则与本地风格冲突 | 修改 `eslint.config.js` 规则（不要为过关全关）；业务文件风格跟随 `.prettierrc.json`（4 空格、双引号） |
| `cargo clippy -D warnings` 失败 | 新代码引入 clippy 警告 | `cargo clippy --fix` 或手工修正；本地跑 `cargo fmt --check` 先自查 |
| Rust 缓存损坏 / 编译奇异报错 | target 缓存污染 | 删除 `server/target` 后重跑；Actions 里可清 `rust-cache` 的 key |
| `tauri build` 找不到 libs 资源 | onnxruntime/pikafish 资源未入库 | 补齐 `libs/windows-cpu|windows-gpu/*.dll`、`libs/linux/libonnxruntime*.so` 后提交（注意 .gitignore 的 `*.dll` 规则） |
| Playwright 浏览器缺失：`Executable doesn't exist` | 未装浏览器 | `pnpm exec playwright install chromium`；CI 已带 `--with-deps` |
| `osv-scanner` 报 lockfile 解析失败 | lockfile 路径/格式 | 确认根目录 `pnpm-lock.yaml` 与 `server/Cargo.lock` 存在且非空 |
| Gitleaks 误报 | 测试密钥/样例被扫 | 根目录建 `.gitleaks.toml` 添加 allowlist（示例：`[[allowlist]] regex = '''...'''`） |
| CodeQL 报 language 不支持 | 语言矩阵问题 | JS/TS 与 Rust 分开矩阵跑；Rust 需 CodeQL 新版支持 |
| Slack 通知不触发 | secret 未配置 | 在 Actions secrets 添加 `SLACK_WEBHOOK_URL` |
| PR 上 secret-scan 跳过 | fork PR 无 token | 属预期安全行为，合并后 push main 会再扫 |

## 8. 当前仓库状态（诚实说明）

- 已入库资源：`libs/pikafish/*`（linux/macos/windows 引擎 + nnue）。
- 未入库资源：`libs/large.onnx`（YOLOv8 模型，Rust 编译期 include_bytes 内嵌）、`libs/windows-cpu/*.dll`、`libs/windows-gpu/*.dll`、`libs/linux/libonnxruntime*.so`
  → 因此 **Windows MSI / Linux deb 目前会被 gate 跳过**，Release 首版携带 web 产物；
  补齐资源后无需改流水线即自动出安装包。
- 本地已可完整验证：format/lint/typecheck/unit/build/E2E（见 `pnpm run` 脚本）。

## 9. 本地一键验证

```bash
pnpm install
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test:coverage
pnpm build
pnpm e2e          # 需要 pnpm exec playwright install chromium
```