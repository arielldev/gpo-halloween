use std::collections::HashMap;
use std::time::Instant;

use crate::config::{Action, RecordMode, SeqId, Step, CODE_PLACEHOLDER};
use crate::core::types::{Key, MouseButton};
use crate::events::{BotEvent, BotState};

use super::actions;
use super::ctx::Ctx;

const POLL_MS: u32 = 5;
const DEBOUNCE_POLLS: u32 = 2;
const TYPE_MAX_HOLD_MS: u32 = 400;
const TYPE_MAX_GAP_MS: u32 = 1500;

struct Edge {
    down: bool,
    streak: u32,
}

impl Edge {
    fn new() -> Self {
        Self { down: false, streak: 0 }
    }

    fn feed(&mut self, raw: bool) -> Option<bool> {
        if raw == self.down {
            self.streak = 0;
            return None;
        }
        self.streak += 1;
        if self.streak < DEBOUNCE_POLLS {
            return None;
        }
        self.streak = 0;
        self.down = raw;
        Some(raw)
    }
}

fn round(ms: u128, step: u32) -> u32 {
    let step = step.max(1) as u128;
    (((ms + step / 2) / step) * step).min(u32::MAX as u128) as u32
}

pub struct Recorder {
    steps: Vec<Step>,
    last_end: Option<Instant>,
    round_ms: u32,
}

impl Recorder {
    fn new(round_ms: u32) -> Self {
        Self { steps: Vec::new(), last_end: None, round_ms }
    }

    fn push(&mut self, at: Instant, action: Action) -> usize {
        if let (Some(prev), Some(end)) = (self.steps.last_mut(), self.last_end) {
            if prev.action != Action::Interact {
                prev.wait_ms = round(at.saturating_duration_since(end).as_millis(), self.round_ms);
            }
        }
        self.steps.push(Step::new(action, 0, ""));
        self.last_end = Some(at);
        self.steps.len() - 1
    }

    fn release(&mut self, idx: usize, pressed: Instant, at: Instant) {
        let hold = round(at.saturating_duration_since(pressed).as_millis(), 10).max(30);
        if let Some(Step { action: Action::Key { hold_ms, .. }, .. }) = self.steps.get_mut(idx) {
            *hold_ms = hold;
        }
        if idx + 1 == self.steps.len() {
            self.last_end = Some(at);
        }
    }
}

pub fn collapse_typing(steps: Vec<Step>, code: &str) -> Vec<Step> {
    let mut out: Vec<Step> = Vec::with_capacity(steps.len());
    let mut run: Vec<Step> = Vec::new();
    let flush = |run: &mut Vec<Step>, out: &mut Vec<Step>| {
        if run.len() >= 2 {
            let mut text: String = run
                .iter()
                .filter_map(|s| match &s.action {
                    Action::Key { key, .. } => key.chars().next(),
                    _ => None,
                })
                .collect();
            let code = code.trim();
            if !code.is_empty() && text.eq_ignore_ascii_case(code) {
                text = CODE_PLACEHOLDER.into();
            }
            let wait = run.last().map(|s| s.wait_ms).unwrap_or(0);
            out.push(Step::new(Action::Type { text }, wait, ""));
        } else {
            out.append(run);
        }
        run.clear();
    };
    for s in steps {
        let typable = matches!(&s.action, Action::Key { key, hold_ms } if key.chars().count() == 1 && key.chars().all(|c| c.is_ascii_alphanumeric()) && *hold_ms <= TYPE_MAX_HOLD_MS);
        if typable {
            let joins = run.last().map(|p| p.wait_ms <= TYPE_MAX_GAP_MS).unwrap_or(true);
            if !joins {
                flush(&mut run, &mut out);
            }
            run.push(s);
        } else {
            flush(&mut run, &mut out);
            out.push(s);
        }
    }
    flush(&mut run, &mut out);
    out
}

