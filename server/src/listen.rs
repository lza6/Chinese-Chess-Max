use serde::Deserialize;
use serde::Serialize;
use xcap::image;
use xcap::image::GenericImage;

#[derive(Serialize, Deserialize, Debug)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub app_name: String,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(win: &xcap::Window) -> Result<Self, String> {
        let id = win.id().map_err(|e| format!("读取窗口 id 失败: {e}"))?;
        let title = win.title().map_err(|e| format!("读取窗口标题失败: {e}"))?;
        let app_name = win
            .app_name()
            .map_err(|e| format!("读取窗口 app_name 失败: {e}"))?;
        let width = win.width().map_err(|e| format!("读取窗口宽度失败: {e}"))?;
        let height = win.height().map_err(|e| format!("读取窗口高度失败: {e}"))?;
        Ok(Self {
            id,
            title,
            app_name,
            width,
            height,
        })
    }
}

#[tauri::command]
pub async fn list_windows() -> Result<Vec<Window>, String> {
    let windows = xcap::Window::all().map_err(|e| format!("枚举窗口失败: {e}"))?;
    if windows.is_empty() {
        return Err("未找到任何可截屏窗口".to_string());
    }
    let mut result = vec![];
    for window in windows.iter() {
        match Window::new(window) {
            Ok(w) => result.push(w),
            Err(e) => tracing::debug!("跳过不可读窗口: {e}"),
        }
    }
    if result.is_empty() {
        return Err("窗口信息读取失败，无可用窗口".to_string());
    }
    Ok(result)
}

pub struct ListenWindow {
    window: xcap::Window,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl ListenWindow {
    #[tracing::instrument]
    pub fn new(target: &Window, _w: usize, _h: usize) -> Result<Self, String> {
        let windows = xcap::Window::all().map_err(|e| format!("枚举窗口失败: {e}"))?;
        for window in windows {
            if window.id().map_err(|e| format!("读取窗口 id 失败: {e}"))? == target.id {
                return Ok(Self {
                    window,
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                });
            }
        }
        Err(format!(
            "目标窗口已不存在或不可用（id={}），请重新选择",
            target.id
        ))
    }

    pub fn capture(&self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, String> {
        let mut pic = self
            .window
            .capture_image()
            .map_err(|e| format!("窗口截屏失败: {e}"))?;
        if self.w > 0 {
            pic = pic.sub_image(self.x, self.y, self.w, self.h).to_image();
        }
        Ok(pic)
    }

    pub fn set_sub_bound(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
    }
}
