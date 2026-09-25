use std::cmp::Ordering;
use std::collections::HashMap;

use serde::Serialize;
use tracing::warn;

pub const BOARD_MAP: [[&str; 9]; 10] = [
    ["a9", "b9", "c9", "d9", "e9", "f9", "g9", "h9", "i9"],
    ["a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", "i8"],
    ["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "i7"],
    ["a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", "i6"],
    ["a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "i5"],
    ["a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "i4"],
    ["a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "i3"],
    ["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "i2"],
    ["a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "i1"],
    ["a0", "b0", "c0", "d0", "e0", "f0", "g0", "h0", "i0"],
];

#[derive(Debug, Serialize)]
pub struct Position {
    piece: char,
    pos: String,
}

#[derive(Debug)]
pub enum BoardChangeState {
    // 变化了一个棋子
    One,
    // 正常一步棋移动
    Move,
    // 未知多个变化
    Unknown,
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize)]
pub enum Camp {
    #[default]
    None,
    Red,
    Black,
}

impl Camp {
    pub fn to_char(&self) -> char {
        match self {
            Camp::None => '0',
            Camp::Red => 'w',
            Camp::Black => 'b',
        }
    }

    pub fn from_piece(p: char) -> Self {
        if p > 'Z' { Self::Black } else { Self::Red }
    }

    pub fn is_black(&self) -> bool {
        Camp::Black.eq(self)
    }

    pub fn from_char(c: char) -> Self {
        match c {
            'b' => Camp::Black,
            _ => Camp::Red,
        }
    }
}

const BLACK_VERTICALS: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
const RED_VERTICALS: [char; 9] = ['九', '八', '七', '六', '五', '四', '三', '二', '一'];

pub(crate) const RED_STARTPOS: [[char; 9]; 10] = [
    ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
    ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
    [' ', 'C', ' ', ' ', ' ', ' ', ' ', 'C', ' '],
    [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
];

pub fn get_verticals(p: char) -> [char; 9] {
    if p > 'Z' {
        BLACK_VERTICALS
    } else {
        RED_VERTICALS
    }
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Changed {
    pub piece: char,
    pub camp: Camp,
    pub from: String,
    pub to: String,
}

/// 校验并解析 ICCS 坐标（如 "e2e4"）：必须 4 个 ASCII 字符，file a-i，rank 0-9
/// 返回 (from_x, from_y, to_x, to_y)，非法输入返回 Err
pub fn parse_iccs(iccs: &str) -> Result<(usize, usize, usize, usize), String> {
    let b = iccs.as_bytes();
    if b.len() != 4 {
        return Err(format!("非法着法(长度≠4): {iccs:?}"));
    }
    let sq = |c: u8, is_file: bool| -> Result<usize, ()> {
        if is_file {
            // 文件: 'a'..='i'
            if c.is_ascii_lowercase() && (b'a'..=b'i').contains(&c) {
                Ok((c - b'a') as usize)
            } else {
                Err(())
            }
        } else {
            // 行: '0'..='9'
            if c.is_ascii_digit() {
                // 行号 '0' -> y=9（黑方底线 row0 对应 FEN 首行），'9' -> y=0（红方底线）
                Ok((9 - (c - b'0')) as usize)
            } else {
                Err(())
            }
        }
    };
    let (from_x, from_y, to_x, to_y) = (
        sq(b[0], true),
        sq(b[1], false),
        sq(b[2], true),
        sq(b[3], false),
    );
    match (from_x, from_y, to_x, to_y) {
        (Ok(fx), Ok(fy), Ok(tx), Ok(ty)) => Ok((fx, fy, tx, ty)),
        _ => Err(format!("非法着法坐标: {iccs:?}")),
    }
}

pub fn red_startpos() -> [[char; 9]; 10] {
    RED_STARTPOS
}

impl Changed {
    pub fn from_pv(pv: &str, board: [[char; 9]; 10]) -> Result<Self, String> {
        let (from_x, from_y, _, _) = parse_iccs(pv)?;
        let (from, to) = pv.split_at(2);
        let piece = board[from_y][from_x];
        Ok(Self {
            piece,
            camp: Camp::from_piece(piece),
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

// 对比棋盘, 返回值是发生变化的索引
pub fn board_diff(
    old_board: [[char; 9]; 10],
    board: [[char; 9]; 10],
) -> (Changed, BoardChangeState) {
    let mut changed = Changed::default();
    let mut count = 0;
    for y in 0..10 {
        for x in 0..9 {
            if old_board[y][x] != board[y][x] {
                count += 1;
                match board[y][x] {
                    ' ' => {
                        changed.piece = old_board[y][x];
                        changed.from = BOARD_MAP[y][x].to_string();
                        changed.camp = Camp::from_piece(changed.piece);
                    }
                    _ => changed.to = BOARD_MAP[y][x].to_string(),
                }
            }
        }
    }

    match count {
        1 => (changed, BoardChangeState::One),
        2 => {
            if changed.from.is_empty() || changed.to.is_empty() {
                (changed, BoardChangeState::One)
            } else {
                (changed, BoardChangeState::Move)
            }
        }
        _ => (changed, BoardChangeState::Unknown),
    }
}

pub struct Move {
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
}

impl Move {
    pub fn new(iccs: &str) -> Result<Self, String> {
        let (from_x, from_y, to_x, to_y) = parse_iccs(iccs)?;
        Ok(Self {
            from_x,
            from_y,
            to_x,
            to_y,
        })
    }
}

pub fn board_move(board: [[char; 9]; 10], iccs: &str) -> Result<[[char; 9]; 10], String> {
    let mv = Move::new(iccs)?;
    let mut new_board = board;
    let p = new_board[mv.from_y][mv.from_x];
    new_board[mv.to_y][mv.to_x] = p;
    new_board[mv.from_y][mv.from_x] = ' ';
    Ok(new_board)
}

// 棋盘转换FEN逻辑
pub fn board_fen(camp: &Camp, board: [[char; 9]; 10]) -> String {
    let mut fen = String::new();
    for row in &board {
        let mut empty = 0;
        for &piece in row {
            if piece == ' ' {
                empty += 1;
            } else {
                if empty > 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                fen.push(piece);
            }
        }
        if empty > 0 {
            fen.push_str(&empty.to_string());
        }
        fen.push('/');
    }
    fen.pop();
    fen.push(' ');
    fen.push(camp.to_char());
    fen
}

// 检测棋盘是否合法
pub fn board_check(board: [[char; 9]; 10]) -> bool {
    let mut bk = 0;
    let mut ba = 0;
    let mut bb = 0;
    let mut bc = 0;
    let mut bp = 0;
    let mut br = 0;
    let mut bn = 0;
    let mut rk = 0;
    let mut ra = 0;
    let mut rb = 0;
    let mut rc = 0;
    let mut rp = 0;
    let mut rr = 0;
    let mut rn = 0;

    for (y, row) in board.iter().enumerate() {
        for (x, &col) in row.iter().enumerate() {
            match col {
                'k' => {
                    bk += 1;
                    if y > 2 || !(3..=5).contains(&x) {
                        warn!("黑方'将'不在合法位置内");
                        return false;
                    }
                }
                'a' => {
                    ba += 1;
                    if !(x == 3 && (y == 0 || y == 2))
                        && !(x == 4 && y == 1)
                        && !(x == 5 && (y == 0 || y == 2))
                    {
                        warn!("黑方'士'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'b' => {
                    bb += 1;
                    if !(y == 0 && (x == 2 || x == 6))
                        && !(y == 2 && (x == 0 || x == 4 || x == 8))
                        && !(y == 4 && (x == 2 || x == 6))
                    {
                        warn!("黑方'象'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'c' => {
                    bc += 1;
                }
                'p' => {
                    bp += 1;
                    if y < 3 {
                        warn!("黑方'兵'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                    if y < 5 && x % 2 == 1 {
                        warn!("黑方'兵'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'r' => {
                    br += 1;
                }
                'n' => {
                    bn += 1;
                }
                'K' => {
                    rk += 1;
                    if y < 7 || !(3..=5).contains(&x) {
                        warn!("红方'将'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'A' => {
                    ra += 1;
                    if !(x == 3 && (y == 7 || y == 9))
                        && !(x == 4 && y == 8)
                        && !(x == 5 && (y == 7 || y == 9))
                    {
                        warn!("红方'士'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'B' => {
                    rb += 1;
                    if !(y == 9 && (x == 2 || x == 6))
                        && !(y == 7 && (x == 0 || x == 4 || x == 8))
                        && !(y == 5 && (x == 2 || x == 6))
                    {
                        warn!("红方'象'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'C' => {
                    rc += 1;
                }
                'P' => {
                    rp += 1;
                    if y > 6 {
                        warn!("红方'兵'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                    if y > 4 && x % 2 == 1 {
                        warn!("红方'兵'不在合法位置内, ({}行{}列)", y, x);
                        return false;
                    }
                }
                'R' => {
                    rr += 1;
                }
                'N' => {
                    rn += 1;
                }
                _ => {}
            }
        }
    }

    if bk != 1 || rk != 1 {
        warn!("黑方或红方'将'超出合法数量(红:{}, 黑:{})", rk, bk);
        return false;
    }
    if ba > 2 || ra > 2 {
        warn!("黑方或红方'士'超出合法数量(红:{}, 黑:{})", ra, ba);
        return false;
    }
    if bb > 2 || rb > 2 {
        warn!("黑方或红方'象'超出合法数量(红:{}, 黑:{})", rb, bb);
        return false;
    }
    if bc > 2 || rc > 2 {
        warn!("黑方或红方'炮'超出合法数量(红:{}, 黑:{})", rc, bc);
        return false;
    }
    if br > 2 || rr > 2 {
        warn!("黑方或红方'车'超出合法数量(红:{}, 黑:{})", rr, br);
        return false;
    }
    if bn > 2 || rn > 2 {
        warn!("黑方或红方'马'超出合法数量(红:{}, 黑:{})", rn, bn);
        return false;
    }
    if bp > 5 || rp > 5 {
        warn!("黑方或红方'兵'超出合法数量(红:{}, 黑:{})", rp, bp);
        return false;
    }
    true
}

pub const fn get_piece_name(piece: char) -> char {
    match piece {
        'K' => '帅',
        'k' => '将',
        'A' => '仕',
        'a' => '士',
        'B' => '相',
        'b' => '象',
        'N' => '马',
        'n' => '马',
        'R' => '车',
        'r' => '车',
        'C' => '炮',
        'c' => '炮',
        'P' => '兵',
        'p' => '卒',
        _ => ' ',
    }
}

pub fn startpos(board: [[char; 9]; 10]) -> bool {
    board == RED_STARTPOS
}

pub fn board_fix(camp: &Camp, board: &mut [[char; 9]; 10]) {
    if Camp::Black.eq(camp) {
        board.reverse();
        for i in board {
            i.reverse()
        }
    }
}

// board_to_map 棋盘数组转换为坐标模式
pub fn board_map(board: [[char; 9]; 10]) -> Vec<Position> {
    let mut position = vec![];

    for row in 0..10 {
        for col in 0..9 {
            position.push(Position {
                piece: board[row][col],
                pos: BOARD_MAP[row][col].to_string(),
            });
        }
    }
    position
}

fn overlap_piece_y(board: [[char; 9]; 10], x: usize, y: usize, piece: char) -> Vec<usize> {
    let mut other_ys = Vec::new();
    for (i, value) in board.iter().enumerate() {
        if i != y && value[x] == piece {
            other_ys.push(i);
        }
    }
    other_ys
}

fn overlap_piece_xy(
    board: [[char; 9]; 10],
    from_x: usize,
    piece: char,
) -> Option<HashMap<usize, Vec<usize>>> {
    let mut other_xys = HashMap::new();
    for x in 0..9 {
        if x != from_x {
            let other_ys = overlap_piece_y(board, x, 10, piece); // 注意：这里将y设置为10是一个技巧，因为我们的棋盘只有10行，所以永远不会找到y==10的情况。但这样做并不是最好的方式。
            if other_ys.len() > 1 {
                other_xys.insert(x, other_ys);
                return Some(other_xys);
            }
        }
    }
    None
}

// 棋子坐标移动转中文模式
pub fn board_move_chinese(board: [[char; 9]; 10], iccs: &str) -> String {
    let mut chinese = String::new();
    let Ok(mv) = Move::new(iccs) else {
        return String::new();
    };
    let piece = board[mv.from_y][mv.from_x];
    let verticals = get_verticals(piece);
    match piece {
        'K' => {
            chinese.push(get_piece_name(piece));
            chinese.push(verticals[mv.from_x]);

            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Equal => {
                    // 平
                    chinese.push('平');
                    chinese.push(verticals[mv.to_x]);
                }
                Ordering::Less => {
                    // 进
                    chinese.push('退');
                    chinese.push(verticals[8]);
                }
                Ordering::Greater => {
                    // 退
                    chinese.push('进');
                    chinese.push(verticals[8]);
                }
            }
        }
        'k' => {
            chinese.push(get_piece_name(piece));
            chinese.push(verticals[mv.from_x]);

            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Less => {
                    // 退
                    chinese.push('进');
                    chinese.push(verticals[0]);
                }
                Ordering::Greater => {
                    // 进
                    chinese.push('退');
                    chinese.push(verticals[0]);
                }
                Ordering::Equal => {
                    // 平
                    chinese.push('平');
                    chinese.push(verticals[mv.to_x]);
                }
            }
        }
        'A' | 'B' => {
            chinese.push(get_piece_name(piece));
            chinese.push(verticals[mv.from_x]);
            // 士、象只有进退
            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Less => chinese.push('退'),
                _ => chinese.push('进'),
            }
            chinese.push(verticals[mv.to_x]);
        }
        'a' | 'b' => {
            chinese.push(get_piece_name(piece));
            chinese.push(verticals[mv.from_x]);
            // 士、象只有进退
            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Less => chinese.push('进'),
                _ => chinese.push('退'),
            }
            chinese.push(verticals[mv.to_x]);
        }
        'R' | 'C' => {
            // 判断是否有纵向重叠情况
            let other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);
            if other_ys.is_empty() {
                // 非重叠情况
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                // 重叠, 判断前后
                if mv.from_y > other_ys[0] {
                    chinese.push('后')
                } else {
                    chinese.push('前')
                }
                chinese.push(get_piece_name(piece));
            }
            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Less => {
                    // 退
                    let step = 9 - mv.to_y + mv.from_y;
                    chinese.push('退');
                    chinese.push(verticals[step]);
                }
                Ordering::Greater => {
                    // 进
                    let step = 9 - mv.from_y + mv.to_y;
                    chinese.push('进');
                    chinese.push(verticals[step]);
                }
                Ordering::Equal => {
                    // 平
                    chinese.push('平');
                    chinese.push(verticals[mv.to_x]);
                }
            }
        }
        'r' | 'c' => {
            // 判断是否有纵向重叠情况
            let other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);
            if other_ys.is_empty() {
                // 非重叠情况
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                // 重叠, 判断前后
                if mv.from_y > other_ys[0] {
                    chinese.push('前')
                } else {
                    chinese.push('后')
                }
                chinese.push(get_piece_name(piece));
            }
            match mv.from_y.cmp(&mv.to_y) {
                Ordering::Less => {
                    // 进
                    let step = mv.to_y - mv.from_y - 1;
                    chinese.push('进');
                    chinese.push(verticals[step]);
                }
                Ordering::Greater => {
                    // 退
                    let step = mv.from_y - mv.to_y - 1;
                    chinese.push('退');
                    chinese.push(verticals[step]);
                }
                Ordering::Equal => {
                    // 平
                    chinese.push('平');
                    chinese.push(verticals[mv.to_x]);
                }
            }
        }
        'N' => {
            // 判断是否有纵向重叠情况
            let other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);

            if other_ys.is_empty() {
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                if mv.from_y > other_ys[0] {
                    chinese.push('后');
                } else {
                    chinese.push('前');
                }
                chinese.push(get_piece_name(piece));
            }

            if mv.from_y < mv.to_y {
                chinese.push('退');
            } else {
                chinese.push('进');
            }
            chinese.push(verticals[mv.to_x]);
        }
        'n' => {
            // 判断是否有纵向重叠情况
            let other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);
            if other_ys.is_empty() {
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                if mv.from_y > other_ys[0] {
                    chinese.push('前');
                } else {
                    chinese.push('后');
                }
                chinese.push(get_piece_name(piece));
            }

            if mv.from_y < mv.to_y {
                chinese.push('进')
            } else {
                chinese.push('退')
            }
            chinese.push(verticals[mv.to_x]);
        }
        'P' => {
            // 判断是否有纵向重叠情况
            let mut other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);
            if other_ys.is_empty() {
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                // 判断其他纵线上是否有重叠
                if let Some(other_xys) = overlap_piece_xy(board, mv.from_x, piece) {
                    // 其他纵线有重叠
                    for y in &mut other_ys {
                        *y = (mv.from_x + 10) * 100 - *y;
                    }

                    for (x, ys) in &other_xys {
                        for y in ys {
                            other_ys.push((x + 10) * 100 - y);
                        }
                    }
                    // 降序
                    let value = (mv.from_x + 10) * 100 - mv.from_y;
                    other_ys.push(value);
                    other_ys.sort_by(|a, b| b.cmp(a));
                    let seq = other_ys
                        .iter()
                        .position(|&v| v == value)
                        .map(|i| i + 1)
                        .unwrap_or(1);
                    chinese.push(verticals[9 - seq]);
                } else if other_ys.len() > 1 {
                    // 找出当前纵向重叠数量
                    let mut num = 1;
                    for y in other_ys {
                        if mv.from_y < y {
                            break;
                        }
                        num += 1;
                    }
                    chinese.push_str(num.to_string().as_str());
                } else {
                    // 只有前后
                    if mv.from_y > other_ys[0] {
                        chinese.push('后');
                    } else {
                        chinese.push('前');
                    }
                }
                chinese.push(get_piece_name(piece));
            }

            if mv.from_y == mv.to_y {
                // 平
                chinese.push('平');
                chinese.push(verticals[mv.to_x]);
            } else {
                // 进
                chinese.push('进');
                chinese.push(verticals[8]);
            }
        }
        'p' => {
            // 判断是否有纵向重叠情况
            let mut other_ys = overlap_piece_y(board, mv.from_x, mv.from_y, piece);
            if other_ys.is_empty() {
                chinese.push(get_piece_name(piece));
                chinese.push(verticals[mv.from_x]);
            } else {
                // 判断其他纵线上是否有重叠
                if let Some(other_xys) = overlap_piece_xy(board, mv.from_x, piece) {
                    // 其他纵线有重叠
                    for y in &mut other_ys {
                        *y += mv.from_x * 100;
                    }

                    for (x, ys) in &other_xys {
                        for y in ys {
                            other_ys.push(x * 100 + y);
                        }
                    }
                    // 升序
                    let value = mv.from_x * 100 + mv.from_y;
                    other_ys.push(value);
                    other_ys.sort_by(|a, b| b.cmp(a));
                    let seq = other_ys.iter().position(|&v| v == value).unwrap_or(0);
                    chinese.push(verticals[seq]);
                } else if other_ys.len() > 1 {
                    // 找出当前纵向重叠数量
                    let mut num = 0;
                    for y in other_ys {
                        if mv.from_y > y {
                            break;
                        }
                        num += 1;
                    }
                    chinese.push_str(num.to_string().as_str());
                } else {
                    // 只有前后
                    if mv.from_y > other_ys[0] {
                        chinese.push('前');
                    } else {
                        chinese.push('后');
                    }
                }
                chinese.push(get_piece_name(piece));
            }

            if mv.from_y == mv.to_y {
                // 平
                chinese.push('平');
                chinese.push(verticals[mv.to_x]);
            } else {
                // 进
                chinese.push('进');
                chinese.push(verticals[0]);
            }
        }
        _ => {}
    }

    chinese
}

#[allow(dead_code)]
/// 解析 FEN 到棋盘；非法 FEN 尽力容忍（不 panic），越界字符跳过。
pub fn fen_to_board(mut fen: &str) -> [[char; 9]; 10] {
    if fen.contains(' ') {
        fen = fen.split_once(' ').map(|(board, _)| board).unwrap_or(fen);
    }
    let mut board = [[' '; 9]; 10];
    let mut rank = 0;
    let mut file = 0;
    for c in fen.chars() {
        match c {
            '1'..='9' => {
                if let Some(d) = c.to_digit(10) {
                    file += d as usize;
                }
            }
            '/' => {
                rank += 1;
                file = 0;
                if rank >= 10 {
                    break;
                }
            }
            _ => {
                if rank < 10 && file < 9 {
                    board[rank][file] = c;
                    file += 1;
                }
            }
        }
    }
    board
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red_board_to_fen() {
        let board = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        let expected_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/9/RNBAKABNR w";
        assert_eq!(board_fen(&Camp::Red, board), expected_fen);
    }

    #[test]
    fn test_fen_to_board() {
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/9/RNBAKABNR";
        let board = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        assert_eq!(fen_to_board(fen), board);
    }

    #[test]
    fn test_chinese() {
        let fen = "2rakab2/9/1cn6/p3p3p/2b2n3/6R2/P3P1c1P/2N1C3C/4N4/2BAKAB2 w";
        let mut board = fen_to_board(fen);
        for pv in [
            "g4g5", "b7b5", "g5g9", "c5e7", "g9g4", "f9e8", "i2i6", "c7d5",
        ] {
            let notice = board_move_chinese(board, pv);
            board = board_move(board, pv).unwrap();
            println!("pv: {} => {}", pv, notice);
        }
    }

    #[test]
    fn test_board_check() {
        let board = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'a', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        println!("{}", board_check(board))
    }

    #[test]
    fn test_board_diff() {
        let old = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'a', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        let new = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'a', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', ' ', 'P', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        let (changed, state) = board_diff(old, new);
        println!("{:?} {:?}", changed, state);
    }

    #[test]
    fn test_board_map() {
        let board = [
            ['r', 'n', 'b', 'a', 'k', 'a', 'a', 'n', 'r'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', 'C', ' ', ' ', 'C', ' ', ' ', ' ', ' '],
            ['R', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
        ];
        println!("{:?}", board_map(board))
    }

    #[test]
    fn test_board_fix() {
        let mut board: [[char; 9]; 10] = [
            ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', 'C', ' ', ' ', 'C', ' '],
            ['P', ' ', 'P', ' ', 'P', ' ', 'P', ' ', 'P'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['p', ' ', 'p', ' ', 'p', ' ', 'p', ' ', 'p'],
            [' ', 'c', ' ', ' ', ' ', ' ', ' ', 'c', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'],
        ];
        board_fix(&Camp::Black, &mut board);
        println!("{:?}", board)
    }

    #[test]
    fn test_parse_iccs_rejects_invalid_inputs() {
        // 空串 / 短串 / 越界坐标 / 非 ASCII：必须 Err，绝不 panic
        for bad in [
            "", "a", "a1", "a1b", "0000",  // 空着法（UCI null move）
            "z9z9",  // 文件越界
            "a;a1",  // 非数字行
            "帅2e4", // 非 ASCII
            "a10a1", // 行越界（两字符）
        ] {
            assert!(parse_iccs(bad).is_err(), "should reject: {bad:?}");
        }
        // 合法着法
        for good in ["e2e4", "b7c7", "i9i8", "a0a1"] {
            assert!(parse_iccs(good).is_ok(), "should accept: {good:?}");
        }
    }

    #[test]
    fn test_board_move_rejects_invalid_without_panic() {
        let board = red_startpos();
        assert!(board_move(board, "").is_err());
        assert!(board_move(board, "0000").is_err());
        assert!(board_move(board, "z9z9").is_err());
        assert!(board_move(board, "帅2e4").is_err());
        // from_pv 同源校验
        assert!(Changed::from_pv("", board).is_err());
        assert!(Changed::from_pv("0000", board).is_err());
    }

    #[test]
    fn test_red_startpos_matches_standard_fen_orientation() {
        // 标准 FEN 初始局面：黑方在上（小写），首行 "rnbakabnr"
        let fen = board_fen(&Camp::Red, red_startpos());
        assert!(
            fen.starts_with("rnbakabnr/"),
            "initial FEN must be standard orientation, got: {fen}"
        );
        // startpos() 对 red_startpos() 为真
        assert!(startpos(red_startpos()));
    }

    #[test]
    fn test_board_move_apply_then_diff_roundtrip() {
        // 应用走子后 board_diff 应识别为 Move，且走子前后局面与 FEN 一致（防双重应用回归）
        let board = red_startpos();
        let after = board_move(board, "e3e4").unwrap();
        assert_eq!(after[5][4], 'P'); // 红兵从 row6 col4 (e3) 进到 row5 col4 (e4)
        assert_eq!(after[6][4], ' '); // 原位置 row6 col4 清空
        // diff(board, after) 应为一个 Move
        let (changed, state) = board_diff(board, after);
        assert!(matches!(state, BoardChangeState::Move), "expected Move");
        assert_eq!(changed.from, "e3");
        assert_eq!(changed.to, "e4");
        // 用 from_pv 重建后 from/to 一致
        let rebuilt = Changed::from_pv("e3e4", board).unwrap();
        assert_eq!(rebuilt.from, "e3");
        assert_eq!(rebuilt.to, "e4");
        assert_eq!(rebuilt.piece, 'P');
    }

    #[test]
    fn test_fen_to_board_tolerates_malformed_without_panic() {
        // 非法/畸形 FEN：必须不 panic（防御性）
        let _ = fen_to_board(""); // 空
        let _ = fen_to_board("rnbakabnr/9"); // 缺行
        let _ = fen_to_board("aaaaaaaaaaaaaaaaaaaa"); // 无斜杠超长
        let _ = fen_to_board("z".repeat(20).as_str()); // 越界文件
        let _ = fen_to_board("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - -"); // 含附加字段
        // 合法 FEN 仍正确解析
        let b = fen_to_board("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w");
        assert_eq!(b[0][0], 'r');
        assert_eq!(b[9][0], 'R');
    }
}
