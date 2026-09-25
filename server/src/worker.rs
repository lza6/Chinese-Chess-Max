use std::thread;
use std::time::Duration;

use tauri::AppHandle;
use tauri::Emitter as _;
use tauri::Manager as _;
use tauri::async_runtime::block_on;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use xcap::image::ImageBuffer;
use xcap::image::Rgba;

use crate::SHARED_STATE;
use crate::chess;
use crate::common;
use crate::engine::QueryResult;
use crate::listen::ListenWindow;
use crate::listen::Window;
use crate::yolo::IMAGE_HEIGHT;
use crate::yolo::IMAGE_WIDTH;
use crate::yolo::predict;

/// 监听线程状态（前端订阅 listen_state 事件）
#[derive(Debug, serde::Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ListenState {
    Idle,
    Running,
    Error(String),
}

/// 对局历史单步
#[derive(Debug, serde::Serialize, Clone)]
pub struct HistoryEntry {
    pub seq: usize,
    pub from: String,
    pub to: String,
    pub piece: char,
    pub camp: String,
    pub iccs: String,
    pub fen: String,
    pub source: String, // "human" | "engine"
}

/// 对局历史（内存共享，导出时原子写文件）
pub struct GameHistory {
    pub entries: Vec<HistoryEntry>,
    pub current_fen: String,
    pub current_camp: String,
    pub last_board: [[char; 9]; 10],
}

impl Default for GameHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_fen: String::new(),
            current_camp: "w".to_string(),
            last_board: [[' '; 9]; 10],
        }
    }
}

impl GameHistory {
    pub fn reset(&mut self) {
        self.entries.clear();
        self.current_fen.clear();
        self.current_camp = "w".to_string();
        self.last_board = [[' '; 9]; 10];
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        from: String,
        to: String,
        piece: char,
        camp: char,
        iccs: String,
        fen: String,
        source: String,
    ) {
        let fen_empty = fen.is_empty();
        let seq = self.entries.len() + 1;
        self.entries.push(HistoryEntry {
            seq,
            from,
            to,
            piece,
            camp: camp.to_string(),
            iccs,
            fen: fen.clone(),
            source,
        });
        if !fen_empty {
            self.current_fen = fen.clone();
            self.current_camp = camp.to_string();
        }
        if self.entries.len() > 2048 {
            self.entries.remove(0);
        }
    }
}

// 棋盘分析结果
struct BoardAnalysisResult {
    expect_move: chess::Changed,
    expect_board: [[char; 9]; 10],
}

// 定义不同的棋盘状态
#[derive(PartialEq)]
enum ChessboardState {
    Initial,      // 初始状态，没有进行任何分析
    StartPos,     // 初始棋盘状态
    OurTurn,      // 我方行棋
    OpponentTurn, // 对方行棋
    Invalid,      // 无效状态
}

// 分析上下文，保存分析状态和共享数据
struct AnalysisContext {
    app: AppHandle,
    window: ListenWindow,
    last_board: [[char; 9]; 10],
    expect_move: chess::Changed,
    expect_board: [[char; 9]; 10],
    invalid_change_count: usize,
}

unsafe impl Send for AnalysisContext {}
unsafe impl Sync for AnalysisContext {}

impl AnalysisContext {
    fn new(app: AppHandle, window: ListenWindow) -> Self {
        Self {
            app,
            window,
            last_board: [[' '; 9]; 10],
            expect_move: chess::Changed::default(),
            expect_board: [[' '; 9]; 10],
            invalid_change_count: 0,
        }
    }

    // 检查是否需要终止分析线程
    fn should_stop(&self) -> bool {
        let state = SHARED_STATE.get().unwrap();
        state.listen_thread.lock().unwrap().is_none()
    }

    // 获取棋盘图像并分析
    fn capture_and_analyze_board(&self) -> Option<(chess::Camp, [[char; 9]; 10])> {
        let image = self.window.capture().ok()?;
        get_board(image)
    }

    // 确认棋盘状态是否稳定
    fn confirm_board(&self, board: [[char; 9]; 10]) -> bool {
        thread::sleep(Duration::from_millis(100));
        if let Ok(conf_image) = self.window.capture()
            && let Some((_, conf_board)) = get_board(conf_image)
        {
            return conf_board == board;
        }
        false
    }

