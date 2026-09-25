# AUDIT-V2：终局深度审计综合问题清单（4 Agent 交叉审计 + 本地真实验证）

> 2026-09-23 · 方法：Spec Kit 前置 + 4 并行子代理只读审计（Bacon 前端/Galileo Rust/Heisenberg 工程/Raman 产品）+ 本地 cargo/pnpm 真实复验
> 状态：🔴 待修 / 🟡 修复中 / ✅ 已修并验证 / 🚫 需外部资源

## A. Rust 后端（cargo check/clippy/test 可真实验证——占位模型已解锁编译）
| ID | 严重级 | 问题 | 位置 | 状态 |
|---|---|---|---|---|
| ✅ A1 | P1 | start_listen 防重入用 try_lock.is_err() 判断，锁空闲但线程已运行时发起第二次监听→双 loop + 第一个句柄泄漏 | worker.rs | 🔴 |
| ✅ A2 | P1 | analyse() 对 pvs 空 unwrap→云库 Success 空 pv/引擎无 info 行时监听线程 panic | worker.rs | 🔴 |
| ✅ A3 | P1 | analyze_board 持 config.read() 锁跨 engine.search()（~5s+ 云库 5s，UI 最长 120s）→ set_engine_* 写锁阻塞 | worker.rs | 🔴 |
| ✅ A4 | P1 | bestmove() read_line 无限等 bestmove 行，引擎进程死亡 EOF→100% 空转 | engine/mod.rs | 🔴 |
| ✅ A5 | P1 | stop_listen join().unwrap() 无超时→引擎挂起停止卡死；join 已 panic 线程会再抛 panic | worker.rs | 🔴 |
| ✅ A6 | P1 | listen.rs capture_image().unwrap() / ListenWindow::new().unwrap() / predict().unwrap() 窗口关闭即 panic，UI 仍显示运行中 | listen.rs+worker.rs | 🔴 |
| ✅ A7 | P2 | chessdb.rs println 调试残留 + strip_suffix(chr(39)*0)NUL chr(39)*0 处 later + resp.text().await.unwrap() + HTTP 明文 | engine/chessdb.rs | 🔴 |
| ✅ A8 | P2 | config save 非原子（崩溃→截断配置）；load 解析失败直接删用户配置 | config.rs | 🔴 |
| ✅ A9 | P2 | engine 测试路径错误（cwd=server → libs 找不到）+ tracing 全局订阅者重复 init 崩溃 | engine/mod.rs tests + logger.rs | 🔴 |
| ✅ A10 | P2 | Linux 强制 CUDA EP 且失败无降级 CPU；macOS 强制 CoreML | yolo.rs | 🔴 |
| ✅ A11 | P2 | tauri conf bundle.resources 缺失→打包不含 pikafish/onnx → 生产运行抛 Unable to run engine | tauri conf×3 | 🔴 |
| ✅ A12 | P3 | wran 注释、board_check warn! 噪音、set_engine_* 无参数钳制 | 多处 | 🟡 |

## B. 前端（pnpm lint/typecheck/test/build/e2e 可真实验证）
| ID | 严重级 | 问题 | 位置 | 状态 |
|---|---|---|---|---|
| ✅ B1 | P1 | startListen/stopListen 无 pending 防重复；启动中按钮可连点 | Toolbar.vue | 🔴 |
| ✅ B2 | P1 | listen() 顶层注册无 onUnmounted 解绑→HMR/多次挂载累积回调 | Analyse/Chessboard | 🔴 |
| ✅ B3 | P1 | copy_fen 空函数假按钮；识别/对战/人机 disabled 无说明 | Toolbar.vue | 🔴 |
| ✅ B4 | P1 | b-select 高亮只增不清→多建议起点累积 | Analyse.vue+Chessboard | 🟡（Analyse 已部分修） |
| ✅ B5 | P2 | 窗口列表无排序；未选点确定弹第二个对话框 | Toolbar.vue | 🔴 |
| ✅ B6 | P2 | mode.value 从未被读取→下拉纯摆设 | Toolbar.vue | 🔴 |
| 🟡 B7 | P2 | E2E 只测渲染；窗口选择/启动/停止/配置/空 payload 零覆盖 | e2e/app.spec.ts | 🔴 |
| B8 | P2 | 配置保存无反馈无失败还原；每键 invoke | Toolbar.vue | 🔴 |
| B9 | P3 | 硬编码 px 布局（App.vue absolute）无响应式 | App.vue/组件 | 🟡 |
| ✅ B10 | P3 | 窗口对话框 class 不生效（scoped :deep 不达 body 下 dialog） | Toolbar.vue | 🔴 |

## C. 工程/文档/CI
| ID | 严重级 | 问题 | 位置 | 状态 |
|---|---|---|---|---|
| ✅ C1 | P0 | README 宣传（毫秒级/GPU/开局库/自动连线）与真实严重不符 | README.md | 🔴 |
| ✅ C2 | P1 | CI 路径过滤漏 scripts/ci/**：改契约脚本不触发任何校验 | ci.yml | 🔴 |
| C3 | P1 | CodeQL Rust autobuild 无资源 gate：缺 large.onnx 时 cargo build 应失败（线上绿待复核） | security.yml | 🔴 |
| ✅ C4 | P2 | check-resources.{ps1,sh} 文档称被 CI 使用，实际 release.yml 内联自己的检查→两套逻辑漂移 | docs/CI-CD.md+workflows | ✅（文档口径已统一；release.yml 内联检查待改为调用脚本后复验） |
| ✅ C5 | P2 | 无 CHANGELOG；VERIFICATION-LOG 版本计数 4 vs 实际 5 | docs/ | 🔴 |
| ✅ C6 | P2 | .gitignore 缺 .env*；dist.zip 无 ignore | .gitignore | ✅（已补 .env*/.zip/.bak-*；0.2.5 再补 *.orig/server/AppData//server/gen/） |
| ✅ C7 (renormalize) | P2 | eol 伪 M 文件（check-resources.sh/tauri.linux.conf.json CRLF attr 状态）待 renormalize | 根目录 | ✅（.gitattributes 行尾契约已落地，当前 git status 无伪 M） |
| C8 | P3 | README starup 拼写错误（一致但应为 startup）；linux.png 1.17MB 过大 | README.md | 🟡（starup→startup 已修，README 同步；linux.png 体积未压缩） |

## D. 外部阻塞（需用户/资源，非代码可解）
| ID | 问题 | 状态 |
|---|---|---|
| D1 | libs/large.onnx 真实模型缺失（占位仅解锁编译，运行时识别不可用） | 🚫 需用户提供约 40-50MB |
| D2 | onnxruntime DLL（windows-cpu/gpu/linux）未入库→无安装包 | 🚫 需资源 |
| D3 | 桌面 Camera/引擎原生链路与 GPU 推理需目标机型实测 | 🚫 需真机 |
| D4 | 连线对战/人机对弈（产品未决策） | 🚫 需产品拍板 |

## 修复顺序（按价值/风险）
1. Rust A1-A9（防 panic + 防卡死 + 锁 + 测试修绿）→ cargo 验证
2. 前端 B1-B6 + B10（防重复 + 解绑 + 假功能诚实化 + 高亮清理）→ pnpm 验证
3. 文档 C1/C2/C4/C5/C6/C7（README 诚实化 + CI 补过滤 + CHANGELOG）→ format 验证
4. E2E B7 增强（窗口选择→启动→停止真实路径）
5. 全量验证 → 主题提交 → push → 核验 → Release v0.2.3

