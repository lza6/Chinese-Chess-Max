use tracing::trace;

use crate::chess;
use crate::yolo;

// detections_bound 获取截图的边界
pub fn detections_bound(
    origin_width: u32,
    origin_height: u32,
    detections: &[yolo::Detection],
) -> Result<(u32, u32, u32, u32), String> {
    // 找到棋盘（label == '0'）
    let board_det = detections
        .iter()
        .find(|d| d.label == '0')
        .ok_or("未识别到棋盘")?;

    // 计算模型图到原图的缩放
    let scale_x = origin_width as f32 / yolo::IMAGE_WIDTH as f32;
    let scale_y = origin_height as f32 / yolo::IMAGE_HEIGHT as f32;

    // 模型坐标 → 原图坐标
    let bx0 = (board_det.x0 * scale_x).max(0.0);
    let by0 = (board_det.y0 * scale_y).max(0.0);
    let bx1 = (board_det.x1 * scale_x).min(origin_width as f32);
    let by1 = (board_det.y1 * scale_y).min(origin_height as f32);

    // 计算原图下的“半格”尺寸
    let board_w = bx1 - bx0;
    let board_h = by1 - by0;
    let half_cell_x = board_w / 9.0 / 2.0;
    let half_cell_y = board_h / 10.0 / 2.0;

    // 计算裁剪框左上
    let crop_x = (bx0 - half_cell_x).max(0.0) as u32;
    let crop_y = (by0 - half_cell_y).max(0.0) as u32;

    // 计算裁剪框右下，在原图范围内
    let x1p = (bx1 + half_cell_x).min(origin_width as f32);
    let y1p = (by1 + half_cell_y).min(origin_height as f32);

    // 宽高 = 右下 - 左上
    let width = (x1p - crop_x as f32) as u32;
    let height = (y1p - crop_y as f32) as u32;

    Ok((crop_x, crop_y, width, height))
}

const MODEL_CELL_W: f32 = yolo::IMAGE_WIDTH as f32 / 9.0;
const MODEL_CELL_H: f32 = yolo::IMAGE_HEIGHT as f32 / 10.0;

// detections_to_board 识别结果转换为棋盘结构
pub fn detections_to_board(
    detections: &[yolo::Detection],
) -> Result<(chess::Camp, [[char; 9]; 10]), String> {
    let mut camp = chess::Camp::None;
    let mut board = [[' '; 9]; 10];

    match detections.iter().find(|&&x| x.label == '0') {
        Some(_) => {
            for det in detections.iter().filter(|d| d.label != '0') {
                // 中心点
                let cx = (det.x0 + det.x1) / 2.0;
                let cy = (det.y0 + det.y1) / 2.0;
                // 行列：x 轴分成 9 格，y 轴分成 10 格
                let col = (cx / MODEL_CELL_W).floor() as usize; // 0–8
                let row = (cy / MODEL_CELL_H).floor() as usize; // 0–9
                trace!("{} row={} col={}", det.label, row, col);

                // 边界处理
                if !(0..=8).contains(&col) || !(0..=9).contains(&row) {
                    continue;
                }

                // 构建board
                board[row][col] = det.label;

                // 判断阵营
                if camp == chess::Camp::None && (3..=5).contains(&col) && row >= 7 {
                    match det.label {
                        'k' => camp = chess::Camp::Black,
                        'K' => camp = chess::Camp::Red,
                        _ => {}
                    }
                }
            }
        }
        None => return Err("not board".to_string()),
    }
    Ok((camp, board))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::yolo::Detection;

    fn det(x: f32, y: f32, w: f32, h: f32, label: char) -> Detection {
        // Detection::new 的 idx 决定 label；LABELS 中 '0' 在 idx=14
        let idx = match label {
            '0' => 14,
            'K' => 10,
            'k' => 3,
            'r' => 4,
            'R' => 7,
            _ => 0,
        };
        Detection::new(x, y, w, h, idx, 0.9)
    }

    #[test]
    fn detections_to_board_maps_center_to_cell() {
        // 棋盘检测（label '0'）必须存在
        let dets = vec![
            det(320.0, 320.0, 640.0, 640.0, '0'), // 整盘
            det(35.5, 35.5, 70.0, 70.0, 'K'),     // 左上角格（col0,row0）
            det(320.5, 604.5, 70.0, 70.0, 'k'),   // cx=355.5→col4, cy=639.5→row9（满足阵营判定）
        ];
        let (camp, board) = detections_to_board(&dets).unwrap();
        assert_eq!(board[0][0], 'K');
        assert_eq!(board[9][4], 'k');
        assert_eq!(camp, chess::Camp::Black); // 右下 k 在 row9 判定黑
    }

    #[test]
    fn detections_to_board_rejects_without_board() {
        let dets = vec![det(35.0, 35.0, 70.0, 70.0, 'K')];
        assert!(detections_to_board(&dets).is_err());
    }

    #[test]
    fn detections_to_board_skips_out_of_range() {
        // 中心点越界（col>8）的棋子被跳过
        let dets = vec![
            det(320.0, 320.0, 640.0, 640.0, '0'),
            det(670.0, 35.0, 70.0, 70.0, 'R'), // cx=705 → col=9 → 越界跳过
            det(35.5, 35.5, 70.0, 70.0, 'r'),  // col0,row0
        ];
        let (_, board) = detections_to_board(&dets).unwrap();
        assert_eq!(board[0][0], 'r');
        // col9 被跳过，不留棋子
        assert_eq!(board[0][8], ' '); // col9 越界跳过，col8 无棋子
    }

    #[test]
    fn detections_bound_computes_crop() {
        // 原始 1280x720，棋盘居中 label '0'
        let dets = vec![det(320.0, 320.0, 640.0, 640.0, '0')];
        let (x, y, w, h) = detections_bound(1280, 720, &dets).unwrap();
        // 棋盘框 0..640 (模型) → 缩放 x2 到 0..1280, y*1.125 到 0..720
        // half_cell = 640/9/2≈35.6, 720/10/2=36
        // crop_x ≈ max(0-35.6,0)=0; crop_y=0; x1p=1280; y1p=720
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert!(w > 1200 && w <= 1280);
        assert!(h > 640 && h <= 720);
    }

    #[test]
    fn detections_bound_rejects_without_board() {
        let dets = vec![det(35.0, 35.0, 70.0, 70.0, 'K')];
        assert!(detections_bound(1280, 720, &dets).is_err());
    }
}