    // 分析棋盘并返回结果
    fn analyze_board(
        &mut self,
        camp: &chess::Camp,
        board: [[char; 9]; 10],
    ) -> Option<BoardAnalysisResult> {
        let fen = chess::board_fen(camp, board);
        // 先克隆配置再释放读锁：避免引擎搜索期间阻塞 set_engine_* 写锁
        let config = {
            let guard = SHARED_STATE.get().unwrap().config.read().unwrap();
            guard.engine
        };
        let state = SHARED_STATE.get().unwrap();
        let mut engine = state.engine.lock().unwrap();
        let result = block_on(engine.search(&fen, &config));
        result.as_ref()?;

        let (expect_move, expect_board) = analyse(&self.app, result.unwrap(), board);
        Some(BoardAnalysisResult {
            expect_move,
            expect_board,
        })
    }

    // 更新UI显示
    fn update_ui(&self, camp: &chess::Camp, board: [[char; 9]; 10]) {
        let board_map = chess::board_map(board);
        let _ = self.app.emit("mirror", camp.is_black());
        let _ = self.app.emit("position", &board_map);
    }

    // 处理移动事件（记录历史；emit 失败不 panic）
    fn handle_move(&mut self, changed: &chess::Changed) {
        let fen = chess::board_fen(
            &changed.camp,
            Self::board_after_move(self.last_board, changed),
        );
        {
            let state = SHARED_STATE.get().unwrap();
            let mut history = state.history.lock().unwrap();
            history.push(
                changed.from.clone(),
                changed.to.clone(),
                changed.piece,
                changed.camp.to_char(),
                format!("{}{}", changed.from, changed.to),
                fen,
                "human".to_string(),
            );
        }
        let _ = self.app.emit("move", changed);
    }

    fn board_after_move(board: [[char; 9]; 10], changed: &chess::Changed) -> [[char; 9]; 10] {
        let mut nb = board;
        let from_x = changed.from.chars().next().unwrap_or('a') as usize - 97;
        let from_y = 57 - changed.from.chars().nth(1).unwrap_or('0') as usize;
        let to_x = changed.to.chars().next().unwrap_or('a') as usize - 97;
        let to_y = 57 - changed.to.chars().nth(1).unwrap_or('0') as usize;
        if from_x < 9 && from_y < 10 && to_x < 9 && to_y < 10 {
            nb[to_y][to_x] = nb[from_y][from_x];
            nb[from_y][from_x] = ' ';
        }
        nb
    }

    // 记录引擎走子到历史（source=engine）
    fn record_engine_move(&mut self, changed: &chess::Changed) {
        let fen = chess::board_fen(
            &changed.camp,
            Self::board_after_move(self.last_board, changed),
        );
        {
            let state = SHARED_STATE.get().unwrap();
            let mut history = state.history.lock().unwrap();
            history.push(
                changed.from.clone(),
                changed.to.clone(),
                changed.piece,
                changed.camp.to_char(),
                format!("{}{}", changed.from, changed.to),
                fen,
                "engine".to_string(),
            );
        }
    }

    // 处理错误变化计数
    fn handle_invalid_change(
        &mut self,
        last_board: [[char; 9]; 10],
        board: [[char; 9]; 10],
        camp: &chess::Camp,
    ) -> ChessboardState {
        if self.invalid_change_count < 3 {
            self.invalid_change_count += 1;
            let last_fen = chess::board_fen(camp, last_board);
            let current = chess::board_fen(camp, board);
            debug!("OneChanged last {}", last_fen);
            debug!("OneChanged current {}", current);
            ChessboardState::Invalid
        } else {
            // 如果出现次数超过3次，重置为初始状态
            debug!("OneChanged count=3, reload");
            self.invalid_change_count = 0;
            SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
            ChessboardState::Initial
        }
    }
}

