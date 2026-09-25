# 反向审判：对上一轮工作的最强反驳（Self-Review）

> 2026-09-23 · 终局闭环审计 · 依据：不轻信台账，读代码 + 线上证据 + 多 Agent 交叉审计

## 1. 是否误解/偷换需求？
- 上轮把"闭环"定义为"测试/CI/文档全绿"，但对**用户真实操作路径**（选择窗口→启动→识别→分析→看懂结果）验证不足。
- 上轮未逐项核对 README 宣传的功能是否真实可用（发现 copy_fen 空实现、识别/复制按钮 disabled、"毫秒级"无依据）。

## 2. 是否把浅实现当完整实现？
- `copy_fen()` 是空函数，却保留在 UI 并有 tooltip——形式上"有按钮"，实际"点了没结果"。
- Analyse.vue 滚动用 `logInstRef` 但模板 ref 是 `logInst`——滚动功能形同虚设（模板上看起来完整）。
- 窗口选择对话框搜索框用 return h(NInput...) 的 content 函数：Naive UI 渲染后是否响应式未验证（疑似静态快照）。

## 3. 是否只修表面没打通链路？
- start_listen/stop_listen/reload_engine/predict 等大量 unwrap：识别失败、窗口关闭、引擎挂起时后台线程直接 panic，核心链路无错误兜底。
- analyze_board 持 config.read() 锁跨 engine.search()（最长 120s）：配置抽屉在分析时完全卡死，用户侧无反馈。
- stop_listen 的 join().unwrap() 无超时：引擎卡死时停止按钮永远转圈。

## 4. 是否漏掉用户操作路径的按钮/状态/异常？
- 启动引擎前无 loading、无"识别中"提示；识别失败只说"启动监听失败"无原因明细。
- 无窗口/窗口关闭/分辨率变化/棋盘被遮挡等场景均无明确错误反馈。
- 无空状态指引（首次打开怎么用完全靠猜）。

## 5. 是否漏掉调用方最容易踩坑的点？
- 单位契约（秒 vs 毫秒）虽已加测试，但 `time` 字段在前端展示为秒、后端存 ms、README/CONTRACT 描述分散，仍有混淆风险。
- check-resources.ps1/.sh 文档声称被 CI 使用，实际 release.yml 内联自己的检查——两套逻辑并存，未来会漂移。
- CI 路径过滤漏了 scripts/ci/**：改契约脚本不会触发任何校验。

## 6. 是否改一点破坏了其他模块？
- 上轮把 .gitattributes 加 eol=lf 后，工作树出现 2 个"假 M"文件（CRLF 规范化伪影），需 renormalize 清理。
- CodeQL 升 v4 后，security.yml 的 Rust autobuild 无资源 gate：缺 large.onnx 时理论上 cargo build 会失败（线上两次绿的证据需复核，标记为风险）。

## 7. 是否默认环境天然存在导致新环境无法复现？
- libs/large.onnx（40-50MB）缺失：所有 Rust 编译/安装包/GPU 链路都无法本地验收，上轮诚实标注了阻塞，但未提供"拿到模型后解锁的明确步骤"给用户。
- onnxruntime DLL 资源（windows-cpu/gpu/linux）同样缺失且无获取指引。

## 8. 是否没更新文档/README/模板？
- README 宣传"毫秒级识别""GPU 加速""开局库"但安装包未发布、GPU 未实测——宣传与事实不符。
- 无 CHANGELOG.md；无 docs 索引；VERIFICATION-LOG 写"4 处 version"实际 5 处。

## 9. 是否无证据过早宣称完成？
- 上轮宣称"CI/Security/Release 三线全绿"——已用 gh 实时 API 复核属实（@dab9adf 三线 success），这条站得住。
- "无假功能"当时未核——本轮发现 copy_fen 假按钮、disabled 识别按钮。

## 10. 是否漏掉真实用户第一时间的痛点？
- 首次打开无引导；窗口选择无排序（按 id 而非标题）；分析日志无时间戳；最佳招法高亮随时间累积不清理（b-select 只增不减）。
- 引擎配置修改无"已保存"反馈；保存后引擎不热更新（reload 未接入前端）。

## 结论
上轮在**构建/测试/CI/文档一致性**上基本扎实（线上证据复核通过），但在**运行时可靠性（panic/锁/卡死）、假功能清除、新手体验、宣传诚实性**上有真实缺口。本轮按 P0>P1>P2>P3 修复。

---

# 第七轮反向审判（2026-09-26，攻击 v0.2.5-v0.2.9）

> 依据：终局闭环总审计的「最狠反对者」角色，先假设前几轮存在伪闭环，再逐项攻击并修复。

## 1. 是否把浅实现误当完整实现？
- **攻击**：v0.2.7 说「非法输入不 panic」，但 `board_move_chinese` 内部仍有 `position().unwrap()`（P/p 分支，棋子重叠计算）、`fen_to_board` 仍有 `split_once(' ').unwrap()`/`to_digit(10).unwrap()`/越界索引——只是**外部入口**被堵了，内部算法路径仍可能 panic。
- **反驳**：成立。本轮已修复：P/p 分支 `position().unwrap()` → `unwrap_or(1/0)`；`fen_to_board` 全防御化（split 容错、to_digit 容错、rank/file 越界防护）。这是「顺着上下游链路向前追查」的实证。

## 2. 是否文档与实现不一致（虚假闭环）？
- **攻击**：README 功能矩阵「复制局面」仍写「开发中/未接线」，但 v0.2.5 已接通 get_current_fen——文档与实现脱节多轮未被发现。
- **反驳**：成立。本轮已修正 README 矩阵（复制局面✅、补复盘/导出行）+ SPEC 版本刷新。

## 3. 是否需求矩阵「看起来完整」实则过时？
- **攻击**：REQUIREMENT-MATRIX.md 停在 v0.2.5，R4「无保存反馈」等已修复项未刷新，A-I 层主动补位缺失。
- **反驳**：成立。本轮全面刷新至 v0.2.9 + 新增 A-I 九层补位检查。

## 4. 哪些攻击站不住（诚实保留）？
- v0.2.7 声称「初始局面方向统一」——经 `test_red_startpos_matches_standard_fen_orientation` 验证属实（FEN 首行 rnbakabnr）。
- v0.2.7 声称「引擎锁不中毒」——非法 pv 跳过分析路径真实存在。
- v0.2.9 声称「wait_until 超时」——10s 时间预算已落地。

## 5. 遗留（诚实标注，非伪闭环）
- large.onnx / onnxruntime DLL：外部资源，SOP §2.5 已给获取路径，非代码可解。
- human_move 前端接线：产品决策项，契约 WARN 如实保留。
- 真实模型识别精度：需模型到位后真实验收（现为诚实 gate 跳过）。

## 结论
前几轮在**外部输入防御、数据正确性、契约、CI** 上基本扎实；本轮攻击暴露的「内部算法路径残留 panic + 文档漂移 + 矩阵过时」已全部修复。仍无「把 mock/占位当完成」的伪闭环——不可实测项全部诚实标注阻塞。
