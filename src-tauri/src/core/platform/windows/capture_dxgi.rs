use parking_lot::Mutex;
use windows::core::Interface;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
    D3D11_CREATE_DEVICE_FLAG, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter, IDXGIFactory1, IDXGIOutput1, IDXGIOutputDuplication,
    IDXGIResource, DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_DESC, DXGI_OUTDUPL_FRAME_INFO,
};

use crate::core::platform::{Capture, PlatformError, Result};
use crate::core::types::{Frame, PxRect};

use super::capture::GdiCapture;

const FIRST_FRAME_TIMEOUT_MS: u32 = 250;

struct Session {
    context: ID3D11DeviceContext,
    dup: IDXGIOutputDuplication,
    staging: ID3D11Texture2D,
    origin: (i32, i32),
    size: (i32, i32),
    has_frame: bool,
    frames: u32,
    last_new: std::time::Instant,
}

const STALE_AFTER: std::time::Duration = std::time::Duration::from_millis(1200);


unsafe impl Send for Session {}

pub struct DxgiCapture {
    session: Mutex<Option<Session>>,
    fallback: GdiCapture,
}

impl DxgiCapture {
    pub fn new() -> Self {
        Self { session: Mutex::new(None), fallback: GdiCapture::new() }
    }

    fn open(rect: PxRect) -> std::result::Result<Session, String> {
        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1().map_err(|e| e.to_string())?;
            let mut adapter_index = 0;
            while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
                adapter_index += 1;
                let mut output_index = 0;
                while let Ok(output) = adapter.EnumOutputs(output_index) {
                    output_index += 1;
                    let Ok(desc) = output.GetDesc() else { continue };
                    let dc = desc.DesktopCoordinates;
                    if rect.x < dc.left || rect.y < dc.top || rect.x >= dc.right || rect.y >= dc.bottom {
                        continue;
                    }
                    let base: IDXGIAdapter = adapter.cast().map_err(|e| e.to_string())?;
                    let mut device: Option<ID3D11Device> = None;
                    let mut context: Option<ID3D11DeviceContext> = None;
                    D3D11CreateDevice(
                        Some(&base),
                        D3D_DRIVER_TYPE_UNKNOWN,
                        HMODULE::default(),
                        D3D11_CREATE_DEVICE_FLAG(0),
                        None,
                        D3D11_SDK_VERSION,
                        Some(&mut device),
                        None,
                        Some(&mut context),
                    )
                    .map_err(|e| e.to_string())?;
                    let (Some(device), Some(context)) = (device, context) else {
                        return Err("D3D11CreateDevice returned nothing".into());
                    };
                    let output1: IDXGIOutput1 = output.cast().map_err(|e| e.to_string())?;
                    let dup = output1.DuplicateOutput(&device).map_err(|e| e.to_string())?;
                    let dd: DXGI_OUTDUPL_DESC = dup.GetDesc();
                    let (w, h) = (dd.ModeDesc.Width as i32, dd.ModeDesc.Height as i32);
                    let tex_desc = D3D11_TEXTURE2D_DESC {
                        Width: w as u32,
                        Height: h as u32,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                        Usage: D3D11_USAGE_STAGING,
                        BindFlags: 0,
                        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                        MiscFlags: 0,
                    };
                    let mut staging: Option<ID3D11Texture2D> = None;
                    device.CreateTexture2D(&tex_desc, None, Some(&mut staging)).map_err(|e| e.to_string())?;
                    let Some(staging) = staging else {
                        return Err("CreateTexture2D returned nothing".into());
                    };
                    return Ok(Session { context, dup, staging, origin: (dc.left, dc.top), size: (w, h), has_frame: false, frames: 0, last_new: std::time::Instant::now() });
                }
            }
            Err("no output contains the capture rect".into())
        }
    }

    fn grab_dxgi(&self, rect: PxRect) -> std::result::Result<Frame, String> {
        let mut guard = self.session.lock();
        let needs_open = match guard.as_ref() {
            Some(s) => {
                rect.x < s.origin.0
                    || rect.y < s.origin.1
                    || rect.x + rect.w > s.origin.0 + s.size.0
                    || rect.y + rect.h > s.origin.1 + s.size.1
            }
            None => true,
        };
        if needs_open {
            *guard = Some(Self::open(rect)?);
        }
        let s = guard.as_mut().expect("session");
        let result = unsafe { Self::read(s, rect) };
        if let Err(e) = &result {
            if !(e.starts_with("first desktop frame") || e.starts_with("no desktop frame yet")) {
                *guard = None;
            }
        }
        result
    }

    unsafe fn read(s: &mut Session, rect: PxRect) -> std::result::Result<Frame, String> {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        let timeout = if s.has_frame { 0 } else { FIRST_FRAME_TIMEOUT_MS };
        match s.dup.AcquireNextFrame(timeout, &mut info, &mut resource) {
            Ok(()) => {
                let copied = resource
                    .as_ref()
                    .ok_or_else(|| "AcquireNextFrame returned no resource".to_string())
                    .and_then(|r| r.cast::<ID3D11Texture2D>().map_err(|e| e.to_string()))
                    .map(|tex| s.context.CopyResource(&s.staging, &tex));
                let _ = s.dup.ReleaseFrame();
                copied?;
                s.frames += 1;
                s.last_new = std::time::Instant::now();
                if s.frames < 2 {
                    return Err("first desktop frame skipped".into());
                }
                s.has_frame = true;
            }
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {
                if !s.has_frame {
                    return Err("no desktop frame yet".into());
                }
                if s.last_new.elapsed() > STALE_AFTER {
                    return Err("desktop frames stopped arriving (stale image)".into());
                }
            }
            Err(e) => return Err(e.to_string()),
        }

        let lx = rect.x - s.origin.0;
        let ly = rect.y - s.origin.1;
        let w = rect.w as usize;
        let h = rect.h as usize;
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        s.context.Map(&s.staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).map_err(|e| e.to_string())?;
        let pitch = mapped.RowPitch as usize;
        let base = mapped.pData as *const u8;
        let mut rgba = vec![0u8; w * h * 4];
        for row in 0..h {
            let src = base.add((ly as usize + row) * pitch + lx as usize * 4);
            let src_row = std::slice::from_raw_parts(src, w * 4);
            let dst_row = &mut rgba[row * w * 4..(row + 1) * w * 4];
            for (d, sp) in dst_row.chunks_exact_mut(4).zip(src_row.chunks_exact(4)) {
                d[0] = sp[2];
                d[1] = sp[1];
                d[2] = sp[0];
                d[3] = 255;
            }
        }
        s.context.Unmap(&s.staging, 0);
        Ok(Frame::new(w, h, rgba))
    }
}

impl Default for DxgiCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture for DxgiCapture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        if rect.is_empty() {
            return Err(PlatformError::Capture("empty rect".into()));
        }
        match self.grab_dxgi(rect) {
            Ok(f) => Ok(f),
            Err(e) => {
                if e.contains("stale") {
                    *self.session.lock() = None;
                }
                self.fallback.grab(rect)
            }
        }
    }
}

