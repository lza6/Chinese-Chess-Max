pub mod chessdb;
use std::fmt::Display;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
mod command;

use tracing::debug;
use tracing::trace;

#[derive(Debug, serde::Serialize, Default, Clone)]
pub struct QueryResult {
    pub depth: usize,       // 深度
    pub score: isize,       // 得分
    pub time: usize,        // 时间
    pub pvs: Vec<String>,   // 思考(iccs)
    pub moves: Vec<String>, // 思考(chinese)
    pub state: QueryState,  // 状态
    pub source: String,     // 来源
}

const SOURCE_ENGINE: &str = "引擎";

#[derive(Debug, serde::Serialize, Default, Clone, Copy, PartialEq)]
pub enum QueryState {
    Success,
    #[default]
    NotResult,
    InvalidBoard,
    ServerInternalError, // 内部错误
}

#[derive(Debug, serde::Serialize, Clone, serde::Deserialize, Copy)]
pub struct EngineConfig {
    pub depth: usize,
    pub time: usize,
    pub threads: usize,
    pub hash: usize,
    pub show_wdl: bool,
    pub chessdb_enabled: bool,
    pub chessdb_timeout: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            depth: 20,
            time: 5000,
            threads: 4,
            hash: 64,
            show_wdl: false,
            chessdb_enabled: true,
            chessdb_timeout: 5,
        }
    }
}

pub struct Engine {
    stdin: Box<dyn Write>,
    stdout: Box<dyn BufRead>,
    child: std::process::Child, // 添加子进程字段
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Engine {
    pub fn new(libs: &Path) -> Result<Self, String> {
        let mut child = command::new(libs).map_err(|e| format!("引擎启动失败: {e}"))?;

        let nnue = libs.join("pikafish.nnue");

        let stdin = Box::new(
            child
                .stdin
                .take()
                .ok_or_else(|| "引擎 stdin 不可用".to_string())?,
        );
        let stdout = Box::new(BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| "引擎 stdout 不可用".to_string())?,
        ));

