use crate::config::{Action, SeqId, Step, CODE_PLACEHOLDER};
use crate::core::types::{Key, MouseButton, PxPoint, RelPoint};
use crate::events::{BotEvent, BotState};

use super::ctx::Ctx;

const WHEEL_STEP: i32 = 120;
const TILT_CHUNK_PX: i32 = 20;

pub fn click(ctx: &Ctx, p: PxPoint, button: MouseButton) -> bool {
    let input = &ctx.platform.input;
    input.move_to(p);
    if !ctx.sleep_ms(40) {
        return false;
    }
    input.button(button, true);
    let ok = ctx.sleep_ms(40);
    input.button(button, false);
    ok
}

pub fn key_hold(ctx: &Ctx, k: Key, ms: u32) -> bool {
    ctx.platform.input.key(k, true);
    let ok = ctx.sleep_ms(ms.max(20));
    ctx.platform.input.key(k, false);
    ok
}

pub fn type_text(ctx: &Ctx, text: &str) -> bool {
    for c in text.chars() {
        if !key_hold(ctx, Key::Char(c), 30) || !ctx.sleep_ms(40) {
            return false;
        }
    }
    true
}

pub fn paste(ctx: &Ctx, text: &str) -> bool {
    if !ctx.platform.input.set_clipboard(text) {
        ctx.log_warn("Could not set the clipboard; typing instead");
        return type_text(ctx, text);
    }
    if !ctx.sleep_ms(60) {
        return false;
    }
    let input = &ctx.platform.input;
    input.key(Key::Control, true);
    let ok = ctx.sleep_ms(40) && key_hold(ctx, Key::Char('v'), 40);
    input.key(Key::Control, false);
    ok
}

pub fn wheel_steps(ctx: &Ctx, steps: u32, dir: i32, step_delay_ms: u32) -> bool {
    for _ in 0..steps {
        ctx.platform.input.wheel(WHEEL_STEP * dir);
        if !ctx.sleep_ms(step_delay_ms) {
            return false;
        }
    }
    true
}

fn rel_to_px(ctx: &Ctx, p: RelPoint) -> Option<PxPoint> {
    ctx.roblox_rect().map(|r| p.to_px(&r))
}

pub fn camera_setup(ctx: &Ctx) -> bool {
    let cam = ctx.settings.read().camera.clone();
    if !cam.enabled {
        return true;
    }
    ctx.set_state(BotState::CameraSetup, None);
    let Some(center) = ctx.roblox_rect().map(|r| r.center()) else { return false };
    ctx.log_debug("Camera: zooming out");
    ctx.platform.input.move_to(center);
    if !ctx.sleep_ms(120) {
        return false;
    }
    if !wheel_steps(ctx, cam.out_steps, -1, cam.step_delay_ms) {
        return false;
    }
    if cam.in_steps > 0 && (!ctx.sleep_ms(200) || !wheel_steps(ctx, cam.in_steps, 1, cam.step_delay_ms)) {
        return false;
    }
    if cam.tilt_px != 0 {
        ctx.log_debug("Camera: tilting");
        let input = &ctx.platform.input;
        input.move_to(center);
        if !ctx.sleep_ms(80) {
            return false;
        }
        input.button(MouseButton::Right, true);
        let mut left = cam.tilt_px;
        let mut ok = ctx.sleep_ms(60);
        while ok && left != 0 {
            let d = left.signum() * left.abs().min(TILT_CHUNK_PX);
            input.move_rel(0, d);
            left -= d;
            ok = ctx.sleep_ms(10);
        }
        input.button(MouseButton::Right, false);
        if !ok {
            return false;
        }
        input.move_to(center);
    }
    ctx.sleep_ms(cam.settle_ms)
}

pub fn prepare_route(ctx: &Ctx) -> bool {
    if !camera_setup(ctx) {
        return false;
    }
    let equip = ctx.settings.read().keys.equip.clone();
    if equip.trim().is_empty() {
        return true;
    }
    let Some(k) = Key::parse(&equip) else {
        ctx.log_warn(&format!("Item key \"{equip}\" is not a valid key; skipped"));
        return true;
    };
    ctx.log_debug(&format!("Equipping item ({})", k.name()));
    key_hold(ctx, k, 60) && ctx.sleep_ms(500)
}

pub fn ensure_front(ctx: &Ctx) -> bool {
    if ctx.ensure_roblox_focus() {
        return true;
    }
    let prev = ctx.state();
    ctx.set_state(BotState::WaitingForRoblox, Some("Roblox is not in front".into()));
    ctx.log_warn("Roblox is not the active window; waiting");
    while ctx.alive() {
        if ctx.ensure_roblox_focus() {
            ctx.log_info("Roblox is back in front");
            ctx.set_state(prev, None);
            return true;
        }
        if !ctx.sleep_ms(500) {
            return false;
        }
    }
    false
}

