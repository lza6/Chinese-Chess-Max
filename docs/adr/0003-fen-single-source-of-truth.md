# ADR-0003：FEN / ICCS 单一事实来源与输入校验

## 状态
✅ 已接受（v0.2.7 强化）

## 背景
棋盘局面在系统内多处流转：屏幕识别（yolo+board_fix）、引擎（UCI FEN）、云库（querypv）、
历史记录、复盘重放、复制局面、导出、人机走子。早期存在两类真实缺陷：
1. **初始局面重复定义且方向不一致**：chess.rs 私有 `RED_STARTPOS`（黑方在上=标准 FEN） vs
   worker.rs 私有 `start_board()`（红方在上）→ 复盘/复制/人机得到 180° 倒置局面。
2. **非法 ICCS 输入 panic**：`from_pv`/`Move::new` 对空串/短串/越界坐标直接 split_at/索引越界，
   人机走子入口可被一行坏输入打崩。

## 决策
1. **chess.rs 为棋盘/着法唯一权威**：`board_fen` / `fen_to_board` / `red_startpos` /
   `parse_iccs` / `board_move` / `board_move_chinese` 全部收口于此。
2. **初始局面单一来源**：`RED_STARTPOS` 提升为 `pub(crate)`，暴露 `red_startpos()`；
   删除 worker 私有 `start_board()` 重复实现。
3. **严格输入校验**：`parse_iccs` 校验（长度=4、file a-i、rank 0-9，行号 '0'→y9 黑方在上）；
   `from_pv` / `Move::new` / `board_move` 返回 `Result`，非法输入不 panic。
4. **外部入口安全化**：`human_move` 对非法 ICCS 返回可读错误；`analyse` 对引擎/云库 pv
   先校验再应用（非法跳过分析，避免引擎 Mutex 锁中毒）。

## 后果
- 优点：任何路径产生的 FEN/ICCS 语义一致；外部输入不可触发 panic；P0 回归测试固定语义。
- 代价：调用点需处理 `Result`（已全量适配，cargo/clippy/test 全绿）。

## 替代方案（已否决）
- 前端推导 FEN：与后端 `board_fix`（黑方翻转）语义易漂移，双份实现必生分歧。
