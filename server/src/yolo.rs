use std::iter::Iterator;
use std::sync::OnceLock;

use ndarray::Array;
use ndarray::s;
use ort::inputs;
use ort::session::builder::GraphOptimizationLevel;
use xcap::image::DynamicImage;
use xcap::image::GenericImageView;
use xcap::image::ImageBuffer;
use xcap::image::Rgba;
use xcap::image::imageops::FilterType;

static SESSION: OnceLock<ort::Result<ort::session::Session>> = OnceLock::new();

pub const IMAGE_WIDTH: usize = 640;
pub const IMAGE_HEIGHT: usize = 640;
const CONFIDENCE_THRESHOLD: f32 = 0.7;
const IOU_THRESHOLD: f32 = 0.5;
const LABELS: [char; 15] = [
    'n', 'b', 'a', 'k', 'r', 'c', 'p', 'R', 'N', 'A', 'K', 'B', 'C', 'P', '0',
];
const LIMIT: [usize; 15] = [2, 2, 2, 1, 2, 2, 5, 2, 2, 2, 1, 2, 2, 5, 1];

#[cfg(not(feature = "rotate"))]
const MODEL_BYTES: &[u8] = include_bytes!("../../libs/large.onnx");

#[cfg(feature = "rotate")]
const MODEL_BYTES: &[u8] = include_bytes!("../../libs/rotate.onnx");

pub fn session() -> &'static ort::Result<ort::session::Session> {
    // 惰性初始化：失败时缓存 Err，不 panic（GPU/模型不可用 -> predict 返回错误 -> 监听线程降级）
    SESSION.get_or_init(|| {
        #[cfg(all(target_os = "windows", feature = "gpu"))]
        let eps = [
            ort::execution_providers::CUDAExecutionProvider::default().build(),
            ort::execution_providers::DirectMLExecutionProvider::default().build(),
        ];

        #[cfg(all(target_os = "windows", not(feature = "gpu")))]
        let eps = [];

        #[cfg(target_os = "macos")]
        let eps = [ort::execution_providers::CoreMLExecutionProvider::default().build()];

        #[cfg(target_os = "linux")]
        let eps = [ort::execution_providers::CUDAExecutionProvider::default().build()];

        ort::init()
            .with_execution_providers(eps)
            .commit()
            .and_then(|_| {
                let builder = ort::session::Session::builder()?;
                let builder = builder.with_optimization_level(GraphOptimizationLevel::Level3)?;
                builder.commit_from_memory(MODEL_BYTES)
            })
    })
}
pub fn predict(origin_img: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ort::Result<Vec<Detection>> {
    let img = DynamicImage::from(origin_img).resize_exact(
        IMAGE_WIDTH as u32,
        IMAGE_HEIGHT as u32,
        FilterType::Triangle,
    );
    let mut input = Array::zeros((1, 3, IMAGE_WIDTH, IMAGE_HEIGHT));
    for (x, y, pixel) in img.pixels() {
        let [r, g, b, _] = pixel.0;
        input[[0, 0, y as usize, x as usize]] = r as f32 / 255.0;
        input[[0, 1, y as usize, x as usize]] = g as f32 / 255.0;
        input[[0, 2, y as usize, x as usize]] = b as f32 / 255.0;
    }
    let session_ref: &ort::session::Session = match session().as_ref() {
        Ok(s) => s,
        Err(e) => {
            return Err(ort::Error::wrap(std::io::Error::other(format!(
                "模型初始化失败: {e}"
            ))));
        }
    };
    let output = session_ref
        .run(inputs!["images" => input.view()]?)?
        .remove("output0")
        .ok_or_else(|| ort::Error::wrap(std::io::Error::other("模型输出缺少 output0".to_string())))?
        .try_extract_tensor::<f32>()?
        .view()
        .t()
        .slice(s![.., .., 0])
        .t()
        .to_owned();
    let mut detections = output
        .rows()
        .into_iter()
        .filter_map(|row| {
            // YOLOv11 输出契约：4 坐标 + 15 类别（无 objectness），stride=19
            let (class_id, conf) = (4..19)
                .map(|idx| (idx - 4, row[idx]))
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)) // NaN 安全
                .unwrap();

            if conf < CONFIDENCE_THRESHOLD {
                None
            } else {
                Some(Detection::new(
                    row[0], row[1], row[2], row[3], class_id, conf,
                ))
            }
        })
        .collect();

    Ok(nms(&mut detections))
}

#[derive(Debug, Clone, Copy)]
pub struct Detection {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub confidence: f32,
    pub label: char,

    idx: usize,
    area: f32,
}

impl Detection {
    fn new(x: f32, y: f32, w: f32, h: f32, idx: usize, confidence: f32) -> Self {
        Self {
            x0: x - w / 2.0,
            x1: x + w / 2.0,
            y0: y - h / 2.0,
            y1: y + h / 2.0,
            area: w * h,
            idx,
            label: LABELS[idx],
            confidence,
        }
    }

    // 计算两个检测框的IOU(交并比)
    #[inline]
    fn iou(&self, other: &Detection) -> f32 {
        let inter_width = (self.x1.min(other.x1) - self.x0.max(other.x0)).max(0.0);
        let inter_height = (self.y1.min(other.y1) - self.y0.max(other.y0)).max(0.0);
        let intersection = inter_width * inter_height;
        intersection / (self.area + other.area - intersection)
    }
}

// 使用IOU计算去除重叠的检测框（非极大值抑制）
fn nms(detections: &mut Vec<Detection>) -> Vec<Detection> {
    detections.sort_unstable_by(|a, b| {
        // NaN 安全排序
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Less)
    });
    let mut filtered_detections = Vec::with_capacity(33);
    let mut sizemap = [0; 15];
    while let Some(current) = detections.pop() {
        // 跳过多出的部分
        if sizemap[current.idx] + 1 > LIMIT[current.idx] {
            continue;
        }
        filtered_detections.push(current);
        sizemap[current.idx] += 1;
        detections.retain(|detection| current.iou(detection) < IOU_THRESHOLD);
    }
    filtered_detections
}
