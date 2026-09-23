# 可扩展性/性能适用性审查（SCALABILITY-REVIEW）

> 结论先行：本项目为 **桌面优先**（Tauri）+ **静态 Web 构建**，不存在自建云端后端服务。
> 通用 SaaS 高可用组件并非全部适用，此处按“适用性矩阵”给出诚实建议，避免照搬云架构引入复杂度。

## 1. 适用性矩阵
| 关注点 | 是否适用 | 说明/建议 |
|---|---|---|
| CDN 静态加速 | ✅ 适用（Web 构建） | 静态 SPA 直接放 CDN/对象存储；JavaScript 已 gzip(≈144KB)。 |
| 构建缓存 | ✅ 适用 | CI 已启用 pnpm cache + rust-cache。 |
| 引擎并发/吞吐 | ✅ 适用（桌面色） | 引擎分析线程数/hash 可配置（libs/config.json: engine_threads/hash）；避免超线程导致反馈卡顿（建议 ≤4/64MB）。 |
| Observability | 🟡 部分 | `tracing`/tracing-subscriber 已有日志；建议补充：结构化 event 日志 + `listener_thread` 状态可视化。 |
| Health Checks | 🟡 部分 | start_listen 重复启动会报“已经在监听中”；建议前台展示监听状态而非仅 toast。 |
| Rate Limiting | ➖ 不适用 | 无对外 API；本机托盘分析不涉及流量限流。 |
| Redis/Kafka | ➖ 不适用 | 本地单进程、无排队业务，引入即过度工程。 |
| 主从/分片 | ➖ 不适用 | 本地单人工具。 |
| 容量/错误处理 | ✅ 适用 | start_listen/analyse 均有 result/Err；前端应展示错误而非静默。 |
| 资源释放 | ✅ 适用 | stop_listen 会 join 线程；监听崩溃路径需确保线程不悬空（已 review worker）。 |

## 2. 可落地改进（按 ROI 排序）
1. 把 `libs/large.onnx` 入库 → 解锁 Rust 编译/发布（最高优先级，阻断）。
2. 前端错误/空状态收敛（识别失败 / 无棋盘 / 引擎未就绪）统一 UI 提示。
3. 监听线程状态可观测（当前仅内存 flag）。
4. Web 发布目标 + Cloudflare/CDN 说明（示例命令）。
5. `config.loglevel` 增加“详细”透传地。

## 3. 性能基线（轻量）
- 前端构建 gzip：JS ≈143.9KB、CSS ≈0.94KB（vite 报出）。
- 单元测试 10 用例 ~50s、E2E ~11s（本机）。

## 4. 不承诺（诚实声明）
- 未做云端压测 / 慢查询优化：项目无 DB / 无对外服务。
- 桌面 GPU/CPU 推理性能需在目标机型实测（onnxruntime 资源未入库无法本地跑通）。