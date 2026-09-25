# Changelog

本仓库所有显著变更按时间倒序记录。

## [0.2.10] - 2026-09-26（终局闭环审计：内部算法路径防御 + 需求矩阵/反向审判）

### 可靠性（Rust 防御性修复）

- **board_move_chinese P/p 分支**：`position().unwrap()` → `unwrap_or(1/0)`（棋子重叠计算极端场景不再 panic）
- **fen_to_board 全防御化**：`split_once` 容错、`to_digit` 容错、rank/file 越界防护（畸形 FEN 不 panic）

### 文档（终局闭环审计硬性产出）

- **REQUIREMENT-MATRIX 全面刷新至 v0.2.9**：R4/R7/R11/I4/I8/A2/A4/N2/N3/N4/N6/N8 状态修正 + 新增「A-I 九层主动补位检查」
- **SELF-REVIEW 第七轮反向审判**：攻击 v0.2.5-v0.2.9 的伪闭环（内部 panic 残留/文档漂移/矩阵过时），逐一反驳并修复

### 测试

- Rust 14→15（新增 fen_to_board 畸形输入不 panic 回归）

## [0.2.9] - 2026-09-26（引擎等待超时 + 云库重试 + 文档一致性）

### 可靠性（Rust）

- **Engine::wait_until 时间预算**：等待引擎关键输出（uciok/readyok）增加 10s 硬上限（原仅行数 10_000 限制），引擎持续输出但不回应时不再无限阻塞
- **chessdb 单次重试**：云库瞬态失败（网络/连接异常）200ms 后重试 1 次再降级引擎；QueryState 补 PartialEq

### 文档一致性

- README 功能矩阵修正：复制局面「开发中」→「✅ 已实现」（v0.2.5 已接通 get_current_fen）；新增「对局复盘/导出 ✅ 已实现」行
- SPEC/SPEC-V2 版本基线刷新至 v0.2.8+

### 验证

- cargo fmt/clippy/test 14/14 全绿（新增改动无回归）

## [0.2.8] - 2026-09-26（架构资产 ADR + 部署资源 SOP + 资产改名）

### 架构与文档

- **新增 4 个 ADR**（docs/adr/）：模型编译期内嵌与诚实 gate / 无后端静态架构 / FEN 单一事实来源 / Tauri 契约防漂移
- **SOP 补「资源获取与放置」章节**：large.onnx / onnxruntime DLL 获取方式与放置路径，缺资源时 Web dist.zip 仍可发布的说明
- **docs/INDEX.md 补 ADR 区**；`docs/starup.png` → `docs/startup.png`（C8 遗留拼写修正）

### 验证

- 全量门禁重跑（v0.2.7 修复回归确认）：cargo fmt/clippy/test 14/14；pnpm lint/typecheck/test 24/24/build/e2e 6/6；契约 16/15/6 全绿

## [0.2.7] - 2026-09-26（终局闭环总审计：P0 输入校验 + FEN 数据正确性 + 崩溃面根治）

### 修复（P0/P1，真实终局审计发现）

- **非法着法输入不再 panic**：新增 `parse_iccs` 严格校验（长度=4、file a-i、rank 0-9），`Changed::from_pv`/`Move::new`/`board_move` 改 `Result`；`human_move` 传坏 ICCS（如空串/`z9z9`/中文）返回可读错误而非崩溃
- **引擎/云库 pv 不校验导致 Mutex 锁中毒**：`analyse` 在持有引擎锁时遇非法 pv 会 panic → 已改为非法 pv 跳过分析；chessdb 过滤空 pv
- **历史 FEN 数据错乱（双重应用走子）**：`record_move` 现在传入走子前局面计算 FEN，棋子不再消失；引擎走子不再被记录两次（原 record_engine_move + handle_move 重复）
- **初始局面重复定义且方向不一致**：删除 worker.rs 私有 `start_board()`，统一 `chess::red_startpos()`（黑方在上、标准 FEN 方向），复盘/复制/人机不再得到倒置局面
- **config.rs 写盘失败 panic → RwLock 中毒**：save/load 全容错降级（I/O 错误记日志，不 panic，配置命令不永久失效）
- **yolo.rs 会话初始化 expect panic**：GPU/模型不可用时惰性缓存 Err，`predict` 返回错误由监听线程降级；NaN 置信度排序 NaN 安全（max_by/nms）

