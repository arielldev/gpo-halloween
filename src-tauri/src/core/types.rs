use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PxPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PxRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl PxRect {
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub fn is_empty(&self) -> bool {
        self.w <= 0 || self.h <= 0
    }
    pub fn contains(&self, p: PxPoint) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.right() && p.y < self.bottom()
    }
    pub fn center(&self) -> PxPoint {
        PxPoint { x: self.x + self.w / 2, y: self.y + self.h / 2 }
    }
}

impl PxPoint {
    pub fn to_rel(&self, base: &PxRect) -> RelPoint {
        RelPoint {
            x: (self.x - base.x) as f32 / base.w.max(1) as f32,
            y: (self.y - base.y) as f32 / base.h.max(1) as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct RelPoint {
    pub x: f32,
    pub y: f32,
}

impl RelPoint {
    pub fn to_px(&self, base: &PxRect) -> PxPoint {
        PxPoint {
            x: base.x + (self.x * base.w as f32).round() as i32,
            y: base.y + (self.y * base.h as f32).round() as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct RelRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RelRect {
    pub fn to_px(&self, base: &PxRect) -> PxRect {
        PxRect {
            x: base.x + (self.x * base.w as f32).round() as i32,
            y: base.y + (self.y * base.h as f32).round() as i32,
            w: (self.w * base.w as f32).round().max(1.0) as i32,
            h: (self.h * base.h as f32).round().max(1.0) as i32,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

impl Frame {
    pub fn new(w: usize, h: usize, rgba: Vec<u8>) -> Self {
        debug_assert_eq!(rgba.len(), w * h * 4);
        Self { w, h, rgba }
    }

    pub fn downscale(&self, max_dim: usize) -> Frame {
        let longest = self.w.max(self.h);
        if longest <= max_dim || longest == 0 {
            return self.clone();
        }
        let factor = longest.div_ceil(max_dim);
        let nw = (self.w / factor).max(1);
        let nh = (self.h / factor).max(1);
        let mut out = Vec::with_capacity(nw * nh * 4);
        for y in 0..nh {
            let sy = y * factor;
            for x in 0..nw {
                let i = (sy * self.w + x * factor) * 4;
                out.extend_from_slice(&self.rgba[i..i + 4]);
            }
        }
        Frame { w: nw, h: nh, rgba: out }
    }

    pub fn upscale(&self, factor: usize) -> Frame {
        let f = factor.max(1);
        if f == 1 {
            return self.clone();
        }
        let nw = self.w * f;
        let nh = self.h * f;
        let mut out = Vec::with_capacity(nw * nh * 4);
        for y in 0..nh {
            let sy = y / f;
            for x in 0..nw {
                let i = (sy * self.w + x / f) * 4;
                out.extend_from_slice(&self.rgba[i..i + 4]);
            }
        }
        Frame { w: nw, h: nh, rgba: out }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub client: PxRect,
    pub is_foreground: bool,
    pub visible: bool,
    pub dpi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
    Space,
    Shift,
    Control,
}

impl Key {
    pub fn parse(s: &str) -> Option<Key> {
        let mut chars = s.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return Some(if c == ' ' { Key::Space } else { Key::Char(c.to_ascii_lowercase()) });
        }
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "backspace" => Key::Backspace,
            "delete" | "del" => Key::Delete,
            "enter" | "return" => Key::Enter,
            "escape" | "esc" => Key::Escape,
            "tab" => Key::Tab,
            "space" => Key::Space,
            "shift" => Key::Shift,
            "ctrl" | "control" => Key::Control,
            _ => return None,
        })
    }

    pub fn name(&self) -> String {
        match self {
            Key::Char(c) => c.to_string(),
            Key::Backspace => "Backspace".into(),
            Key::Delete => "Delete".into(),
            Key::Enter => "Enter".into(),
            Key::Escape => "Escape".into(),
            Key::Tab => "Tab".into(),
            Key::Space => "Space".into(),
            Key::Shift => "Shift".into(),
            Key::Control => "Ctrl".into(),
        }
    }

    pub fn recordable() -> Vec<Key> {
        let mut v: Vec<Key> = ('a'..='z').chain('0'..='9').map(Key::Char).collect();
        v.extend([Key::Space, Key::Enter, Key::Escape, Key::Tab, Key::Backspace]);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_round_trip() {
        for k in Key::recordable() {
            assert_eq!(Key::parse(&k.name()), Some(k));
        }
        assert_eq!(Key::parse("E"), Some(Key::Char('e')));
        assert_eq!(Key::parse("esc"), Some(Key::Escape));
        assert_eq!(Key::parse("nope"), None);
    }

    #[test]
    fn rel_px_round_trip() {
        let base = PxRect { x: 100, y: 50, w: 1920, h: 1080 };
        let p = PxPoint { x: 1060, y: 590 };
        assert_eq!(p.to_rel(&base).to_px(&base), p);
    }
}
