use std::sync::OnceLock;

use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

use crate::core::platform::{Ocr, PlatformError, Result};
use crate::core::types::Frame;

thread_local! {
    static APARTMENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn ensure_apartment() {
    APARTMENT.with(|done| {
        if !done.get() {
            unsafe {
                let _ = RoInitialize(RO_INIT_MULTITHREADED);
            }
            done.set(true);
        }
    });
}

#[derive(Default)]
pub struct WindowsOcr {
    engine: OnceLock<Option<OcrEngine>>,
}

unsafe impl Send for WindowsOcr {}
unsafe impl Sync for WindowsOcr {}

impl WindowsOcr {
    pub fn new() -> Self {
        Self::default()
    }

    fn engine(&self) -> Option<&OcrEngine> {
        self.engine
            .get_or_init(|| {
                ensure_apartment();
                let engine = OcrEngine::TryCreateFromUserProfileLanguages().ok().or_else(|| {
                    let lang = Language::CreateLanguage(&HSTRING::from("en-US")).ok()?;
                    OcrEngine::TryCreateFromLanguage(&lang).ok()
                });
                if engine.is_none() {
                    tracing::warn!("Windows OCR engine unavailable; fruit detection disabled");
                }
                engine
            })
            .as_ref()
    }
}

impl Ocr for WindowsOcr {
    fn available(&self) -> bool {
        self.engine().is_some()
    }

    fn read(&self, frame: &Frame) -> Result<String> {
        let engine = self.engine().ok_or(PlatformError::OcrUnavailable)?;
        ensure_apartment();
        if frame.w < 8 || frame.h < 8 {
            return Ok(String::new());
        }
        let mut bgra = frame.rgba.clone();
        for px in bgra.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        let err = |e: windows::core::Error| PlatformError::Ocr(e.message());
        let writer = DataWriter::new().map_err(err)?;
        writer.WriteBytes(&bgra).map_err(err)?;
        let buffer = writer.DetachBuffer().map_err(err)?;
        let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
            &buffer,
            BitmapPixelFormat::Bgra8,
            frame.w as i32,
            frame.h as i32,
            BitmapAlphaMode::Ignore,
        )
        .map_err(err)?;
        let result = engine.RecognizeAsync(&bitmap).map_err(err)?.join().map_err(err)?;
        let text = result.Text().map_err(err)?;
        Ok(text.to_string())
    }
}
