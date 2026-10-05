use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::app::AppState;
use crate::core::types::{PxPoint, PxRect, RelPoint, WindowInfo};

const HUD_W: f64 = 340.0;
const HUD_H: f64 = 56.0;
const MARGIN: f64 = 8.0;
const PANEL_MIN_W: f64 = 380.0;
const PANEL_MIN_H: f64 = 420.0;
const GUIDE_W: f64 = 680.0;
const GUIDE_H: f64 = 560.0;

static GUIDE_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PANEL_SUPPRESSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PANEL_WAS_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn suppressed() -> bool {
    PANEL_SUPPRESSED.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn on_bot_state(app: &AppHandle, active: bool) {
    use std::sync::atomic::Ordering::SeqCst;
    if active {
        if !PANEL_SUPPRESSED.swap(true, SeqCst) {
            PANEL_WAS_OPEN.store(panel_visible(app), SeqCst);
            hide(panel(app));
            hide(guide(app));
            emit_panel_visible(app);
        }
    } else if PANEL_SUPPRESSED.swap(false, SeqCst) && PANEL_WAS_OPEN.swap(false, SeqCst) {
        reposition(app);
    }
}

pub fn init(app: &AppHandle) {
    for w in [hud(app), panel(app), guide(app), overlay(app)].into_iter().flatten() {
        undecorate(&w);
    }
    for w in [hud(app), panel(app), guide(app), overlay(app)].into_iter().flatten() {
        let _ = w.set_shadow(false);
    }
    if let Some(hud) = hud(app) {
        let _ = hud.set_size(LogicalSize::new(HUD_W, HUD_H));
        set_noactivate(&hud);
    }
    if let Some(panel) = panel(app) {
        let [pw, ph] = panel_size(app);
        let _ = panel.set_size(LogicalSize::new(pw, ph));
    }
    let info = *app.state::<AppState>().roblox.read();
    on_roblox_changed(app, info);
}

fn hud(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("hud")
}

fn panel(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("panel")
}

fn overlay(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("overlay")
}

fn guide(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("guide")
}

fn roblox_active(info: Option<WindowInfo>) -> Option<PxRect> {
    match info {
        Some(w) if w.visible && w.is_foreground => Some(w.client),
        _ => None,
    }
}

pub fn on_roblox_changed(app: &AppHandle, info: Option<WindowInfo>) {
    match info {
        None => {
            hide(hud(app));
            if overlay_visible(app) {
                hide_overlay(app);
            }
            if let Some(p) = panel(app) {
                if !p.is_visible().unwrap_or(false) && !p.is_minimized().unwrap_or(false) {
                    show_noactivate(&p);
                }
            }
            if GUIDE_OPEN.load(std::sync::atomic::Ordering::SeqCst) {
                if let Some(g) = guide(app) {
                    if !g.is_visible().unwrap_or(false) {
                        show_noactivate(&g);
                    }
                }
            }
        }
        Some(w) if !w.visible || !w.is_foreground => {
            hide(hud(app));
            if overlay_visible(app) {
                hide_overlay(app);
            }
            hide(panel(app));
            hide(guide(app));
        }
        Some(w) => {
            position_hud(app, w.client);
            position_panel(app, w.client);
            if GUIDE_OPEN.load(std::sync::atomic::Ordering::SeqCst) && !suppressed() {
                position_guide(app, w.client);
            }
            if let Some(o) = overlay(app) {
                if o.is_visible().unwrap_or(false) {
                    fit_overlay(&o, w.client);
                    let _ = o.emit("overlay:roblox", w.client);
                }
            }
        }
    }
    let _ = app.emit("ui:visibility", roblox_active(info).is_some() || info.is_none());
    emit_panel_visible(app);
}

fn hide(w: Option<WebviewWindow>) {
    if let Some(w) = w {
        if w.is_visible().unwrap_or(false) {
            hide_window(&w);
        }
    }
}

#[cfg(windows)]
fn set_noactivate(w: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    };
    if let Ok(h) = w.hwnd() {
        unsafe {
            let hwnd = HWND(h.0 as *mut _);
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE.0 as isize);
        }
    }
}

#[cfg(not(windows))]
fn set_noactivate(_w: &WebviewWindow) {}

fn undecorate(w: &WebviewWindow) {
    if w.is_decorated().unwrap_or(false) {
        let _ = w.set_decorations(false);
    }
}

