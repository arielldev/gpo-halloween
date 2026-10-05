use std::time::{Duration, Instant};

use crate::config::{Timer, TimerMode};
use crate::core::platform::Platform;
use crate::core::ocrprep::{clock_groups, prepare, Prep};
use crate::core::timer::{fmt_clock, parse_clock, parse_clock_digits, two_digits, ClockFilter, Direction};
use crate::core::types::{Frame, PxRect};
use crate::events::{BotEvent, TimerReading, TimerSource};

use super::ctx::Ctx;

pub struct OcrRead {
    pub text: String,
    pub seconds: Option<u32>,
    pub frame: Frame,
    pub method: &'static str,
}

static LAST_GOOD: parking_lot::Mutex<Option<Prep>> = parking_lot::Mutex::new(None);

pub fn read_timer(platform: &Platform, client: PxRect, timer: &Timer) -> Result<OcrRead, String> {
    if !platform.ocr.available() {
        return Err("Windows OCR is not available on this system".into());
    }
    let frame = platform.capture.grab(timer.region.to_px(&client)).map_err(|e| e.to_string())?;
    let scale = timer.ocr_scale.clamp(2, 6) as usize;
    if let Some(r) = read_groups(platform, &frame, scale) {
        return Ok(OcrRead { seconds: parse_clock_digits(&r.replace(':', "")), text: r, frame, method: "groups" });
    }
    let mut order: Vec<Prep> = Prep::ALL.to_vec();
    if let Some(last) = *LAST_GOOD.lock() {
        order.retain(|p| *p != last);
        order.insert(0, last);
    }
    let mut first_text = String::new();
    for prep in order {
        let img = prepare(&frame, prep, scale);
        let text = platform.ocr.read(&img).map_err(|e| e.to_string())?;
        let parsed = if prep.digits_only() { parse_clock_digits(&text) } else { parse_clock(&text) };
        if let Some(seconds) = parsed {
            *LAST_GOOD.lock() = Some(prep);
            return Ok(OcrRead { seconds: Some(seconds), text, frame, method: prep.name() });
        }
        if first_text.trim().is_empty() {
            first_text = text;
        }
    }
    Ok(OcrRead { seconds: None, text: first_text, frame, method: "none" })
}

fn read_groups(platform: &Platform, frame: &Frame, scale: usize) -> Option<String> {
    let mut found: [Option<String>; 3] = [None, None, None];
    let mut sizes = vec![2usize, 3, 4, 5, scale];
    sizes.dedup();
    for k in sizes {
        let Some(groups) = clock_groups(frame, k) else { continue };
        for (slot, g) in found.iter_mut().zip(groups.iter()) {
            if slot.is_none() {
                *slot = platform.ocr.read(g).ok().as_deref().and_then(two_digits);
            }
        }
        if found.iter().all(|f| f.is_some()) {
            break;
        }
    }
    let [Some(h), Some(m), Some(s)] = found else { return None };
    let digits = format!("{h}{m}{s}");
    parse_clock_digits(&digits)?;
    Some(format!("{h}:{m}:{s}"))
}

pub fn run(ctx: &Ctx) {
    let started = Instant::now();
    let mut filter = ClockFilter::default();
    let mut was_armed = false;
    let mut misses = 0u32;
    let mut warned = false;
    let mut last_good: Option<(u32, Instant)> = None;
    while ctx.running() {
        let t = ctx.settings.read().timer.clone();
        let next = Instant::now() + Duration::from_millis(t.interval_ms.clamp(250, 10_000) as u64);
        while ctx.running() && Instant::now() < next {
            std::thread::sleep(Duration::from_millis(25));
        }
        if !ctx.running() {
            return;
        }
        if !t.enabled {
            continue;
        }
        let armed = ctx.timer_armed();
        if armed && !was_armed {
            filter.reset();
            misses = 0;
        }
        was_armed = armed;
        let dir = if t.mode == TimerMode::Elapsed { Direction::Up } else { Direction::Down };
        let ocr = match ctx.roblox_rect() {
            Some(client) => read_timer(&ctx.platform, client, &t),
            None => Err("Roblox window not found".into()),
        };
        let estimate = |at: &(u32, Instant)| -> u32 {
            let dt = at.1.elapsed().as_secs() as u32;
            match dir {
                Direction::Up => at.0 + dt,
                Direction::Down => at.0.saturating_sub(dt),
            }
        };
        let fresh = |lg: &Option<(u32, Instant)>| lg.filter(|g| g.1.elapsed() < Duration::from_secs(10));
        let mut raw = String::new();
        let mut estimated = false;
        let (seconds, source, confirmed) = match ocr {
            Ok(OcrRead { seconds: Some(s), text, .. }) => {
                misses = 0;
                raw = text;
                let streak = filter.feed(s, started.elapsed().as_secs_f64(), dir);
                let shown = match fresh(&last_good) {
                    Some(g) if streak < 2 => {
                        estimated = true;
                        estimate(&g)
                    }
                    _ => {
                        last_good = Some((s, Instant::now()));
                        s
                    }
                };
                (Some(shown), TimerSource::Ocr, t.reached(s) && streak >= t.confirm_reads.max(2))
            }
            other => {
                misses += 1;
                match other {
                    Ok(r) => raw = r.text,
                    Err(e) if !warned => {
                        warned = true;
                        ctx.log_warn(&format!("Timer: {e}"));
                    }
                    Err(_) => {}
                }
                match (ctx.server_clock(), fresh(&last_good)) {
                    (Some(c), _) if t.fallback && misses >= t.fallback_after_misses.max(3) => {
                        (Some(c), TimerSource::Clock, c >= t.fallback_stop_s)
                    }
                    (_, Some(g)) => {
                        estimated = true;
                        (Some(estimate(&g)), TimerSource::Ocr, false)
                    }
                    _ => (None, TimerSource::None, false),
                }
            }
        };
        let tripped = armed && confirmed && ctx.trip_timer();
        let reading = TimerReading { seconds, source, raw: raw.trim().to_string(), armed, tripped, estimated };
        *ctx.last_timer.lock() = Some(reading.clone());
        ctx.emit(BotEvent::Timer(reading));
        if tripped {
            let shown = seconds.map(fmt_clock).unwrap_or_default();
            let via = if source == TimerSource::Clock { " (backup clock, OCR could not read the timer)" } else { "" };
            ctx.log_warn(&format!("Server timer hit {shown}{via}; stopping the route"));
        }
    }
}