pub fn get_board(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Option<(chess::Camp, [[char; 9]; 10])> {
    let data = predict(image).ok()?;
    if let Ok((camp, mut board)) = common::detections_to_board(&data) {
        chess::board_fix(&camp, &mut board);
        Some((camp, board))
    } else {
        None
    }
}

pub fn analyse(
    app: &AppHandle,
    mut result: QueryResult,
    board: [[char; 9]; 10],
) -> (chess::Changed, [[char; 9]; 10]) {
    // 引擎结果翻译为中文（防御云库返回 Success 但 pv 为空）
    let Some(best_pv) = result.pvs.first() else {
        error!("analyse: pvs 为空，跳过本次分析");
        return (chess::Changed::default(), board);
    };
    let best_move = chess::board_move_chinese(board, best_pv);
    let expect_board = chess::board_move(board, best_pv);
    let expect_move = chess::Changed::from_pv(best_pv, board);

    let mut tmp_board = expect_board;
    result.moves.push(best_move);
    for pv in result.pvs.iter().skip(1).take(3) {
        let mv = chess::board_move_chinese(tmp_board, pv);
        result.moves.push(mv);
        tmp_board = chess::board_move(tmp_board, pv);
    }
    // 把结果发送给前端
    info!("分析结果 {:?}", result);
    let _ = app.emit("analyse", result);

    // 返回一个预期move和预期board
    (expect_move, expect_board)
}

// 处理循环逻辑的主函数
fn process_analysis_loop(mut context: AnalysisContext) {
    let mut current_state = ChessboardState::Initial;

    loop {
        // 检查是否需要终止监听
        if context.should_stop() {
            debug!("listen stopped");
            SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
            emit_listen_state(&context.app, ListenState::Idle);
            break;
        }

        // 获取等待间隔
        let interval = SHARED_STATE
            .get()
            .unwrap()
            .config
            .read()
            .unwrap()
            .timer_interval;
        thread::sleep(Duration::from_millis(interval));

        // 捕获并分析棋盘
        if let Some(error) = context.window.capture().err() {
            debug!("窗口截屏失败: {error}，等待下一轮");
            emit_listen_state(&context.app, ListenState::Error(error));
            thread::sleep(Duration::from_millis(300));
            continue;
        }
        let board_result = context.capture_and_analyze_board();
        if board_result.is_none() {
            continue;
        }

        let (camp, board) = board_result.unwrap();
        trace!("{:?} {:?}", camp, board);

        // 根据不同状态处理棋盘
        current_state = match current_state {
            ChessboardState::Initial => {
                // 初始状态，做第一次分析
                debug!("首次启动，立即分析");

                // 设置前端棋盘
                context.update_ui(&camp, board);

                // 分析当前棋盘
                if let Some(result) = context.analyze_board(&camp, board) {
                    context.expect_move = result.expect_move;
                    context.expect_board = result.expect_board;
                }

                context.last_board = board;

                // 如果是初始棋盘，进入初始状态，否则进入一般状态
                if chess::startpos(board) {
                    ChessboardState::StartPos
                } else if camp.eq(&chess::Camp::Red) {
                    ChessboardState::OurTurn
                } else {
                    ChessboardState::OpponentTurn
                }
            }

            ChessboardState::StartPos => {
                // 判断棋盘是否仍然是初始棋盘
                if !chess::startpos(board) {
                    // 不再是初始棋盘，处理正常的棋局变化
                    if board == context.last_board {
                        ChessboardState::StartPos // 没有变化
                    } else {
                        // 有变化，更新UI并分析
                        let (changed, board_state) = chess::board_diff(context.last_board, board);

                        match board_state {
                            chess::BoardChangeState::Move => {
                                context.last_board = board;
                                context.handle_move(&changed);

                                if camp.eq(&changed.camp) {
                                    // 我方移动
                                    ChessboardState::OurTurn
                                } else {
                                    // 对方移动，需要分析
                                    if let Some(result) = context.analyze_board(&camp, board) {
                                        context.expect_move = result.expect_move;
                                        context.expect_board = result.expect_board;
                                    }
                                    ChessboardState::OpponentTurn
                                }
                            }
                            chess::BoardChangeState::One => {
                                context.handle_invalid_change(context.last_board, board, &camp)
                            }
                            chess::BoardChangeState::Unknown => {
                                debug!("棋局变化未知，重置上下文");
                                context.update_ui(&camp, board);
                                context.last_board = board;
                                SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
                                ChessboardState::Initial
                            }
                        }
                    }
                } else if chess::Camp::Red.eq(&camp) {
                    // 仍然是初始棋盘，且我方先手
                    if context.last_board == board {
                        // 防止重复分析
                        ChessboardState::StartPos
                    } else {
                        // 设置前端棋盘
                        context.last_board = board;
                        context.update_ui(&camp, board);

                        // 调用引擎查询
                        if let Some(result) = context.analyze_board(&camp, board) {
                            context.expect_move = result.expect_move;
                            context.expect_board = result.expect_board;
                        }

                        ChessboardState::OurTurn
                    }
                } else {
                    // 对方先手，跳过分析
                    debug!("对方先手，跳过分析");
                    context.last_board = board;
                    context.update_ui(&camp, board);
                    ChessboardState::OpponentTurn
                }
            }

            ChessboardState::OurTurn | ChessboardState::OpponentTurn => {
                // 判断棋盘是否未发生变化
                if board == context.last_board {
                    debug!("棋盘未发生变化，跳过分析");
                    current_state // 保持当前状态
                } else if board == context.expect_board {
                    // 符合预期棋盘，跳过分析
                    debug!("棋盘为预期棋盘，跳过分析");
                    let expect_move = context.expect_move.clone();
                    let expect_board = context.expect_board;
                    context.last_board = expect_board;
                    context.record_engine_move(&expect_move);
                    context.handle_move(&expect_move);

                    // 更换下一个行动方
                    if current_state == ChessboardState::OurTurn {
                        ChessboardState::OpponentTurn
                    } else {
                        ChessboardState::OurTurn
                    }
                } else {
                    // 确认棋盘变化是否稳定
                    if !context.confirm_board(board) {
                        debug!("棋盘延迟确认失败");
                        let confirm_interval = SHARED_STATE
                            .get()
                            .unwrap()
                            .config
                            .read()
                            .unwrap()
                            .confirm_interval;
                        thread::sleep(Duration::from_millis(confirm_interval));
                        current_state // 保持当前状态
                    } else if !chess::board_check(board) {
                        // 检测棋盘是否有效
                        let debug_fen = chess::board_fen(&camp, board);
                        debug!("棋盘识别无效: {}", debug_fen);
                        current_state // 保持当前状态
                    } else {
                        // 处理正常棋盘变化
                        let (changed, board_state) = chess::board_diff(context.last_board, board);

                        match board_state {
                            chess::BoardChangeState::Move => {
                                context.last_board = board;
                                context.handle_move(&changed);

                                if camp.eq(&changed.camp) {
                                    // 我方移动，跳过分析
                                    debug!(
                                        "我方移动, {} -> {}, 跳过分析",
                                        changed.from, changed.to
                                    );
                                    ChessboardState::OurTurn
                                } else {
                                    // 对方移动，需要分析
                                    debug!(
                                        "对方移动, {} -> {}, 需要分析",
                                        changed.from, changed.to
                                    );
                                    if let Some(result) = context.analyze_board(&camp, board) {
                                        context.expect_move = result.expect_move;
                                        context.expect_board = result.expect_board;
                                    }
                                    ChessboardState::OpponentTurn
                                }
                            }
                            chess::BoardChangeState::One => {
                                context.handle_invalid_change(context.last_board, board, &camp)
                            }
                            chess::BoardChangeState::Unknown => {
                                debug!("棋局变化未知，重置上下文");
                                context.update_ui(&camp, board);
                                context.last_board = board;
                                SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
                                ChessboardState::Initial
                            }
                        }
                    }
                }
            }

            ChessboardState::Invalid => {
                // 复位到初始状态，等待下一次有效的变化
                ChessboardState::Initial
            }
        };
    }
}

fn emit_listen_state(app: &AppHandle, state: ListenState) {
    let _ = app.emit("listen_state", &state);
}

// 初始化Tauri的command处理
#[tauri::command]
pub async fn start_listen(app: AppHandle, target: Window) -> Result<(), String> {
    trace!("start_listen");
    {
        let guard = SHARED_STATE.get().unwrap().listen_thread.lock().unwrap();
        if guard.is_some() {
            error!("current listen thread is running, please stop it first");
            return Err("已经在监听中".to_string());
        }
    }

    // 初始化监听窗口模块
    let mut window = ListenWindow::new(&target, IMAGE_WIDTH, IMAGE_HEIGHT)
        .map_err(|e| format!("{e}，请重新选择窗口"))?;
    let image = window
        .capture()
        .map_err(|e| format!("目标窗口截屏失败: {e}"))?;

    let image_h = image.height();
    let image_w = image.width();

    let detections = predict(image).map_err(|e| format!("棋盘识别失败（模型/推理错误）: {e}"))?;

    match common::detections_bound(image_w, image_h, &detections) {
        Ok((x, y, w, h)) => {
            window.set_sub_bound(x, y, w, h); // 设置窗口边界
        }
        Err(e) => {
            emit_listen_state(&app, ListenState::Error(e.clone()));
            return Err(e); // 未识别到棋盘
        }
    }

    // 创建分析上下文
    let context = AnalysisContext::new(app.clone(), window);
    SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
    emit_listen_state(&app, ListenState::Running);

    // 启动后台线程进行截图和处理
    let listen_thread = thread::spawn(move || {
        trace!("into thread");
        process_analysis_loop(context);
    });

    SHARED_STATE
        .get()
        .unwrap()
        .listen_thread
        .lock()
        .unwrap()
        .replace(listen_thread);

    Ok(())
}

#[tauri::command]
pub fn stop_listen() {
    info!("stop listen");
    let shared_state = SHARED_STATE.get().unwrap();
    if let Ok(mut state) = shared_state.listen_thread.lock()
        && let Some(listen_thread) = state.take()
    {
        // 释放锁，停止后台线程
        debug!("释放锁，停止后台线程");
        drop(state);
        // 不 join：后台线程在每次轮询开头检测 should_stop（listen_thread 已置 None）
        // 后会自行退出；引擎搜索最多一个搜索周期内结束，避免停止按钮被永久阻塞。
        let _ = listen_thread;
    }
    debug!("stoped");
}

/// 读取当前 FEN（供复制局面/导出/复盘初始化）
#[tauri::command]
pub fn get_current_fen() -> Result<(String, String), String> {
    let state = SHARED_STATE.get().unwrap();
    let history = state.history.lock().unwrap();
    if history.current_fen.is_empty() {
        // 未监听时返回空，由前端决定是否用初始局面
        return Ok((
            chess::board_fen(&chess::Camp::Red, start_board()),
            "w".to_string(),
        ));
    }
    Ok((history.current_fen.clone(), history.current_camp.clone()))
}

fn start_board() -> [[char; 9]; 10] {
    let mut b = [[' '; 9]; 10];
    let init = [
        (0, 0, 'R'),
        (1, 0, 'N'),
        (2, 0, 'B'),
        (3, 0, 'A'),
        (4, 0, 'K'),
        (5, 0, 'A'),
        (6, 0, 'B'),
        (7, 0, 'N'),
        (8, 0, 'R'),
        (1, 2, 'C'),
        (7, 2, 'C'),
        (0, 3, 'P'),
        (2, 3, 'P'),
        (4, 3, 'P'),
        (6, 3, 'P'),
        (8, 3, 'P'),
        (0, 9, 'r'),
        (1, 9, 'n'),
        (2, 9, 'b'),
        (3, 9, 'a'),
        (4, 9, 'k'),
        (5, 9, 'a'),
        (6, 9, 'b'),
        (7, 9, 'n'),
        (8, 9, 'r'),
        (1, 7, 'c'),
        (7, 7, 'c'),
        (0, 6, 'p'),
        (2, 6, 'p'),
        (4, 6, 'p'),
        (6, 6, 'p'),
        (8, 6, 'p'),
    ];
    for (x, y, p) in init {
        b[y][x] = p;
    }
    b
}

/// 导出对局（FEN/TXT/JSON），原子写用户文档目录
#[tauri::command]
pub async fn export_game(app: AppHandle, format: String) -> Result<String, String> {
    let state = SHARED_STATE.get().unwrap();
    let history = state.history.lock().unwrap();
    let entries = history.entries.clone();
    let current_fen = history.current_fen.clone();
    drop(history);

    let dir = app
        .path()
        .document_dir()
        .map_err(|e| format!("获取文档目录失败: {e}"))?;
    let ts = chrono::now_timestamp();
    let filename = format!(
        "chess-game-{ts}.{}",
        if format == "json" {
            "json"
        } else if format == "txt" {
            "txt"
        } else {
            "fen"
        }
    );
    let path = dir.join(&filename);

    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&GameExport {
            fen: current_fen,
            moves: entries,
        })
        .map_err(|e| format!("序列化 JSON 失败: {e}"))?,
        "txt" => {
            let mut s = String::new();
            if !current_fen.is_empty() {
                s.push_str(&format!("FEN: {}\n", current_fen));
            }
            for e in &entries {
                s.push_str(&format!(
                    "{}. {} -> {} ({})\n",
                    e.seq, e.from, e.to, e.source
                ));
            }
            s
        }
        _ => current_fen,
    };

    // 原子写
    let ext = filename
        .rfind('.')
        .map(|i| &filename[i + 1..])
        .unwrap_or("fen");
    let tmp = path.with_extension(format!("{ext}.tmp"));
    std::fs::write(&tmp, content.as_bytes()).map_err(|e| format!("写入临时文件失败: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("保存文件失败: {e}"))?;
    Ok(path.display().to_string())
}

#[derive(serde::Serialize)]
struct GameExport {
    fen: String,
    moves: Vec<HistoryEntry>,
}

/// 读取对局历史
#[tauri::command]
pub fn load_history() -> Vec<HistoryEntry> {
    SHARED_STATE
        .get()
        .unwrap()
        .history
        .lock()
        .unwrap()
        .entries
        .clone()
}

/// 清空对局历史
#[tauri::command]
pub fn clear_history() {
    SHARED_STATE.get().unwrap().history.lock().unwrap().reset();
}

/// 复盘回放：跳到第 index 步（0=初始局面），把对应 FEN 局面重发 position 事件
#[tauri::command]
pub fn review_step(app: AppHandle, index: usize) -> Result<(), String> {
    let state = SHARED_STATE.get().unwrap();
    let history = state.history.lock().unwrap();
    let entries = &history.entries;
    let total = entries.len();
    let (fen, camp) = if index == 0 {
        (
            chess::board_fen(&chess::Camp::Red, start_board()),
            chess::Camp::Red,
        )
    } else if let Some(e) = entries.get(index - 1) {
        (
            e.fen.clone(),
            chess::Camp::from_char(e.camp.chars().next().unwrap_or('w')),
        )
    } else {
        return Err(format!("复盘步数越界: index={index} > {total}"));
    };
    drop(history);

    let board = chess::fen_to_board(&fen);
    let board_map = chess::board_map(board);
    let _ = app.emit("position", &board_map);
    let _ = app.emit("mirror", camp.is_black());
    let _ = app.emit(
        "review_state",
        serde_json::json!({ "index": index, "total": total, "fen": fen }),
    );
    Ok(())
}

/// 人机对战：前端走子（iccs），引擎应对
#[tauri::command]
pub async fn human_move(app: AppHandle, iccs: String) -> Result<(), String> {
    let state = SHARED_STATE.get().unwrap();
    let history = state.history.lock().unwrap();
    let current_fen = if history.current_fen.is_empty() {
        chess::board_fen(&chess::Camp::Red, start_board())
    } else {
        history.current_fen.clone()
    };
    drop(history);

    let board = chess::fen_to_board(&current_fen);
    let changed = chess::Changed::from_pv(&iccs, board);
    let _ = app.emit("move", &changed);
    // 简单版：前端已用 human_move 更新棋盘；引擎应对由 review_step/分析线程负责。
    // 后续接入引擎搜索（Pending）。
    Ok(())
}

mod chrono {
    use std::time::{SystemTime, UNIX_EPOCH};
    pub fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