pub fn record(ctx: &Ctx, seq: SeqId) {
    if !actions::wait_for_roblox(ctx) || !actions::ensure_front(ctx) {
        return;
    }
    let s = ctx.settings();
    if seq.walks() && s.recording.camera_first {
        ctx.log_info("Recording: setting up the camera and item first so playback sees the same view");
        if !actions::prepare_route(ctx) {
            return;
        }
    }
    ctx.set_state(BotState::Recording, Some(seq.label().into()));
    let interact = Key::parse(&s.keys.interact);
    let hint = match seq {
        SeqId::Macro => format!("click to move, press {} at each house", s.keys.interact.to_uppercase()),
        _ => "click the buttons in order".into(),
    };
    ctx.log_info(&format!("Recording {}: {hint}. Press {} to finish.", seq.label(), s.hotkeys.record));
    ctx.emit(BotEvent::Recording { seq, steps: 0 });

    let input = ctx.platform.input.clone();
    let window = ctx.platform.window.clone();
    let mut rec = Recorder::new(s.recording.round_ms);
    let mut left = Edge::new();
    let keys: Vec<Key> = if s.recording.record_keys { Key::recordable() } else { interact.into_iter().collect() };
    let mut key_edges: HashMap<Key, Edge> = keys.iter().map(|k| (*k, Edge::new())).collect();
    let mut held: HashMap<Key, (usize, Instant)> = HashMap::new();

    while ctx.sleep_ms(POLL_MS) {
        let now = Instant::now();
        let mut added = false;
        if let Some(true) = left.feed(input.button_down(MouseButton::Left)) {
            let p = input.cursor();
            match ctx.roblox_rect() {
                Some(client) if !client.contains(p) => ctx.log_info(&format!("Click at {},{} is outside the Roblox window; skipped", p.x, p.y)),
                Some(_) if !window.owns_point(p) => ctx.log_info("Click landed on another window (HUD or panel); skipped"),
                Some(client) => {
                    let rel = p.to_rel(&client);
                    rec.push(now, Action::Click { point: Some(rel), button: MouseButton::Left });
                    ctx.log_info(&format!("Step {}: click at {:.1}%, {:.1}%", rec.steps.len(), rel.x * 100.0, rel.y * 100.0));
                    added = true;
                }
                None => ctx.log_info("Click skipped: Roblox window not found"),
            }
        }
        let front = window.game_foreground();
        for k in &keys {
            let Some(edge) = key_edges.get_mut(k) else { continue };
            match edge.feed(input.key_down(*k)) {
                Some(true) if front => {
                    if *k == Key::Char('v') && input.key_down(Key::Control) {
                        rec.push(now, Action::Paste { text: CODE_PLACEHOLDER.into() });
                        ctx.log_info(&format!("Step {}: paste server code", rec.steps.len()));
                    } else if Some(*k) == interact {
                        rec.push(now, Action::Interact);
                        ctx.log_info(&format!("Step {}: interact ({})", rec.steps.len(), k.name().to_uppercase()));
                    } else {
                        let idx = rec.push(now, Action::Key { key: k.name(), hold_ms: 60 });
                        held.insert(*k, (idx, now));
                    }
                    added = true;
                }
                Some(false) => {
                    if let Some((idx, at)) = held.remove(k) {
                        rec.release(idx, at, now);
                    }
                }
                _ => {}
            }
        }
        if added {
            ctx.emit(BotEvent::Recording { seq, steps: rec.steps.len() });
        }
    }

    commit(ctx, seq, rec.steps);
}

fn commit(ctx: &Ctx, seq: SeqId, mut steps: Vec<Step>) {
    if steps.is_empty() {
        ctx.log_warn(&format!("Recording {}: nothing captured, kept the old steps", seq.label()));
        return;
    }
    if let Some(last) = steps.last_mut() {
        if last.action != Action::Interact {
            last.wait_ms = 0;
        }
    }
    let snapshot = {
        let mut s = ctx.settings.write();
        if s.recording.collapse_typing {
            steps = collapse_typing(steps, &s.server.code);
        }
        let n = steps.len();
        let mode = s.recording.mode;
        let target = s.steps_mut(seq);
        match mode {
            RecordMode::Replace => *target = steps,
            RecordMode::Append => target.extend(steps),
        }
        ctx.log_info(&format!("Recorded {n} steps into {}", seq.label()));
        s.clone()
    };
    if let Err(e) = ctx.store.save(&snapshot) {
        ctx.log_error(&format!("Could not save settings: {e}"));
    }
    ctx.emit(BotEvent::SettingsChanged(Box::new(snapshot)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn waits_are_measured_between_actions() {
        let t0 = Instant::now();
        let mut r = Recorder::new(50);
        r.push(t0, Action::Click { point: None, button: MouseButton::Left });
        r.push(t0 + Duration::from_millis(1230), Action::Interact);
        r.push(t0 + Duration::from_millis(9000), Action::Click { point: None, button: MouseButton::Left });
        assert_eq!(r.steps[0].wait_ms, 1250);
        assert_eq!(r.steps[1].wait_ms, 0);
    }

    #[test]
    fn edge_debounces_single_poll_glitches() {
        let mut e = Edge::new();
        assert_eq!(e.feed(true), None);
        assert_eq!(e.feed(false), None);
        assert_eq!(e.feed(true), None);
        assert_eq!(e.feed(true), Some(true));
        assert_eq!(e.feed(true), None);
    }

    #[test]
    fn typing_collapses_to_code_placeholder() {
        let k = |c: &str, wait: u32| Step::new(Action::Key { key: c.into(), hold_ms: 60 }, wait, "");
        let steps = vec![
            Step::new(Action::Click { point: None, button: MouseButton::Left }, 300, ""),
            k("a", 100),
            k("b", 100),
            k("1", 400),
            Step::new(Action::Key { key: "Enter".into(), hold_ms: 60 }, 0, ""),
        ];
        let out = collapse_typing(steps, "AB1");
        assert_eq!(out.len(), 3);
        assert_eq!(out[1].action, Action::Type { text: CODE_PLACEHOLDER.into() });
        assert_eq!(out[1].wait_ms, 400);
    }
}
