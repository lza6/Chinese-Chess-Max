# Changelog

本仓库所有显著变更按时间倒序记录。

## [0.2.3] - 2026-09-24（终局闭环审计修复）

### 修复（Rust 后端）

- **引擎 UCI 握手**：Engine::new 增加 `uci`/`isready` 握手与超时检测，避免未就绪即搜索；bestmove 引擎 EOF 不再无限空转（worker 线程不 panic）
- **analyse() pvs 空兜底**：云库返回 Success 但无 pv 时不再 unwrap panic，跳过本次分析并记 error 日志
- **锁阻塞修复**：analyze_board 先克隆引擎配置再释放读锁，引擎搜索期间 set_engine_* 写锁不再被阻塞数秒
- **start_listen 防重入**：改用 `guard.is_some()` 判断（原 try_lock 在锁空闲但线程运行时仍会双 loop + 句柄泄漏）
- **stop_listen 不再 join**：后台线程靠 should_stop 自退，停止按钮不会被引擎挂起永久卡住
- **listen.rs**：ListenWindow::new 返回 Result，目标窗口不存在给可操作错误
- **chessdb.rs**：https、println 改 tracing、body 读取/解析全部防 panic（strip_suffix/text/parts 兜底）
- **config.rs**：原子写（临时文件+rename+回退）、解析失败备份为 .bak 而非删用户配置
- **logger.rs**：`try_init()` 防止测试重复初始化全局订阅者崩溃；read_line lossy 容忍引擎非 UTF-8 横幅

### 修复（前端）

- **Analyse.vue**：模板 ref 与变量对齐（logInstRef）、空 pvs/moves 兜底、time 展示、日志上限 splice、新分析前清旧高亮、nextTick 后滚动、listen 在 onMounted 注册并在 onUnmounted 解绑（防 HMR 累积）
- **Chessboard.vue**：mirror/position/move 监听统一 onMounted 注册 + onUnmounted 解绑
- **Toolbar.vue**：启动/停止 pending 防重复点击 + loading；图片识别/复制局面按钮如实标注“开发中”（删除空函数 copy_fen）；错误反馈补齐

### 修复（工程/文档）

- **README 诚实化**：功能状态矩阵（已实现/开发中）、下载安装如实说明当前仅 Web 构建、文档索引补全
- **CI 路径过滤**：scripts/ci/** 与 security.yml 加入 frontend/backend 触发，改契约脚本不再漏检
- **.gitignore**：补 .env*/.env.example/zip/bak-* 防误提交
- **tests/setup.ts**：jsdom Element.scrollTo polyfill（NLog 滚动依赖）
- **新增文档**：docs/SPEC-V2.md（Spec Kit 增量规范）、REQUIREMENT-MATRIX.md（需求追踪矩阵）、AUDIT-V2.md（深度审计清单）、SELF-REVIEW.md（反向审判）

### 基础设施（外部资源，需用户提供）

- `libs/large.onnx`（YOLOv8 模型）缺失 → Rust 编译已用占位文件解锁本地验证，运行时识别仍需真实模型
- 仓库自带 pikafish-windows.exe（2025-01-10，1.4MB）无法加载 NNUE → 已下载官方 2026-09-06 引擎+配套 nnue 到工作区验证可用（未入库，避免 50MB+ diff；原文件备份为 .bak-orig）

## [0.2.3] - 2026-09-23

- 契约脚本修复（check-contract 脏变量/错误符号）、.gitattributes LF 行尾契约、CodeQL v3→v4、CI e2e 显式契约步骤、第二轮审计文档

## [0.2.1] - 2026-09-23

- set_engine_time 单位 Bug 修复、前后端契约防漂移、CI/Security 安全门禁修复（osv 官方 CLI + CVSS>=7 阻断）、依赖高危漏洞定向升级

## [0.2.0] - 2026-09-23

- 首次发布：Vue3 + Tauri2 中国象棋分析工具基线