pub fn wait_for_roblox(ctx: &Ctx) -> bool {
    if ctx.roblox_rect().is_some() {
        return true;
    }
    ctx.set_state(BotState::WaitingForRoblox, None);
    ctx.log_warn("Waiting for Roblox window");
    while ctx.alive() {
        if ctx.roblox_rect().is_some() {
            ctx.log_info("Roblox found");
            return true;
        }
        if !ctx.sleep_ms(500) {
            return false;
        }
    }
    false
}

pub fn run_sequence(ctx: &Ctx, seq: SeqId) -> bool {
    run_range(ctx, seq, 0, None)
}

pub fn run_range(ctx: &Ctx, seq: SeqId, from: usize, end: Option<usize>) -> bool {
    let steps = ctx.settings.read().steps(seq).clone();
    let total = steps.len();
    if from >= total {
        ctx.log_warn(&format!("{} has no step {}", seq.label(), from + 1));
        return false;
    }
    let end = end.unwrap_or(total).min(total);
    if from == 0 && end == total {
        ctx.log_info(&format!("{}: {} steps", seq.label(), total));
    } else if end == from + 1 {
        ctx.log_info(&format!("{}: running step {} only", seq.label(), from + 1));
    } else {
        ctx.log_info(&format!("{}: running steps {} to {}", seq.label(), from + 1, end));
    }
    for (i, step) in steps.iter().enumerate().take(end).skip(from) {
        if !ctx.alive() || !ensure_front(ctx) {
            return false;
        }
        if !step.enabled {
            if end == from + 1 {
                ctx.log_warn(&format!("{} step {} is switched off; turn it on to run it", seq.label(), i + 1));
            }
            continue;
        }
        ctx.emit(BotEvent::Progress { seq, index: i, total });
        if !run_step(ctx, seq, i, step) {
            return false;
        }
    }
    ctx.emit(BotEvent::Progress { seq, index: total, total });
    true
}

fn run_step(ctx: &Ctx, seq: SeqId, i: usize, step: &Step) -> bool {
    let ok = match &step.action {
        Action::Click { point: None, .. } => {
            ctx.log_warn(&format!("{} step {}: no click point set; skipped", seq.label(), i + 1));
            true
        }
        Action::Click { point: Some(p), button } => match rel_to_px(ctx, *p) {
            Some(px) => click(ctx, px, *button),
            None => false,
        },
        Action::Key { key, hold_ms } => match Key::parse(key) {
            Some(k) => key_hold(ctx, k, *hold_ms),
            None => {
                ctx.log_warn(&format!("{} step {}: unknown key \"{key}\"; skipped", seq.label(), i + 1));
                true
            }
        },
        Action::Type { text } => {
            let code = ctx.settings.read().server.code.clone();
            type_text(ctx, &text.replace(CODE_PLACEHOLDER, code.trim()))
        }
        Action::Paste { text } => {
            let code = ctx.settings.read().server.code.clone();
            paste(ctx, &text.replace(CODE_PLACEHOLDER, code.trim()))
        }
        Action::Scroll { amount, point } => {
            if let Some(px) = point.and_then(|p| rel_to_px(ctx, p)) {
                ctx.platform.input.move_to(px);
                if !ctx.sleep_ms(80) {
                    return false;
                }
            }
            let delay = ctx.settings.read().camera.step_delay_ms;
            wheel_steps(ctx, amount.unsigned_abs(), amount.signum(), delay)
        }
        Action::Interact => interact(ctx, seq),
        Action::Wait => true,
    };
    ok && ctx.sleep_ms(step.wait_ms)
}

fn interact(ctx: &Ctx, seq: SeqId) -> bool {
    let (key, hold, wait) = {
        let s = ctx.settings.read();
        (s.keys.interact.clone(), s.route.interact_hold_ms, s.route.house_wait_ms)
    };
    let Some(k) = Key::parse(&key) else {
        ctx.log_warn(&format!("Interact key \"{key}\" is not a valid key"));
        return ctx.sleep_ms(wait);
    };
    if !key_hold(ctx, k, hold) {
        return false;
    }
    if seq == SeqId::Macro {
        let n = {
            let mut s = ctx.session.lock();
            s.houses += 1;
            s.houses
        };
        ctx.log_info(&format!("House #{n}: pressed {}, waiting {:.1}s", k.name(), wait as f32 / 1000.0));
        ctx.emit_stats();
    }
    ctx.sleep_ms(wait)
}
