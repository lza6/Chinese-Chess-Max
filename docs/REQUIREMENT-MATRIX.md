# 需求追踪矩阵（Requirement Traceability Matrix）

> 生成：2026-09-23 终局闭环审计 · 更新：2026-09-26（v0.2.4/v0.2.5）· 方法：Spec Kit + 多 Agent 深度审计
> 状态：✅ 已闭环 / 🟡 部分闭环 / ❌ 未闭环 / ⚠️ 有风险 / 🚫 阻塞

## 1. 显式需求
| # | 需求 | 对应模块/文件 | 状态 | 证据 | 缺口与后续动作 |
|---|---|---|---|---|---|
| R1 | 实时识别屏幕棋盘（YOLOv8/ONNX） | server/src/yolo.rs, listen.rs | ⚠️ | predict/识别链路已实现；CI 资源 gate 检测 large.onnx 缺失 | large.onnx 缺失阻塞编译验证；需用户提供 |
| R2 | Pikafish 引擎分析并给中文招法 | server/src/engine/*, worker.rs | ✅ | search/bestmove/board_move_chinese 已实现；UCI 握手/EOF 防死循环/pvs 空兜底/锁快照（0.2.3+0.2.4） | — |
| R3 | 前端展示分析结果（最佳招法/评分/深度/日志） | src/components/Analyse.vue | ✅ | listen analyse 已接；滚动 ref/空兜底/日志裁剪/高亮清理（0.2.3） | — |
| R4 | 引擎参数配置（深度/时间/线程/哈希/云库） | src/components/Toolbar.vue + config.rs | ✅ | set_engine_* 命令 + 抽屉表单 + 单位契约 | 无保存成功反馈；写锁可能被搜索阻塞 |
| R5 | 窗口选择与监听启停 | Toolbar + worker start/stop_listen | ✅ | list_windows/start/stop + 对话框；pending 防重复；stop detach 不 join；窗口关闭/截屏失败 Result 化（0.2.3+0.2.5） | — |
| R6 | 复制局面 | Toolbar copy_fen + get_current_fen | ✅ | get_current_fen 命令读取当前 FEN（0.2.5） | — |
| R7 | 图片识别 | server/src/yolo.rs + Toolbar | 🟡 | 真实 YOLOv11 输出契约已适配（output0+stride19 无 objectness，8acfac6） | 前端按钮仍标注开发中；运行时依赖 D1 真实模型验证 |
| R8 | 连线对战/人机对弈 | Toolbar mode 下拉 | 🚫 | disabled | 产品决策项，未越权实现 |
| R9 | 中文招法展示 | engine/mod.rs + Analyse | ✅ | moves 中文已接 | — |
| R10 | 开局库（云库 chessdb） | engine/chessdb.rs | ✅ | query 已实现；println 改 tracing、https、Success 空 pvs 兜底（0.2.3） | — |
| R11 | 复盘/导出/人机走子 | worker.rs 历史命令 + Analyse/Toolbar | 🟡 | get_current_fen/export_game/load_history/clear_history/review_step/human_move 后端就绪（0.2.5） | 前端交互同步中；人机完整对弈流程待产品确认 |

## 2. 隐式需求
| # | 需求 | 对应模块 | 状态 | 证据 | 缺口 |
|---|---|---|---|---|---|
| I1 | 可运行/可构建 | package.json / workflows | ✅ | 本机全绿 + CI 三线全绿 @dab9adf | — |
| I2 | 可调用（Tauri 命令/事件契约） | check-contract + Contract.test | ✅ | 命令16/事件6 双向对齐（0.2.5） | — |
| I3 | 无伪实现/无假功能 | Toolbar | ✅ | copy_fen 空函数已删并接通 get_current_fen；识别按钮如实标注开发中；README 诚实矩阵（0.2.3+0.2.5） | — |
| I4 | 文档与实现一致 | docs/* + workflows + scripts | 🟡 | 基本一致 | 资源门禁脚本 ps1 待核验；CHANGELOG/ADR 缺 |
| I5 | 错误可反馈可恢复 | 前端对话框/后端 Result | ✅ | listen/worker/engine 全链 Result 化（0.2.3+0.2.5） | — |
| I6 | 配置持久化安全 | config.rs | ✅ | 原子写 + 解析失败备份 .bak 重建（0.2.3） | — |
| I7 | 数据一致性与防竞态 | worker.rs / engine | ✅ | 锁快照/stop detach/pvs 兜底（0.2.3）；listen_state 状态机（0.2.5） | — |
| I8 | 日志与可排障 | logger.rs | ✅ | tracing 滚动+控制台 | 建议 chessdb println 改 tracing |

## 3. 验收导向要求
| # | 要求 | 验证方式 | 状态 |
|---|---|---|---|
| A1 | 一次调用尽量跑通 | E2E + 契约测试 + 构建 | ✅ |
| A2 | UI/按钮/功能真实接通 | 手工+E2E；copy_fen 修复/标注 | 🟡 |
| A3 | 非外部付费受限资源尽量真实验证 | 本地全绿 + CI 全绿；large.onnx 阻塞项诚实标注 | ✅ |
| A4 | md/README 主动更新 | 本轮同步 SPEC-V2/矩阵/VERIFICATION-LOG | 🟡 |

## 4. 非功能性要求
| # | 要求 | 现状 | 缺口 |
|---|---|---|---|
| N1 | 兼容性（Windows/macOS/Linux） | CI backend×3 + cfg 分支 | onnxruntime DLL/模型未入库 |
| N2 | 稳定性 | 全链 Result 化（0.2.3+0.2.5），cargo clippy/test 10/10 | — |
| N3 | 易用性 | 新手可 3 分钟跑通核心链路 | 无引导/空状态说明；假按钮误导 |
| N4 | 可维护性 | 文档齐全 | CHANGELOG/docs/INDEX 已补；ADR 未建 |
| N5 | 可部署性 | release.yml + dist.zip | 安装包资源 gate |
| N6 | 可排障性 | tracing 日志 | chessdb println；错误信息不统一 |
| N7 | 可验证性 | CI 三线 + 契约 + 覆盖率 | large.onnx 阻塞后端验证 |
| N8 | 一致性 | 版本全局 0.2.4/0.2.5；契约 16/6 对齐 | README 诚实矩阵已修（0.2.3） |

## 5. 结论
- 已闭环：核心链路实现、构建/测试/CI、契约（16 命令/6 事件）、文档体系、Rust 可靠性（Result 化全链）。
- 进行中（v0.2.5）：前端复盘/导出/人机交互同步、E2E 新命令/事件覆盖、响应式/无障碍。
- 阻塞：libs/large.onnx 缺失（需用户提供）；onnxruntime DLL 未入库（打包）。