### 前端

- 复盘面板「清空历史」按钮（clear_history 接入，契约前端调用 15/16）
- 复制局面 clipboard 降级（http 非安全上下文 textarea+execCommand fallback）
- 复盘显示统一用 history.length（不再依赖事件时序的 reviewTotal）

### 测试

- Rust 新增 4 个 P0 回归（非法 ICCS 拒绝 / 非法走子不 panic / 初始局面标准方向 / 走子往返一致性）：cargo test 10→14
- 前端 23→24（clear_history）；E2E 6/6 全绿

## [0.2.6] - 2026-09-26（CI/CD 双轨统一 + CodeQL 资源 gate）

### 工程

- **release.yml 资源检查统一**：check-linux / check-win-cpu / check-win-gpu 三处内联 `compgen`/`Get-ChildItem` 改为调用 `scripts/ci/check-resources.{sh,ps1}`（C4 落地，消除双轨漂移）
- **security.yml CodeQL 资源 gate**：Rust 语言缺 `libs/large.onnx` 时跳过 CodeQL 并输出 notice（C3 落地，不假绿不失败）

## [0.2.5] - 2026-09-26（主控代理完整落地）

### 新增（Rust 后端）

- **新命令 `get_current_fen`**：读取当前局面 FEN（未监听时返回初始局面），支撑「复制局面/导出/复盘初始化」
- **新命令 `export_game`**：对局导出（fen/txt/json），原子写用户文档目录
- **新命令 `load_history` / `clear_history`**：对局历史读取/清空（内存共享）
- **新命令 `review_step`**：复盘跳步（index=0 初始局面），重发 `position`/`mirror`/`review_state` 事件
- **新命令 `human_move`**：人机走子（ICCS），回发 `move` 事件；引擎应对后续接入
- **`reload_engine` 接通**：重新加载引擎并套用当前配置（配置热生效闭环）
- **新事件 `listen_state`**：监听线程状态（idle/running/error），前端订阅展示
- **新事件 `review_state`**：复盘步进状态（index/total/fen）

### 可靠性（Rust）

- `listen.rs` 全链 `Result` 化：`Window::new`/`ListenWindow::new`/`capture` 不再 `unwrap()` panic，窗口关闭/截屏失败返回可读错误
- `engine/mod.rs`：`Engine::new`/`reload`/`setoption`/`write_command` 返回 `Result`；`bestmove` 写命令失败返回空串；`parse_line` 容错坏数字
- `worker.rs`：`start_listen` 中 `predict().unwrap()` → `map_err`，识别失败不启动线程且 emit `listen_state:error`；`analyse`/`update_ui`/`handle_move` emit 失败不 panic；`confirm_board` 截屏失败不 panic
- 配置热生效：引擎搜索前克隆配置快照（已存在），`reload_engine` 提供手动重载

### 前端（已落地）

- 复制局面（get_current_fen + 剪贴板）、配置保存反馈 + 失败还原、重载引擎按钮、窗口列表排序、键盘可选中、监听状态徽标、空状态引导
- 分析日志时间戳、最佳招法空态「等待分析…」、高亮组件内状态（不全局清）、复盘面板、导出按钮
- 棋盘坐标数据生成（替代 90 格硬编码）、position 全量重绘
- App 布局 Flex 响应式 + 无障碍触控目标 ≥44px

## [0.2.4] - 2026-09-26（YOLOv11 模型契约修复）

### 修复（识别链路）

- **YOLOv11 真实模型输出契约适配**（8acfac6）：`output0` + `stride19` 无 objectness，识别链路按真实输出解析
- **版本 bump 0.2.3 → 0.2.4**（20eb522）

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
