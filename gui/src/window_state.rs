use gpui::{Bounds, Point, Size, WindowBounds, px};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Trạng thái cửa sổ - persist giữa các lần mở app.
#[derive(Debug, Clone)]
pub struct WindowState {
    // pub width: f32,
    // pub height: f32,
    // pub x: f32,
    // pub y: f32,
    pub window_bounds: WindowBounds,
}

/// Struct trung gian để serialize/deserialize WindowBounds.
/// WindowBounds không implement Serialize nên phải chuyển thủ công.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowBoundsData {
    mode: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl WindowState {
    /// Đường dẫn file lưu state.
    /// Debug: target/window_state.json
    /// Release: window_state.json (cùng thư mục executable)
    fn path() -> PathBuf {
        #[cfg(debug_assertions)]
        let path = std::env::current_dir()
            .unwrap()
            .join("target/window_state.json");
        #[cfg(not(debug_assertions))]
        let path = std::env::current_dir().unwrap().join("window_state.json");
        path
    }

    /// Load state từ file. Trả về None nếu file không tồn tại hoặc lỗi.
    pub fn load() -> Option<Self> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).ok()?;
        let data: WindowBoundsData = serde_json::from_str(&content).ok()?;
        let bounds = Bounds::new(
            Point::new(px(data.x), px(data.y)),
            Size::new(px(data.width), px(data.height)),
        );
        let window_bounds = match data.mode.as_str() {
            "maximized" => WindowBounds::Maximized(bounds),
            "fullscreen" => WindowBounds::Fullscreen(bounds),
            _ => WindowBounds::Windowed(bounds),
        };
        Some(Self { window_bounds })
    }

    /// Save state ra file.
    pub fn save(&self) {
        let data = match self.window_bounds {
            WindowBounds::Windowed(bounds) => WindowBoundsData {
                mode: "windowed".to_string(),
                x: bounds.origin.x.as_f32(),
                y: bounds.origin.y.as_f32(),
                width: bounds.size.width.as_f32(),
                height: bounds.size.height.as_f32(),
            },
            WindowBounds::Maximized(bounds) => WindowBoundsData {
                mode: "maximized".to_string(),
                x: bounds.origin.x.as_f32(),
                y: bounds.origin.y.as_f32(),
                width: bounds.size.width.as_f32(),
                height: bounds.size.height.as_f32(),
            },
            WindowBounds::Fullscreen(bounds) => WindowBoundsData {
                mode: "fullscreen".to_string(),
                x: bounds.origin.x.as_f32(),
                y: bounds.origin.y.as_f32(),
                width: bounds.size.width.as_f32(),
                height: bounds.size.height.as_f32(),
            },
        };
        let path = Self::path();
        if let Ok(content) = serde_json::to_string_pretty(&data) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, content);
        }
    }
}
