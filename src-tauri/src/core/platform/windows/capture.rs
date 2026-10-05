use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

use crate::core::platform::{Capture, PlatformError, Result};
use crate::core::types::{Frame, PxRect};

pub struct GdiCapture;

impl GdiCapture {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GdiCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture for GdiCapture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        if rect.is_empty() {
            return Err(PlatformError::Capture("empty rect".into()));
        }
        let w = rect.w;
        let h = rect.h;
        unsafe {
            let screen = GetDC(Some(HWND::default()));
            if screen.is_invalid() {
                return Err(PlatformError::Capture("GetDC".into()));
            }
            let mem = CreateCompatibleDC(Some(screen));
            let bmp = CreateCompatibleBitmap(screen, w, h);
            let old = SelectObject(mem, bmp.into());
            let ok = BitBlt(mem, 0, 0, w, h, Some(screen), rect.x, rect.y, SRCCOPY).is_ok();

            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut buf = vec![0u8; (w * h * 4) as usize];
            let lines = GetDIBits(
                mem,
                bmp,
                0,
                h as u32,
                Some(buf.as_mut_ptr() as *mut _),
                &mut info,
                DIB_RGB_COLORS,
            );

            SelectObject(mem, old);
            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(mem);
            ReleaseDC(Some(HWND::default()), screen);

            if !ok || lines == 0 {
                return Err(PlatformError::Capture("BitBlt/GetDIBits".into()));
            }
            if buf.chunks_exact(4).all(|p| p[0] == 0 && p[1] == 0 && p[2] == 0) {
                return Err(PlatformError::Capture("screen capture came back all black".into()));
            }
            for px in buf.chunks_exact_mut(4) {
                px.swap(0, 2);
                px[3] = 255;
            }
            Ok(Frame::new(w as usize, h as usize, buf))
        }
    }
}

