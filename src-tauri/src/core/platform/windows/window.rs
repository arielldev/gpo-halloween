use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetAncestor, GetClientRect, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow,
    IsWindowVisible, SetForegroundWindow, ShowWindow, WindowFromPoint, GA_ROOT, SW_RESTORE,
};

use crate::core::platform::GameWindow;
use crate::core::types::{PxPoint, PxRect, WindowInfo};

pub struct Win32Window {
    class: Vec<u16>,
    title: Vec<u16>,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

impl Win32Window {
    pub fn new(title: &str) -> Self {
        Self { class: wide("WINDOWSCLIENT"), title: wide(title) }
    }

    fn hwnd(&self) -> Option<HWND> {
        unsafe {
            let by_class = FindWindowW(PCWSTR(self.class.as_ptr()), PCWSTR(self.title.as_ptr())).ok();
            let h = match by_class {
                Some(h) if !h.is_invalid() => h,
                _ => FindWindowW(PCWSTR::null(), PCWSTR(self.title.as_ptr())).ok()?,
            };
            if h.is_invalid() || !IsWindow(Some(h)).as_bool() {
                return None;
            }
            Some(h)
        }
    }
}

fn alt_tap() {
    let mk = |up: bool| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_MENU,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[mk(false), mk(true)], std::mem::size_of::<INPUT>() as i32);
    }
}

impl GameWindow for Win32Window {
    fn focus(&self) -> bool {
        let Some(h) = self.hwnd() else { return false };
        unsafe {
            if GetForegroundWindow() == h {
                return true;
            }
            if IsIconic(h).as_bool() {
                let _ = ShowWindow(h, SW_RESTORE);
            }
            if SetForegroundWindow(h).as_bool() {
                return true;
            }
            alt_tap();
            SetForegroundWindow(h).as_bool()
        }
    }

    fn game_foreground(&self) -> bool {
        let Some(h) = self.hwnd() else { return false };
        unsafe { GetForegroundWindow() == h }
    }

    fn owns_point(&self, p: PxPoint) -> bool {
        let Some(h) = self.hwnd() else { return false };
        unsafe {
            let under = WindowFromPoint(POINT { x: p.x, y: p.y });
            if under.is_invalid() {
                return false;
            }
            under == h || GetAncestor(under, GA_ROOT) == h
        }
    }

    fn find(&self) -> Option<WindowInfo> {
        let h = self.hwnd()?;
        unsafe {
            if !IsWindowVisible(h).as_bool() {
                return None;
            }
            let minimized = IsIconic(h).as_bool();
            let mut rc = RECT::default();
            GetClientRect(h, &mut rc).ok()?;
            let mut origin = POINT { x: rc.left, y: rc.top };
            if !ClientToScreen(h, &mut origin).as_bool() {
                return None;
            }
            let client = PxRect { x: origin.x, y: origin.y, w: rc.right - rc.left, h: rc.bottom - rc.top };
            if client.is_empty() && !minimized {
                return None;
            }
            let fg = GetForegroundWindow();
            let mut fg_pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            let ours = fg_pid == GetCurrentProcessId();
            let dpi = GetDpiForWindow(h);
            Some(WindowInfo {
                client,
                is_foreground: fg == h || ours,
                visible: !minimized,
                dpi: if dpi == 0 { 96 } else { dpi },
            })
        }
    }
}