        let mut eng = Engine {
            stdin,
            stdout,
            child,
        };
        // UCI 握手：等 uciok，避免未就绪就 setoption/搜索
        eng.write_command("uci")?;
        eng.wait_until("uciok", "uci 初始化")
            .ok_or_else(|| "引擎 UCI 初始化超时或已退出".to_string())?;
        eng.setoption("EvalFile", nnue.display())?;
        eng.setoption("Sixty Move Rule", false)?;
        eng.isready()?;
        Ok(eng)
    }

    /// 发送 isready 并等待 readyok（同步引擎就绪）
    fn isready(&mut self) -> Result<(), String> {
        self.write_command("isready")?;
        self.wait_until("readyok", "isready")
            .ok_or_else(|| "引擎 isready 超时或已退出".to_string())?;
        Ok(())
    }

    /// 读取输出直到出现目标关键字；EOF 时返回 None。
    /// 同时设时间预算（默认 10s）与行数上限（10_000），引擎持续输出但不给目标时不会无限阻塞。
    fn wait_until(&mut self, keyword: &str, ctx: &str) -> Option<bool> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut tries = 0u32;
        loop {
            if std::time::Instant::now() >= deadline {
                tracing::warn!(
                    "{}({})：等待超时（10s 时间预算用尽，引擎可能卡死）",
                    ctx,
                    keyword
                );
                return None;
            }
            match self.read_line() {
                None => {
                    tracing::warn!("{}({})：引擎输出 EOF 提前退出", ctx, keyword);
                    return None;
                }
                Some(line) => {
                    if line.contains(keyword) {
                        return Some(true);
                    }
                    tries += 1;
                    if tries > 10_000 {
                        tracing::warn!("{}({})：等待超时（引擎输出行数过多）", ctx, keyword);
                        return None;
                    }
                }
            }
        }
    }

    pub fn reload(&mut self, libs: &Path, config: &EngineConfig) -> Result<(), String> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        *self = Self::new(libs)?;
        self.set_hash(config.hash)?;
        self.set_show_wdl(config.show_wdl)?;
        self.set_threads(config.threads)?;
        self.isready()?;
        Ok(())
    }

    fn write_command<A: Display>(&mut self, args: A) -> Result<(), String> {
        writeln!(self.stdin, "{}", args).map_err(|e| format!("写入引擎命令失败: {e}"))?;
        self.stdin
            .flush()
            .map_err(|e| format!("刷新引擎 stdin 失败: {e}"))?;
        debug!("{}", args);
        Ok(())
    }

    pub fn set_show_wdl(&mut self, open: bool) -> Result<(), String> {
        self.setoption("UCI_ShowWDL", open)
    }

    pub fn set_threads(&mut self, num: usize) -> Result<(), String> {
        self.setoption("Threads", num)
    }

    pub fn set_hash(&mut self, size: usize) -> Result<(), String> {
        self.setoption("Hash", size)
    }

    pub fn setoption<T: Display>(&mut self, name: &str, value: T) -> Result<(), String> {
        self.write_command(format!("setoption name {} value {}", name, value))
    }

    pub fn position(&mut self, fen: &str) -> Result<(), String> {
        self.write_command(format!("position fen {}", fen))
    }

    /// 读一行；引擎进程退出(EOF)时返回 None，避免无限空转。
    /// 引擎横幅可能含非 UTF-8 字节，用 read_until + lossy 转换容忍。
    fn read_line(&mut self) -> Option<String> {
        let mut buf: Vec<u8> = Vec::with_capacity(128);
        match self.stdout.read_until(b'\n', &mut buf) {
            Ok(0) => {
                tracing::warn!("engine stdout EOF（引擎可能已退出）");
                None
            }
            Ok(_) => {
                let line = String::from_utf8_lossy(&buf);
                trace!("line::{}", line.trim_end());
                Some(line.trim().to_string())
            }
            Err(e) => {
                tracing::warn!("engine read_line error: {}", e);
                None
            }
        }
    }

    fn parse_line(line: String, result: &mut QueryResult) {
        let mut iter = line.split_whitespace();
        result.source = SOURCE_ENGINE.to_string();
        loop {
            if let Some(key) = iter.next() {
                match key {
                    "depth" => {
                        if let Some(d) = iter.next() {
                            result.depth = d.parse().unwrap_or(result.depth);
                        }
                    }
                    "time" => {
                        if let Some(t) = iter.next() {
                            result.time = t.parse().unwrap_or(result.time);
                        }
                    }
                    "score" => match iter.next().unwrap_or("") {
                        "cp" => {
                            if let Some(s) = iter.next() {
                                result.score = s.parse().unwrap_or(result.score);
                            }
                        }
                        "mate" => {
                            if let Some(round) = iter.next() {
                                let round: isize = round.parse().unwrap_or(0);
                                result.score = if round > 0 {
                                    30000 - round
                                } else {
                                    -(30000 + round)
                                };
                            }
                        }
                        _ => {}
                    },
                    "pv" => loop {
                        if let Some(pv) = iter.next() {
                            result.pvs.push(pv.to_string());
                            continue;
                        }
                        break;
                    },
                    _ => {}
                }
                continue;
            }
            break;
        }
    }

    fn bestmove(&mut self, depth: usize, time: usize) -> String {
        if let Err(e) = self.write_command(format!("go depth {} movetime {}", depth, time)) {
            tracing::error!("bestmove: 写入 go 命令失败: {e}");
            return String::new();
        }
        let mut pre_line = String::new();
        loop {
            let Some(line) = self.read_line() else {
                tracing::error!("bestmove: 引擎无响应(EOF)，放弃本次搜索");
                return String::new();
            };
            if line.starts_with("bestmove") {
                trace!("{}", pre_line);
                break;
            }
            pre_line = line;
        }
        pre_line
    }

    pub async fn search(&mut self, fen: &str, params: &EngineConfig) -> Option<QueryResult> {
        let mut result = if params.chessdb_enabled {
            // 先查询云库
            chessdb::query(fen, params.chessdb_timeout).await
        } else {
            QueryResult::default()
        };

        match result.state {
            QueryState::Success => Some(result),
            QueryState::InvalidBoard => None,
            QueryState::ServerInternalError | QueryState::NotResult => {
                // 查询云库失败调用引擎
                if self.position(fen).is_err() {
                    tracing::error!("search: position 命令发送失败，跳过本次分析");
                    return None;
                }
                let best_line = self.bestmove(params.depth, params.time);
                Self::parse_line(best_line, &mut result);
                Some(result)
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.write_command("quit");
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_handles_mate_and_missing_fields() {
        let mut r = QueryResult::default();
        Engine::parse_line("info depth 12 score mate 3 pv e2e4".to_string(), &mut r);
        assert_eq!(r.score, 30000 - 3);
        assert_eq!(r.depth, 12);
        assert!(r.pvs.iter().any(|p| p == "e2e4"));
    }

    #[test]
    fn parse_line_tolerates_bad_numbers() {
        let mut r = QueryResult {
            depth: 7,
            ..Default::default()
        };
        Engine::parse_line("info depth xyz score cp abc pv".to_string(), &mut r);
        assert_eq!(r.depth, 7);
        assert_eq!(r.score, 0);
    }
}
