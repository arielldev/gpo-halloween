use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use parking_lot::Mutex;

use super::{Capture, GameWindow, Input, Ocr, Platform, PlatformError, Result};
use crate::core::types::{Frame, Key, MouseButton, PxPoint, PxRect, WindowInfo};

#[derive(Default)]
pub struct MockWindow {
    pub info: Mutex<Option<WindowInfo>>,
}

impl GameWindow for MockWindow {
    fn find(&self) -> Option<WindowInfo> {
        *self.info.lock()
    }
    fn focus(&self) -> bool {
        true
    }
    fn game_foreground(&self) -> bool {
        self.info.lock().is_some()
    }
    fn owns_point(&self, p: PxPoint) -> bool {
        self.info.lock().map(|w| w.client.contains(p)).unwrap_or(false)
    }
}

#[derive(Default)]
pub struct ScriptedCapture {
    pub frames: Mutex<VecDeque<Frame>>,
}

impl Capture for ScriptedCapture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        if let Some(f) = self.frames.lock().pop_front() {
            return Ok(f);
        }
        let (w, h) = (rect.w.max(1) as usize, rect.h.max(1) as usize);
        Ok(Frame::new(w, h, vec![0; w * h * 4]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Move(PxPoint),
    MoveRel(i32, i32),
    Button(MouseButton, bool),
    Key(Key, bool),
    Wheel(i32),
    Text(String),
    Clipboard(String),
}

#[derive(Default)]
pub struct RecordingInput {
    pub events: Mutex<Vec<InputEvent>>,
    pub pos: Mutex<PxPoint>,
    pub held_keys: Mutex<HashSet<Key>>,
    pub held_buttons: Mutex<HashSet<u8>>,
}

impl Input for RecordingInput {
    fn move_to(&self, p: PxPoint) {
        *self.pos.lock() = p;
        self.events.lock().push(InputEvent::Move(p));
    }
    fn move_rel(&self, dx: i32, dy: i32) {
        self.events.lock().push(InputEvent::MoveRel(dx, dy));
    }
    fn button(&self, b: MouseButton, down: bool) {
        self.events.lock().push(InputEvent::Button(b, down));
    }
    fn key(&self, k: Key, down: bool) {
        self.events.lock().push(InputEvent::Key(k, down));
    }
    fn wheel(&self, delta: i32) {
        self.events.lock().push(InputEvent::Wheel(delta));
    }
    fn type_text(&self, s: &str) {
        self.events.lock().push(InputEvent::Text(s.to_string()));
    }
    fn cursor(&self) -> PxPoint {
        *self.pos.lock()
    }
    fn screen(&self) -> PxRect {
        PxRect { x: 0, y: 0, w: 1920, h: 1080 }
    }
    fn key_down(&self, k: Key) -> bool {
        self.held_keys.lock().contains(&k)
    }
    fn button_down(&self, b: MouseButton) -> bool {
        self.held_buttons.lock().contains(&(b as u8))
    }
    fn set_clipboard(&self, text: &str) -> bool {
        self.events.lock().push(InputEvent::Clipboard(text.to_string()));
        true
    }
}

pub struct NoOcr;

impl Ocr for NoOcr {
    fn available(&self) -> bool {
        false
    }
    fn read(&self, _frame: &Frame) -> Result<String> {
        Err(PlatformError::OcrUnavailable)
    }
}

#[derive(Default)]
pub struct ScriptedOcr {
    pub texts: Mutex<VecDeque<String>>,
}

impl Ocr for ScriptedOcr {
    fn available(&self) -> bool {
        true
    }
    fn read(&self, _frame: &Frame) -> Result<String> {
        Ok(self.texts.lock().pop_front().unwrap_or_default())
    }
}

pub fn platform() -> Platform {
    Platform {
        window: Arc::new(MockWindow::default()),
        capture: Arc::new(ScriptedCapture::default()),
        input: Arc::new(RecordingInput::default()),
        ocr: Arc::new(NoOcr),
    }
}
