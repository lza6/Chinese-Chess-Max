# SOP：从零跑通→验证→发布→回滚（Runbook）

面向“新人/部署者/交接”：按顺序执行即可把一个干净环境跑到可用可交付。

## 1. 首克隆/首次接入
```bash
git clone https://github.com/lza6/Chinese-Chess-Max.git
cd Chinese-Chess-Max
pnpm install
```
前提：Node ≥ 20、pnpm 10、Rust toolchain（GitHub Actions 用 22/stable）。

## 2. 常规验证（改动任何代码后必跑）
```bash
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build
pnpm check:contract
pnpm exec playwright install chromium   # 仅首次
pnpm e2e
cd server && cargo fmt --check
# 缺 libs/large.onnx 时，cargo clippy/check 会 gate 跳过（见 CI notice）
```

## 2.5 资源获取与放置（桌面完整版必需）

桌面完整版（识别 + 安装包）依赖以下资源，Web 静态构建（dist.zip）不依赖它们：

| 资源 | 放置路径 | 获取方式 | 缺省后果 |
|---|---|---|---|
| YOLOv8 识别模型 | `libs/large.onnx` | 官方/自训棋盘识别模型（约 40-50MB），用户自备；**请勿提交占位文件冒充** | Rust 编译/识别链路 gate 跳过；安装包不产出 |
| onnxruntime 运行库 | `libs/windows-cpu/*.dll`、`libs/windows-gpu/*.dll`、`libs/linux/libonnxruntime*.so` | 官方 onnxruntime release（<https://github.com/microsoft/onnxruntime/releases>），按 OS/架构/EP 选包解压到对应目录 | MSI/deb 打包步骤自动跳过（`::notice::` 提示） |
| Pikafish 引擎 | `libs/pikafish/pikafish-{windows,os,linux}` + `pikafish.nnue` | 已随仓库入库（Windows 官方 2026-09-06 版） | — |

放置后：
- `scripts/ci/check-resources.*` 本地跑通（exit 0）即资源齐备；
- 推送 main 触发 Release，CI 自动启用安装包构建；缺资源时仍发布 Web `dist.zip`（诚实 gate，非失败）。

## 3. 本地运行
```bash
pnpm dev                      # Web 开发（Tauri 固定 1420）
pnpm tauri dev                # 桌面调试（需 Rust + libs/large.onnx）
```

## 4. 发布（main）
1. 在 main 上合并全部主题提交。
2. push 触发 `Release` workflow（`environment: production` 需人工审批）。
3. 产物：`dist.zip`（始终）；Linux deb / Windows MSI（资源存在时自动产出）。
4. 审批后在 Releases 页面点 Publish。

## 5. 回滚
- 未发布草稿：直接删除/编辑 Release，零影响。
- 已发布版本：将上一版本设为 Latest，或 `git revert <bad>` 后重发。
- Release 误发：`gh release delete <tag>` 后再删 tag（仅仓库有写权限时）。

## 6. 排障热表（详见 docs/CI-CD.md §7）
- frozen-lockfile → pnpm install 后提交锁文件。
- e2e “Executable doesn't exist” → `pnpm exec playwright install chromium`。
- cargo 缺资源 → `libs/large.onnx`（编译）、onnxruntime DLL（打包）。
- 契约“invoke 未注册” → 改 backend handler 后同步 `libs.rs generate_handler`。

## 7. 交接清单（新人请自查）
- [ ] `pnpm build` 绿
- [ ] `pnpm e2e` 绿
- [ ] `pnpm check:contract` 绿
- [ ] 阅读 docs/{SPEC,CONTRACT,CI-CD,SOP}.md
