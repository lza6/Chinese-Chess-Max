# 需求追踪矩阵（Requirement Traceability Matrix）

> 生成：2026-09-23 终局闭环审计 · 更新：2026-09-26（v0.2.9）· 方法：Spec Kit + 多 Agent 深度审计
> 状态：✅ 已闭环 / 🟡 部分闭环 / ❌ 未闭环 / ⚠️ 有风险 / 🚫 阻塞

## 1. 显式需求
| # | 需求 | 对应模块/文件 | 状态 | 证据 | 缺口与后续动作 |
|---|---|---|---|---|---|
| R1 | 实时识别屏幕棋盘（YOLOv8/ONNX） | server/src/yolo.rs, listen.rs | ⚠️ | predict/识别链路已实现；CI 资源 gate 检测 large.onnx 缺失 | large.onnx 缺失阻塞编译验证；需用户提供 |
| R2 | Pikafish 引擎分析并给中文招法 | server/src/engine/*, worker.rs | ✅ | search/bestmove/board_move_chinese 已实现；UCI 握手/EOF 防死循环/pvs 空兜底/锁快照（0.2.3+0.2.4） | — |
| R3 | 前端展示分析结果（最佳招法/评分/深度/日志） | src/components/Analyse.vue | ✅ | listen analyse 已接；滚动 ref/空兜底/日志裁剪/高亮清理（0.2.3） | — |
| R4 | 引擎参数配置（深度/时间/线程/哈希/云库） | src/components/Toolbar.vue + config.rs | ✅ | set_engine_* 命令 + 统一保存按钮 + 成功/失败反馈 + 失败还原（0.2.5） | — |
| R5 | 窗口选择与监听启停 | Toolbar + worker start/stop_listen | ✅ | list_windows/start/stop + 对话框；pending 防重复；stop detach 不 join；窗口关闭/截屏失败 Result 化（0.2.3+0.2.5） | — |
| R6 | 复制局面 | Toolbar copy_fen + get_current_fen | ✅ | get_current_fen 命令读取当前 FEN（0.2.5） | — |
| R7 | 图片识别 | server/src/yolo.rs + Toolbar | 🟡 | YOLOv11 输出契约已适配；按钮如实标注「开发中，需模型」 | 依赖 D1 真实模型；前端接线待模型到位 |
| R8 | 连线对战/人机对弈 | Toolbar mode 下拉 | 🚫 | disabled | 产品决策项，未越权实现 |
| R9 | 中文招法展示 | engine/mod.rs + Analyse | ✅ | moves 中文已接 | — |
| R10 | 开局库（云库 chessdb） | engine/chessdb.rs | ✅ | query 已实现；println 改 tracing、https、Success 空 pvs 兜底（0.2.3） | — |
| R11 | 复盘/导出/人机走子 | worker.rs 历史命令 + Analyse/Toolbar | ✅ | 后端命令 + 前端复盘面板/导出按钮/清空历史全部接通（0.2.5-0.2.7） | human_move 完整对弈流程待产品确认（R8） |

## 2. 隐式需求
| # | 需求 | 对应模块 | 状态 | 证据 | 缺口 |
|---|---|---|---|---|---|
| I1 | 可运行/可构建 | package.json / workflows | ✅ | 本机全绿 + CI 三线全绿 @dab9adf | — |
| I2 | 可调用（Tauri 命令/事件契约） | check-contract + Contract.test | ✅ | 命令16/事件6 双向对齐（0.2.5） | — |
| I3 | 无伪实现/无假功能 | Toolbar | ✅ | copy_fen 空函数已删并接通 get_current_fen；识别按钮如实标注开发中；README 诚实矩阵（0.2.3+0.2.5） | — |
| I4 | 文档与实现一致 | docs/* + workflows + scripts | ✅ | README 诚实矩阵/SPEC 版本/ADR×4/CHANGELOG 全同步（0.2.5-0.2.9） | — |
| I5 | 错误可反馈可恢复 | 前端对话框/后端 Result | ✅ | listen/worker/engine 全链 Result 化（0.2.3+0.2.5） | — |
| I6 | 配置持久化安全 | config.rs | ✅ | 原子写 + 解析失败备份 .bak 重建（0.2.3） | — |
| I7 | 数据一致性与防竞态 | worker.rs / engine | ✅ | 锁快照/stop detach/pvs 兜底（0.2.3）；listen_state 状态机（0.2.5） | — |
| I8 | 日志与可排障 | logger.rs | ✅ | tracing 滚动+控制台；chessdb println 已改 tracing（0.2.3） | — |

## 3. 验收导向要求
| # | 要求 | 验证方式 | 状态 |
|---|---|---|---|
| A1 | 一次调用尽量跑通 | E2E + 契约测试 + 构建 | ✅ |
| A2 | UI/按钮/功能真实接通 | E2E 6 条真实路径 + 单测 24 | ✅ |
| A3 | 非外部付费受限资源尽量真实验证 | 本地全绿 + CI 全绿；large.onnx 阻塞项诚实标注 | ✅ |
| A4 | md/README 主动更新 | README/SPEC/矩阵/CHANGELOG/ADR/VERIFICATION-LOG 全同步 | ✅ |

## 4. 非功能性要求
| # | 要求 | 现状 | 缺口 |
|---|---|---|---|
| N1 | 兼容性（Windows/macOS/Linux） | CI backend×3 + cfg 分支 | onnxruntime DLL/模型未入库 |
| N2 | 稳定性 | 全链 Result 化 + 防御性修复（0.2.3-0.2.9），cargo 14/14 | — |
| N3 | 易用性 | 空状态引导 + 状态徽标 + 触控≥44px（0.2.5） | — |
| N4 | 可维护性 | 文档齐全 | CHANGELOG/INDEX/ADR×4 齐备（0.2.8） |
| N5 | 可部署性 | release.yml + dist.zip | 安装包资源 gate |
| N6 | 可排障性 | tracing 日志 + listen_state 错误事件 | — |
| N7 | 可验证性 | CI 三线 + 契约 + 覆盖率 | large.onnx 阻塞后端验证 |
| N8 | 一致性 | 版本全局 0.2.9；契约 16/6 对齐 | README/SPEC 已同步（0.2.9） |

## 5. 主动补位检查（A-I 层，2026-09-26 v0.2.9）
| 层 | 结论 | 说明 |
|---|---|---|
| A 用户与交互 | ✅ 闭环 | 空态/加载/错误/禁用/防重复/反馈/还原已覆盖；深链/刷新不适用（桌面单页） |
| B API/调用层 | ✅ 闭环 | Tauri 命令参数校验（parse_iccs）+ 超时/重试（engine/chessdb）+ 契约双向校验 |
| C 后端业务 | ✅ 闭环 | 主链路完整；边界/空值/回滚/竞态已修（v0.2.7）；日志齐全；无敏感信息泄漏 |
| D 数据层 | ✅ 闭环 | 内存数据模型一致；FEN 单一事实来源；无 DB/迁移（不适用持久化 DB） |
| E 前后端衔接 | ✅ 闭环 | 命令/事件/枚举/错误双向对齐（check-contract 16/6） |
| F 配置与运行 | 🟡 部分 | 平台差异已处理；large.onnx/onnxruntime 属外部资源（SOP §2.5 说明，非代码可解） |
| G 工程维护 | ✅ 闭环 | README/CHANGELOG/ADR/INDEX/SOP 齐备；无调试残留 |
| H 测试验证 | ✅ 闭环 | cargo 14/14 + 前端 24/24 + E2E 6/6 + CI 三线；不可实测部分（真实模型）诚实标注 |
| I 安全合规 | ✅ 闭环 | 本地工具无 CORS/CSRF 面；输入校验严格；依赖 CVSS>=7 门禁；无凭证入库 |

## 6. 结论
- 已闭环：核心链路实现、构建/测试/CI、契约（16 命令/6 事件）、文档体系（含 ADR×4）、Rust 可靠性（全链 Result + 防御性修复 v0.2.9）。
- 阻塞：libs/large.onnx 真实模型缺失（编译/识别 gate 跳过，SOP §2.5 已给获取路径）；onnxruntime DLL 未入库（打包）。
- 待产品决策：human_move 完整人机对弈接线（R8）。



