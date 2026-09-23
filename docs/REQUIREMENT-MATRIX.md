# 需求追踪矩阵（Requirement Traceability Matrix）

> 生成：2026-09-23 终局闭环审计 · 方法：Spec Kit + 多 Agent 深度审计
> 状态：✅ 已闭环 / 🟡 部分闭环 / ❌ 未闭环 / ⚠️ 有风险 / 🚫 阻塞

## 1. 显式需求
| # | 需求 | 对应模块/文件 | 状态 | 证据 | 缺口与后续动作 |
|---|---|---|---|---|---|
| R1 | 实时识别屏幕棋盘（YOLOv8/ONNX） | server/src/yolo.rs, listen.rs | ⚠️ | predict/识别链路已实现；CI 资源 gate 检测 large.onnx 缺失 | large.onnx 缺失阻塞编译验证；需用户提供 |
| R2 | Pikafish 引擎分析并给中文招法 | server/src/engine/*, worker.rs | 🟡 | search/bestmove/board_move_chinese 已实现 | analyse() pvs 空 panic 风险、锁阻塞风险（P1 待修） |
| R3 | 前端展示分析结果（最佳招法/评分/深度/日志） | src/components/Analyse.vue | 🟡 | listen analyse 已接 | 滚动失效（ref 不匹配）、pvs/moves 空未兜底 |
| R4 | 引擎参数配置（深度/时间/线程/哈希/云库） | src/components/Toolbar.vue + config.rs | ✅ | set_engine_* 命令 + 抽屉表单 + 单位契约 | 无保存成功反馈；写锁可能被搜索阻塞 |
| R5 | 窗口选择与监听启停 | Toolbar + worker start/stop_listen | 🟡 | list_windows/start/stop + 对话框 | stop join 无超时；搜索框响应性待验证；对话框内容函数静态渲染风险 |
| R6 | 复制局面 | Toolbar copy_fen | ❌ | 空函数 | 假功能：要么实现要么如实标注 disabled |
| R7 | 图片识别 | Toolbar 识别按钮 | ❌ | disabled | 未实现；README 若宣传需修正 |
| R8 | 连线对战/人机对弈 | Toolbar mode 下拉 | 🚫 | disabled | 产品决策项，未越权实现 |
| R9 | 中文招法展示 | engine/mod.rs + Analyse | ✅ | moves 中文已接 | — |
| R10 | 开局库（云库 chessdb） | engine/chessdb.rs | 🟡 | query 已实现 | 调试 println 残留；Success 空 pvs 未兜底 |

## 2. 隐式需求
| # | 需求 | 对应模块 | 状态 | 证据 | 缺口 |
|---|---|---|---|---|---|
| I1 | 可运行/可构建 | package.json / workflows | ✅ | 本机全绿 + CI 三线全绿 @dab9adf | — |
| I2 | 可调用（Tauri 命令/事件契约） | check-contract + Contract.test | ✅ | 命令10/事件4 双向对齐 | reload_engine 未调用保留 |
| I3 | 无伪实现/无假功能 | Toolbar | ❌ | copy_fen 空、识别 disabled、README 宣传不实 | 需修复/如实标注 |
| I4 | 文档与实现一致 | docs/* + workflows + scripts | 🟡 | 基本一致 | 资源门禁脚本 ps1 待核验；CHANGELOG/ADR 缺 |
| I5 | 错误可反馈可恢复 | 前端对话框/后端 Result | 🟡 | 部分 unwrap 无错误路径 | start_listen/stop/capture panic 面需修 |
| I6 | 配置持久化安全 | config.rs | ⚠️ | load/save 已实现 | 解析失败删用户配置；无原子写 |
| I7 | 数据一致性与防竞态 | worker.rs / engine | ⚠️ | 状态机/锁已实现 | config 锁跨搜索阻塞；stop join 无超时；analyse pvs 空 panic |
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
| N2 | 稳定性 | 部分 unwrap 会 panic | 需 Result 化（P1） |
| N3 | 易用性 | 新手可 3 分钟跑通核心链路 | 无引导/空状态说明；假按钮误导 |
| N4 | 可维护性 | 文档齐全 | CHANGELOG/ADR/文档索引缺 |
| N5 | 可部署性 | release.yml + dist.zip | 安装包资源 gate |
| N6 | 可排障性 | tracing 日志 | chessdb println；错误信息不统一 |
| N7 | 可验证性 | CI 三线 + 契约 + 覆盖率 | large.onnx 阻塞后端验证 |
| N8 | 一致性 | 版本全局 0.2.2；契约对齐 | README 功能宣传不实 |

## 5. 结论
- 已闭环：核心链路实现、构建/测试/CI、契约、文档体系。
- 待修（本轮）：P1×5（前端滚动/后端 panic/锁阻塞/stop join/panic 面）+ P2×6（假功能/搜索框/配置安全/println/文档补全）。
- 阻塞：libs/large.onnx 缺失（需用户提供）。