#[cfg(windows)]
fn show_noactivate(w: &WebviewWindow) {
    undecorate(w);
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };
    match w.hwnd() {
        Ok(h) => unsafe {
            let _ = SetWindowPos(
                HWND(h.0 as *mut _),
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        },
        Err(_) => {
            let _ = w.show();
        }
    }
}

#[cfg(not(windows))]
fn show_noactivate(w: &WebviewWindow) {
    undecorate(w);
    let _ = w.show();
}

#[cfg(windows)]
fn hide_window(w: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    match w.hwnd() {
        Ok(h) => unsafe {
            let _ = ShowWindow(HWND(h.0 as *mut _), SW_HIDE);
        },
        Err(_) => {
            let _ = w.hide();
        }
    }
}

#[cfg(not(windows))]
fn hide_window(w: &WebviewWindow) {
    let _ = w.hide();
}

pub fn reposition(app: &AppHandle) {
    let info = *app.state::<AppState>().roblox.read();
    on_roblox_changed(app, info);
}

fn position_hud(app: &AppHandle, client: PxRect) {
    let Some(hud) = hud(app) else { return };
    let st = app.state::<AppState>();
    let (visible, offset) = {
        let s = st.settings.read();
        (s.ui.hud_visible, s.ui.hud_offset)
    };
    if !visible {
        hide(Some(hud));
        return;
    }
    let scale = hud.scale_factor().unwrap_or(1.0);
    let w = (HUD_W * scale) as i32;
    let h = (HUD_H * scale) as i32;
    let margin = (MARGIN * scale) as i32;
    let x = client.x + (offset.x * client.w as f32) as i32 - w / 2;
    let y = client.y + (offset.y * client.h as f32) as i32 + margin;
    let x = x.clamp(client.x + margin, (client.right() - w - margin).max(client.x));
    let y = y.clamp(client.y + margin, (client.bottom() - h - margin).max(client.y));
    let _ = hud.set_position(PhysicalPosition::new(x, y));
    if !hud.is_visible().unwrap_or(false) {
        show_noactivate(&hud);
    }
}

fn panel_size(app: &AppHandle) -> [f64; 2] {
    let size = app.state::<AppState>().settings.read().ui.panel_size;
    [(size[0] as f64).max(PANEL_MIN_W), (size[1] as f64).max(PANEL_MIN_H)]
}

fn position_panel(app: &AppHandle, client: PxRect) {
    let Some(panel) = panel(app) else { return };
    if suppressed() {
        hide(Some(panel));
        return;
    }
    let st = app.state::<AppState>();
    let offset = st.settings.read().ui.panel_offset;
    let size = panel_size(app);
    let scale = panel.scale_factor().unwrap_or(1.0);
    let margin = (MARGIN * scale) as i32;
    let max_w = (client.w - margin * 2).max((PANEL_MIN_W * scale) as i32);
    let max_h = (client.h - margin * 2).max((PANEL_MIN_H * scale) as i32);
    let w = ((size[0] * scale) as i32).min(max_w);
    let h = ((size[1] * scale) as i32).min(max_h);
    let _ = panel.set_size(PhysicalSize::new(w as u32, h as u32));
    let x = client.x + (offset.x * client.w as f32) as i32 - w;
    let y = client.y + (offset.y * client.h as f32) as i32 - h / 2;
    let x = x.clamp(client.x + margin, (client.right() - w - margin).max(client.x + margin));
    let y = y.clamp(client.y + margin, (client.bottom() - h - margin).max(client.y + margin));
    let _ = panel.set_position(PhysicalPosition::new(x, y));
    if !panel.is_visible().unwrap_or(false) && !panel.is_minimized().unwrap_or(false) {
        show_noactivate(&panel);
    }
}

fn position_guide(app: &AppHandle, client: PxRect) {
    let Some(g) = guide(app) else { return };
    let scale = g.scale_factor().unwrap_or(1.0);
    let margin = (MARGIN * scale) as i32;
    let w = ((GUIDE_W * scale) as i32).min((client.w - margin * 2).max(1));
    let h = ((GUIDE_H * scale) as i32).min((client.h - margin * 2).max(1));
    let _ = g.set_size(PhysicalSize::new(w as u32, h as u32));
    let x = client.x + (client.w - w) / 2;
    let y = client.y + (client.h - h) / 2;
    let _ = g.set_position(PhysicalPosition::new(x, y));
    if !g.is_visible().unwrap_or(false) {
        show_noactivate(&g);
    }
}

