use std::sync::Arc;

use crate::core::types::{Frame, Key, MouseButton, PxPoint, PxRect, WindowFix, WindowInfo};

pub mod mock;
#[cfg(windows)]
pub mod windows;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("game window not found")]
    WindowNotFound,
    #[error("capture failed: {0}")]
    Capture(String),
    #[error("ocr unavailable")]
    OcrUnavailable,
    #[error("ocr failed: {0}")]
    Ocr(String),
}

pub type Result<T> = std::result::Result<T, PlatformError>;

pub trait GameWindow: Send + Sync {
    fn find(&self) -> Option<WindowInfo>;
    fn focus(&self) -> bool;
    fn game_foreground(&self) -> bool;
    fn owns_point(&self, p: PxPoint) -> bool;
    fn normalize(&self) -> WindowFix;
}

pub trait Capture: Send + Sync {
    fn grab(&self, rect: PxRect) -> Result<Frame>;
}

pub trait Input: Send + Sync {
    fn move_to(&self, p: PxPoint);
    fn move_rel(&self, dx: i32, dy: i32);
    fn button(&self, b: MouseButton, down: bool);
    fn key(&self, k: Key, down: bool);
    fn wheel(&self, delta: i32);
    fn type_text(&self, s: &str);
    fn cursor(&self) -> PxPoint;
    fn screen(&self) -> PxRect;
    fn key_down(&self, k: Key) -> bool;
    fn button_down(&self, b: MouseButton) -> bool;
    fn set_clipboard(&self, text: &str) -> bool;
}

pub trait Ocr: Send + Sync {
    fn available(&self) -> bool;
    fn read(&self, frame: &Frame) -> Result<String>;
}

#[derive(Clone)]
pub struct Platform {
    pub window: Arc<dyn GameWindow>,
    pub capture: Arc<dyn Capture>,
    pub capture_live: Arc<dyn Capture>,
    pub input: Arc<dyn Input>,
    pub ocr: Arc<dyn Ocr>,
}

#[cfg(windows)]
pub fn build() -> Platform {
    Platform {
        window: Arc::new(windows::window::Win32Window::new("Roblox")),
        capture: Arc::new(windows::capture_dxgi::DxgiCapture::new()),
        capture_live: Arc::new(windows::capture::GdiCapture::new()),
        input: Arc::new(windows::input::SendInputBackend::new()),
        ocr: build_ocr(),
    }
}

#[cfg(all(windows, feature = "ocr-windows", not(feature = "ocr-none")))]
fn build_ocr() -> Arc<dyn Ocr> {
    Arc::new(windows::ocr::WindowsOcr::new())
}

#[cfg(all(windows, any(feature = "ocr-none", not(feature = "ocr-windows"))))]
fn build_ocr() -> Arc<dyn Ocr> {
    Arc::new(mock::NoOcr)
}

#[cfg(not(windows))]
pub fn build() -> Platform {
    mock::platform()
}
