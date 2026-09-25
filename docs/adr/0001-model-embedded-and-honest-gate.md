# ADR-0001：识别模型编译期内嵌 + 诚实资源 Gate

## 状态
✅ 已接受（2026-09-23 起，持续生效）

## 背景
YOLOv8 棋盘识别模型 `libs/large.onnx` 是桌面识别链路的核心依赖。最初设计为 Rust 编译期
`include_bytes!("../../libs/large.onnx")` 硬内嵌（server/src/yolo.rs），优点：单二进制自包含、
无运行时文件缺失风险、发布包即开即用。

但仓库长期缺失真实模型文件（当前仅 10.5MB 疑似占位），导致：
- Rust 编译被 include_bytes 硬阻断（cargo check/clippy/test 无法本地验证）
- 早期做法是用占位文件解锁编译，但给出「识别已实现」的假象，运行时识别必然失败

## 决策
1. **保留编译期内嵌**作为最终形态（真实模型到位后自然解锁）。
2. **CI 采用诚实资源 Gate**：`libs/large.onnx` 缺失时，后端 cargo clippy/check 与 Rust CodeQL
   **明确跳过并输出 `::notice::`**（scripts/ci/check-resources.* + security.yml check-model），
   不做假绿、也不让 job 失败。补齐文件后自动启用。
3. 运行期 `session()` 惰性初始化并缓存 `Result`：模型/GPU 初始化失败不 panic，监听线程降级
   并向前端发 `listen_state:error`。

## 后果
- 优点：无模型时可交付 Web 构建并保持 CI 三线绿；真实失败路径可观测；补模型即解锁安装包。
- 缺点：桌面安装包在模型缺失期间不可产出（诚实标注为用户提供的资源阻塞）。

## 替代方案（已否决）
- 运行时按需下载模型：引入网络下载、版本管理、失败重试复杂度，且与「离线本地工具」定位冲突。
- 空占位模型冒充真实：不可接受（伪造成功）。