pub fn save_panel_placement(app: &AppHandle) {
    let Some(panel) = panel(app) else { return };
    let st = app.state::<AppState>();
    let Some(client) = st.roblox.read().map(|w| w.client) else { return };
    if !panel.is_visible().unwrap_or(false) || panel.is_minimized().unwrap_or(false) {
        return;
    }
    let (Ok(pos), Ok(size)) = (panel.outer_position(), panel.inner_size()) else { return };
    let scale = panel.scale_factor().unwrap_or(1.0);
    if (size.width as f64 / scale) < PANEL_MIN_W - 1.0 || (size.height as f64 / scale) < PANEL_MIN_H - 1.0 {
        return;
    }
    let mut s = st.settings.read().clone();
    s.ui.panel_offset = RelPoint {
        x: ((pos.x + size.width as i32 - client.x) as f32 / client.w.max(1) as f32).clamp(0.0, 1.0),
        y: ((pos.y + size.height as i32 / 2 - client.y) as f32 / client.h.max(1) as f32).clamp(0.0, 1.0),
    };
    s.ui.panel_size = [(size.width as f64 / scale) as u32, (size.height as f64 / scale) as u32];
    *st.settings.write() = s.clone();
    let _ = st.store.save(&s);
}

pub fn set_hud_visible(app: &AppHandle, visible: bool) {
    if visible {
        reposition(app);
    } else {
        hide(hud(app));
    }
}

pub fn panel_visible(app: &AppHandle) -> bool {
    panel(app).map(|p| p.is_visible().unwrap_or(false) && !p.is_minimized().unwrap_or(false)).unwrap_or(false)
}

pub fn emit_panel_visible(app: &AppHandle) {
    let _ = app.emit("panel:visible", panel_visible(app));
}

pub fn hide_panel(app: &AppHandle) {
    hide(panel(app));
    emit_panel_visible(app);
}

pub fn toggle_panel(app: &AppHandle) {
    if panel_visible(app) {
        hide_panel(app);
    } else {
        show_panel(app);
    }
}

pub fn show_panel(app: &AppHandle) {
    let Some(p) = panel(app) else { return };
    PANEL_SUPPRESSED.store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = p.unminimize();
    let info = *app.state::<AppState>().roblox.read();
    if let Some(client) = roblox_active(info) {
        position_panel(app, client);
    } else {
        let [pw, ph] = panel_size(app);
        let _ = p.set_size(LogicalSize::new(pw, ph));
    }
    undecorate(&p);
    let _ = p.show();
    let _ = p.set_focus();
    emit_panel_visible(app);
}

pub fn show_guide(app: &AppHandle) -> Result<(), String> {
    let g = guide(app).ok_or("guide window missing")?;
    GUIDE_OPEN.store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = g.unminimize();
    let info = *app.state::<AppState>().roblox.read();
    match roblox_active(info) {
        Some(client) => position_guide(app, client),
        None => {
            let _ = g.center();
        }
    }
    undecorate(&g);
    g.show().map_err(|e| e.to_string())?;
    let _ = g.set_focus();
    let _ = g.emit("guide:open", ());
    Ok(())
}

pub fn hide_guide(app: &AppHandle) {
    GUIDE_OPEN.store(false, std::sync::atomic::Ordering::SeqCst);
    hide(guide(app));
}

fn fit_overlay(o: &WebviewWindow, client: PxRect) {
    let _ = o.set_position(PhysicalPosition::new(client.x, client.y));
    let _ = o.set_size(PhysicalSize::new(client.w as u32, client.h as u32));
}

pub fn show_overlay(app: &AppHandle, roblox: PxRect, interactive: bool) -> Result<PxPoint, String> {
    let o = overlay(app).ok_or("overlay window missing")?;
    let st = app.state::<AppState>();
    if interactive && st.bot.is_running() {
        st.bot.stop();
    }
    undecorate(&o);
    fit_overlay(&o, roblox);
    o.set_ignore_cursor_events(!interactive).map_err(|e| e.to_string())?;
    o.show().map_err(|e| e.to_string())?;
    let _ = o.set_always_on_top(true);
    if interactive {
        o.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(PxPoint { x: roblox.x, y: roblox.y })
}

pub fn hide_overlay(app: &AppHandle) {
    let st = app.state::<AppState>();
    *st.overlay_session.lock() = None;
    if let Some(o) = overlay(app) {
        let _ = o.emit("overlay:close", ());
        let _ = o.set_ignore_cursor_events(true);
        let _ = o.hide();
    }
}

pub fn overlay_visible(app: &AppHandle) -> bool {
    overlay(app).map(|o| o.is_visible().unwrap_or(false)).unwrap_or(false)
}
