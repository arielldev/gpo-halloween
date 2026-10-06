use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::app::AppState;
use crate::bot::{timer_watch, Job};
use crate::config::{Action, SeqId, Settings, StartFrom};
use crate::core::types::{PxPoint, PxRect, RelPoint, RelRect, WindowInfo};
use crate::events::{BotState, Stats, TimerReading};
use crate::hotkeys;
use crate::windows;

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub state: BotState,
    pub job: Option<Job>,
    pub stats: Stats,
    pub roblox: Option<WindowInfo>,
    pub ocr_available: bool,
    pub timer: Option<TimerReading>,
    pub settings: Settings,
    pub version: String,
}

#[tauri::command]
pub fn snapshot(app: AppHandle, st: State<'_, AppState>) -> Snapshot {
    let ctx = st.bot.ctx();
    Snapshot {
        state: st.bot.state(),
        job: st.bot.job(),
        stats: ctx.session.lock().stats(),
        roblox: *st.roblox.read(),
        ocr_available: st.platform.ocr.available(),
        timer: ctx.last_timer.lock().clone(),
        settings: st.settings.read().clone(),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn bot_start(st: State<'_, AppState>) {
    let bot = Arc::clone(&st.bot);
    std::thread::spawn(move || {
        bot.start(Job::Cycle { from: StartFrom::First });
    });
}

#[tauri::command]
pub fn bot_stop(st: State<'_, AppState>) {
    let bot = Arc::clone(&st.bot);
    std::thread::spawn(move || bot.stop());
}

#[tauri::command]
pub fn bot_toggle(st: State<'_, AppState>) {
    let bot = Arc::clone(&st.bot);
    std::thread::spawn(move || bot.toggle());
}

#[tauri::command]
pub fn run_once(st: State<'_, AppState>, seq: SeqId, from: Option<usize>, only: Option<bool>) {
    let bot = Arc::clone(&st.bot);
    let (from, only) = (from.unwrap_or(0), only.unwrap_or(false));
    std::thread::spawn(move || bot.run_steps(seq, from, only));
}

#[tauri::command]
pub fn record_toggle(app: AppHandle, st: State<'_, AppState>, seq: Option<SeqId>) -> Result<(), String> {
    if let Some(seq) = seq {
        if !st.bot.is_running() && st.settings.read().recording.target != seq {
            let mut s = st.settings.read().clone();
            s.recording.target = seq;
            settings_set(app, st.clone(), s)?;
        }
    }
    let bot = Arc::clone(&st.bot);
    std::thread::spawn(move || bot.record_toggle(seq));
    Ok(())
}

#[tauri::command]
pub fn settings_get(st: State<'_, AppState>) -> Settings {
    st.settings.read().clone()
}

#[tauri::command]
pub fn settings_set(app: AppHandle, st: State<'_, AppState>, mut settings: Settings) -> Result<(), String> {
    let (hotkeys_changed, hud_changed) = {
        let cur = st.settings.read();
        settings.ui.panel_offset = cur.ui.panel_offset;
        settings.ui.panel_size = cur.ui.panel_size;
        (cur.hotkeys != settings.hotkeys, cur.ui.hud_visible != settings.ui.hud_visible || cur.ui.hud_offset != settings.ui.hud_offset)
    };
    *st.settings.write() = settings.clone();
    st.store.save(&settings).map_err(|e| e.to_string())?;
    let _ = app.emit("settings:changed", &settings);
    if hud_changed {
        windows::reposition(&app);
    }
    if hotkeys_changed {
        hotkeys::register(&app, &settings.hotkeys)?;
    }
    Ok(())
}

#[tauri::command]
pub fn settings_reset(app: AppHandle, st: State<'_, AppState>) -> Result<Settings, String> {
    let s = Settings::factory();
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn settings_json(st: State<'_, AppState>) -> Result<String, String> {
    serde_json::to_string_pretty(&*st.settings.read()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_export(st: State<'_, AppState>) -> Result<String, String> {
    let s = st.settings.read().clone();
    st.store.export(&s).map(|p| p.display().to_string()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_import(app: AppHandle, st: State<'_, AppState>, json: String) -> Result<Settings, String> {
    let s: Settings = serde_json::from_str(&json).map_err(|e| format!("Not a valid settings file: {e}"))?;
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn sequence_export(st: State<'_, AppState>, seq: SeqId) -> Result<String, String> {
    let steps = st.settings.read().steps(seq).clone();
    if steps.is_empty() {
        return Err(format!("{} has no steps to export", seq.label()));
    }
    st.store.export_sequence(seq, &steps).map(|p| p.display().to_string()).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct SequenceImport {
    pub settings: Settings,
    pub count: usize,
    pub from: Option<SeqId>,
}

#[tauri::command]
pub fn sequence_import(app: AppHandle, st: State<'_, AppState>, seq: SeqId, json: String, append: bool) -> Result<SequenceImport, String> {
    if st.bot.is_running() {
        return Err("Stop the macro before importing".into());
    }
    let (from, steps) = crate::config::parse_sequence(&json)?;
    let count = steps.len();
    let mut s = st.settings.read().clone();
    let target = s.steps_mut(seq);
    if append {
        target.extend(steps);
    } else {
        *target = steps;
    }
    settings_set(app, st, s.clone())?;
    Ok(SequenceImport { settings: s, count, from })
}

#[tauri::command]
pub fn sequences_dir(st: State<'_, AppState>) -> String {
    st.store.dir().join("sequences").display().to_string()
}

#[tauri::command]
pub fn preset_list(st: State<'_, AppState>) -> Vec<String> {
    st.store.list_presets()
}

#[tauri::command]
pub fn preset_save(st: State<'_, AppState>, name: String) -> Result<(), String> {
    st.store.save_preset(&name, &st.settings.read()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preset_load(app: AppHandle, st: State<'_, AppState>, name: String) -> Result<Settings, String> {
    let s = st.store.load_preset(&name).map_err(|e| e.to_string())?;
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn preset_delete(st: State<'_, AppState>, name: String) -> Result<(), String> {
    st.store.delete_preset(&name).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OverlayTarget {
    TimerRegion,
    StepPoint { seq: SeqId, index: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlaySession {
    pub target: OverlayTarget,
    pub title: String,
    pub roblox: PxRect,
    pub overlay_origin: PxPoint,
    pub region: Option<RelRect>,
    pub point: Option<RelPoint>,
    pub path: Vec<Option<RelPoint>>,
}

fn click_points(s: &Settings, seq: SeqId) -> Vec<Option<RelPoint>> {
    s.steps(seq)
        .iter()
        .map(|st| match st.action {
            Action::Click { point, .. } | Action::Scroll { point, .. } => point,
            _ => None,
        })
        .collect()
}

pub fn open_overlay(app: &AppHandle, target: OverlayTarget) -> Result<OverlaySession, String> {
    let st = app.state::<AppState>();
    if st.settings.read().setup.auto_maximize && st.platform.window.normalize() == crate::core::types::WindowFix::Maximized {
        std::thread::sleep(std::time::Duration::from_millis(600));
        *st.roblox.write() = st.platform.window.find();
    }
    let roblox = st.roblox.read().map(|w| w.client).ok_or("Roblox window not found")?;
    let s = st.settings.read().clone();
    let session = match target {
        OverlayTarget::TimerRegion => OverlaySession {
            target,
            title: "Edit layout".into(),
            roblox,
            overlay_origin: PxPoint { x: roblox.x, y: roblox.y },
            region: Some(s.timer.region),
            point: None,
            path: Vec::new(),
        },
        OverlayTarget::StepPoint { seq, index } => {
            let step = s.steps(seq).get(index).ok_or("That step no longer exists")?;
            let point = match step.action {
                Action::Click { point, .. } | Action::Scroll { point, .. } => point,
                _ => return Err("Only click and scroll steps have a point".into()),
            };
            let note = if step.note.is_empty() { String::new() } else { format!(" · {}", step.note) };
            OverlaySession {
                target,
                title: format!("{} step {}{note}", seq.label(), index + 1),
                roblox,
                overlay_origin: PxPoint { x: roblox.x, y: roblox.y },
                region: None,
                point,
                path: click_points(&s, seq),
            }
        }
    };
    windows::show_overlay(app, roblox, true)?;
    *st.overlay_session.lock() = Some(serde_json::to_value(&session).map_err(|e| e.to_string())?);
    if let Some(o) = app.get_webview_window("overlay") {
        let _ = o.emit("overlay:session", &session);
    }
    Ok(session)
}

pub fn open_timer_editor(app: &AppHandle) -> Result<OverlaySession, String> {
    open_overlay(app, OverlayTarget::TimerRegion)
}

#[tauri::command]
pub fn overlay_open(app: AppHandle, target: OverlayTarget) -> Result<OverlaySession, String> {
    open_overlay(&app, target)
}

#[derive(Debug, Deserialize)]
pub struct OverlayCommit {
    pub target: OverlayTarget,
    pub region: Option<RelRect>,
    pub point: Option<RelPoint>,
}

#[tauri::command]
pub fn overlay_commit(app: AppHandle, st: State<'_, AppState>, commit: OverlayCommit) -> Result<Settings, String> {
    let mut s = st.settings.read().clone();
    match commit.target {
        OverlayTarget::TimerRegion => s.timer.region = commit.region.ok_or("region required")?,
        OverlayTarget::StepPoint { seq, index } => {
            let step = s.steps_mut(seq).get_mut(index).ok_or("That step no longer exists")?;
            if let Action::Click { point, .. } | Action::Scroll { point, .. } = &mut step.action {
                *point = Some(commit.point.ok_or("point required")?);
            }
        }
    }
    windows::hide_overlay(&app);
    settings_set(app, st, s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn overlay_cancel(app: AppHandle) {
    windows::hide_overlay(&app);
}

#[tauri::command]
pub fn overlay_pending(st: State<'_, AppState>) -> Option<serde_json::Value> {
    st.overlay_session.lock().clone()
}

#[tauri::command]
pub fn panel_placement_changed(app: AppHandle) {
    windows::save_panel_placement(&app);
}

#[derive(Debug, Serialize)]
pub struct RegionPreview {
    pub width: usize,
    pub height: usize,
    pub png_base64: String,
}

#[tauri::command]
pub async fn region_preview(st: State<'_, AppState>, region: RelRect, max_dim: Option<u32>) -> Result<RegionPreview, String> {
    let roblox = st.roblox.read().map(|w| w.client).ok_or("Roblox window not found")?;
    let capture = st.platform.capture.clone();
    let max_dim = max_dim.unwrap_or(320) as usize;
    blocking(move || {
        let frame = capture.grab(region.to_px(&roblox)).map_err(|e| e.to_string())?;
        let png = encode_png(&frame.downscale(max_dim))?;
        Ok(RegionPreview { width: frame.w, height: frame.h, png_base64: png })
    })
    .await
}

#[derive(Debug, Serialize)]
pub struct TimerTest {
    pub text: String,
    pub seconds: Option<u32>,
    pub png_base64: String,
    pub method: String,
}

#[tauri::command]
pub async fn timer_test(st: State<'_, AppState>, region: Option<RelRect>) -> Result<TimerTest, String> {
    let roblox = st.roblox.read().map(|w| w.client).ok_or("Roblox window not found")?;
    let mut timer = st.settings.read().timer.clone();
    if let Some(r) = region {
        timer.region = r;
    }
    let platform = st.platform.clone();
    blocking(move || {
        let r = timer_watch::read_timer(&platform, roblox, &timer)?;
        Ok(TimerTest { text: r.text.trim().to_string(), seconds: r.seconds, png_base64: encode_png(&r.frame.downscale(320))?, method: r.method.to_string() })
    })
    .await
}

#[tauri::command]
pub fn hud_set_offset(app: AppHandle, st: State<'_, AppState>, offset: RelPoint) -> Result<(), String> {
    let mut s = st.settings.read().clone();
    s.ui.hud_offset = offset;
    settings_set(app, st, s)
}

#[tauri::command]
pub fn hud_toggle(app: AppHandle, st: State<'_, AppState>) -> Result<bool, String> {
    let mut s = st.settings.read().clone();
    s.ui.hud_visible = !s.ui.hud_visible;
    let v = s.ui.hud_visible;
    settings_set(app.clone(), st, s)?;
    windows::set_hud_visible(&app, v);
    Ok(v)
}

#[tauri::command]
pub fn panel_show(app: AppHandle, st: State<'_, AppState>) {
    st.panel_requested.store(true, std::sync::atomic::Ordering::SeqCst);
    windows::show_panel(&app);
}

#[tauri::command]
pub fn panel_toggle(app: AppHandle) {
    windows::toggle_panel(&app);
}

#[tauri::command]
pub fn panel_hide(app: AppHandle) {
    windows::hide_panel(&app);
}

#[tauri::command]
pub fn panel_visible(app: AppHandle) -> bool {
    windows::panel_visible(&app)
}

#[tauri::command]
pub fn guide_open(app: AppHandle) -> Result<(), String> {
    windows::show_guide(&app)
}

#[tauri::command]
pub fn guide_hide(app: AppHandle) {
    windows::hide_guide(&app);
}

#[tauri::command]
pub fn app_quit(app: AppHandle, st: State<'_, AppState>) {
    st.bot.stop();
    app.exit(0);
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn webhook_test(st: State<'_, AppState>) -> Result<(), String> {
    let wh = Arc::clone(&st.bot.ctx().webhook);
    blocking(move || wh.test()).await
}

#[tauri::command]
pub fn hotkey_conflicts() -> Vec<String> {
    hotkeys::conflicts()
}

#[tauri::command]
pub fn data_dir(st: State<'_, AppState>) -> String {
    st.store.dir().display().to_string()
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

fn encode_png(frame: &crate::core::types::Frame) -> Result<String, String> {
    use base64::Engine;
    use image::ImageEncoder;
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new_with_quality(
        &mut out,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    )
    .write_image(&frame.rgba, frame.w as u32, frame.h as u32, image::ExtendedColorType::Rgba8)
    .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(out))
}
