use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::AppState;
use crate::config::{Hotkeys, SeqId};
use crate::core::types::Key;
use crate::windows;

#[derive(Clone, Copy)]
struct Fallback {
    key: Key,
    ctrl: bool,
    shift: bool,
    alt: bool,
    action: Action,
}

static FALLBACKS: Mutex<Vec<(String, Fallback)>> = Mutex::new(Vec::new());
static POLLER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn parse_combo(combo: &str) -> Option<(Key, bool, bool, bool)> {
    let parts: Vec<&str> = combo.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    let (last, mods) = parts.split_last()?;
    let has = |m: &str| mods.iter().any(|x| x.eq_ignore_ascii_case(m));
    Some((Key::parse(last)?, has("ctrl") || has("control"), has("shift"), has("alt")))
}

pub fn conflicts() -> Vec<String> {
    FALLBACKS.lock().iter().map(|(c, _)| c.clone()).collect()
}

fn start_poller(app: &AppHandle) {
    if POLLER.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::Builder::new()
        .name("hotkey-fallback".into())
        .spawn(move || {
            let input = app.state::<AppState>().platform.input.clone();
            let mut was_down: Vec<bool> = Vec::new();
            loop {
                std::thread::sleep(Duration::from_millis(30));
                let list: Vec<Fallback> = FALLBACKS.lock().iter().map(|(_, f)| *f).collect();
                was_down.resize(list.len(), false);
                for (i, f) in list.iter().enumerate() {
                    let down = input.key_down(f.key)
                        && input.key_down(Key::Control) == f.ctrl
                        && input.key_down(Key::Shift) == f.shift
                        && input.key_down(Key::Alt) == f.alt;
                    if down && !was_down[i] {
                        dispatch(&app, f.action);
                    }
                    was_down[i] = down;
                }
            }
        })
        .expect("spawn hotkey fallback");
}

const REPEAT_GUARD: Duration = Duration::from_millis(700);

static LAST_PRESS: Mutex<Option<(u8, Instant)>> = Mutex::new(None);

fn repeated(id: u8) -> bool {
    let mut last = LAST_PRESS.lock();
    let now = Instant::now();
    let hit = matches!(*last, Some((prev, at)) if prev == id && now.duration_since(at) < REPEAT_GUARD);
    *last = Some((id, now));
    hit
}

#[derive(Clone, Copy)]
enum Action {
    Toggle,
    Overlay,
    Quit,
    HideHud,
    Record,
    Run(SeqId),
}

pub fn register(app: &AppHandle, keys: &Hotkeys) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;
    let bindings = [
        (keys.toggle.as_str(), Action::Toggle),
        (keys.overlay.as_str(), Action::Overlay),
        (keys.quit.as_str(), Action::Quit),
        (keys.hide_hud.as_str(), Action::HideHud),
        (keys.record.as_str(), Action::Record),
        (keys.run_leave.as_str(), Action::Run(SeqId::Leave)),
        (keys.run_lobby.as_str(), Action::Run(SeqId::Lobby)),
        (keys.run_macro.as_str(), Action::Run(SeqId::Macro)),
    ];
    let mut errors = Vec::new();
    let mut fallbacks: Vec<(String, Fallback)> = Vec::new();
    for (combo, action) in bindings {
        if combo.trim().is_empty() {
            continue;
        }
        let shortcut: Shortcut = match combo.parse() {
            Ok(s) => s,
            Err(_) => {
                errors.push(format!("Invalid hotkey: {combo}"));
                continue;
            }
        };
        if let Err(e) = gs.on_shortcut(shortcut, move |app, _sc, ev| {
            if ev.state() == ShortcutState::Pressed {
                dispatch(app, action);
            }
        }) {
            match parse_combo(combo) {
                Some((key, ctrl, shift, alt)) => {
                    fallbacks.push((combo.to_string(), Fallback { key, ctrl, shift, alt, action }));
                }
                None => errors.push(format!("Could not bind {combo}: {e}")),
            }
        }
    }
    let taken: Vec<String> = fallbacks.iter().map(|(c, _)| c.clone()).collect();
    *FALLBACKS.lock() = fallbacks;
    if !taken.is_empty() {
        start_poller(app);
        let st = app.state::<AppState>();
        st.bot.ctx().log_warn(&format!(
            "{} {} used by another app, so the macro listens for {} directly. If it doesn't react, pick another key in Settings › Hotkeys.",
            taken.join(", "),
            if taken.len() == 1 { "is" } else { "are" },
            if taken.len() == 1 { "it" } else { "them" }
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn action_id(a: Action) -> u8 {
    match a {
        Action::Toggle => 0,
        Action::Overlay => 1,
        Action::Quit => 2,
        Action::HideHud => 3,
        Action::Record => 4,
        Action::Run(SeqId::Leave) => 5,
        Action::Run(SeqId::Lobby) => 6,
        Action::Run(SeqId::Macro) => 7,
        Action::Run(SeqId::Buy) => 8,
    }
}

fn dispatch(app: &AppHandle, action: Action) {
    if repeated(action_id(action)) {
        return;
    }
    let st = app.state::<AppState>();
    let bot = Arc::clone(&st.bot);
    match action {
        Action::Toggle => {
            std::thread::spawn(move || bot.toggle());
        }
        Action::Record => {
            std::thread::spawn(move || bot.record_toggle(None));
        }
        Action::Run(seq) => {
            std::thread::spawn(move || {
                if bot.is_running() {
                    bot.ctx().log_info(&format!("Hotkey for {} pressed while running: stopping", seq.label()));
                    bot.stop();
                } else {
                    bot.ctx().log_info(&format!("Hotkey: test run {}", seq.label()));
                    bot.run_once(seq);
                }
            });
        }
        Action::Overlay => {
            if windows::overlay_visible(app) {
                windows::hide_overlay(app);
            } else if let Err(e) = crate::commands::open_timer_editor(app) {
                tracing::warn!("timer editor: {e}");
            }
        }
        Action::Quit => {
            st.bot.stop();
            app.exit(0);
        }
        Action::HideHud => {
            let mut s = st.settings.read().clone();
            s.ui.hud_visible = !s.ui.hud_visible;
            let v = s.ui.hud_visible;
            *st.settings.write() = s.clone();
            let _ = st.store.save(&s);
            windows::set_hud_visible(app, v);
        }
    }
}
