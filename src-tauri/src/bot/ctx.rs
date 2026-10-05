use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};

use crate::config::{Settings, Store};
use crate::core::platform::Platform;
use crate::core::types::{MouseButton, PxRect, WindowInfo};
use crate::events::{now_ms, BotEvent, BotState, LogLevel, LogLine, TimerReading};

use super::session::Session;
use super::Job;

const GATE_DISARMED: u8 = 0;
const GATE_ARMED: u8 = 1;
const GATE_TRIPPED: u8 = 2;

pub struct Ctx {
    pub platform: Platform,
    pub settings: Arc<RwLock<Settings>>,
    pub roblox: Arc<RwLock<Option<WindowInfo>>>,
    pub events: Sender<BotEvent>,
    pub store: Arc<Store>,
    pub session: Mutex<Session>,
    pub job: Mutex<Option<Job>>,
    pub last_timer: Mutex<Option<TimerReading>>,
    running: AtomicBool,
    timer_gate: AtomicU8,
    joined_at: Mutex<Option<Instant>>,
    heartbeat_ms: AtomicU64,
    state: Mutex<(BotState, Instant)>,
    start: Instant,
}

impl Ctx {
    pub fn new(
        platform: Platform,
        settings: Arc<RwLock<Settings>>,
        roblox: Arc<RwLock<Option<WindowInfo>>>,
        events: Sender<BotEvent>,
        store: Arc<Store>,
    ) -> Self {
        let session = Session::with_base(store.load_stats());
        Self {
            platform,
            settings,
            roblox,
            events,
            store,
            session: Mutex::new(session),
            job: Mutex::new(None),
            last_timer: Mutex::new(None),
            running: AtomicBool::new(false),
            timer_gate: AtomicU8::new(GATE_DISARMED),
            joined_at: Mutex::new(None),
            heartbeat_ms: AtomicU64::new(0),
            state: Mutex::new((BotState::Stopped, Instant::now())),
            start: Instant::now(),
        }
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_running(&self, v: bool) {
        self.running.store(v, Ordering::SeqCst);
    }

    pub fn alive(&self) -> bool {
        self.running() && self.timer_gate.load(Ordering::SeqCst) != GATE_TRIPPED
    }

    pub fn arm_timer(&self) {
        let _ = self.timer_gate.compare_exchange(GATE_DISARMED, GATE_ARMED, Ordering::SeqCst, Ordering::SeqCst);
    }

    pub fn timer_armed(&self) -> bool {
        self.timer_gate.load(Ordering::SeqCst) == GATE_ARMED
    }

    pub fn trip_timer(&self) -> bool {
        self.timer_gate.compare_exchange(GATE_ARMED, GATE_TRIPPED, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    pub fn disarm_timer(&self) -> bool {
        self.timer_gate.swap(GATE_DISARMED, Ordering::SeqCst) == GATE_TRIPPED
    }

    pub fn mark_joined(&self) {
        *self.joined_at.lock() = Some(Instant::now());
    }

    pub fn clear_joined(&self) {
        *self.joined_at.lock() = None;
    }

    pub fn server_clock(&self) -> Option<u32> {
        self.joined_at.lock().map(|t| t.elapsed().as_secs() as u32)
    }

    pub fn touch(&self) {
        self.heartbeat_ms.store(self.start.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    pub fn heartbeat_age(&self) -> Duration {
        let now = self.start.elapsed().as_millis() as u64;
        Duration::from_millis(now.saturating_sub(self.heartbeat_ms.load(Ordering::Relaxed)))
    }

    pub fn state(&self) -> BotState {
        self.state.lock().0
    }

    pub fn state_age(&self) -> Duration {
        self.state.lock().1.elapsed()
    }

    pub fn set_state(&self, s: BotState, detail: Option<String>) {
        if !self.running() && s != BotState::Stopped {
            return;
        }
        {
            let mut cur = self.state.lock();
            if cur.0 == s && detail.is_none() {
                return;
            }
            if cur.0 != s {
                *cur = (s, Instant::now());
            }
        }
        self.touch();
        self.emit(BotEvent::State { state: s, detail });
    }

    pub fn emit(&self, e: BotEvent) {
        let _ = self.events.send(e);
    }

    pub fn emit_stats(&self) {
        let stats = self.session.lock().stats();
        if let Err(e) = self.store.save_stats(&stats.total) {
            tracing::warn!("stats save: {e}");
        }
        self.emit(BotEvent::Stats(stats));
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        match level {
            LogLevel::Debug => tracing::debug!("{msg}"),
            LogLevel::Info => tracing::info!("{msg}"),
            LogLevel::Warn => tracing::warn!("{msg}"),
            LogLevel::Error => tracing::error!("{msg}"),
        }
        self.emit(BotEvent::Log(LogLine { ts: now_ms(), level, msg: msg.to_string() }));
    }

    pub fn log_debug(&self, m: &str) {
        self.log(LogLevel::Debug, m)
    }
    pub fn log_info(&self, m: &str) {
        self.log(LogLevel::Info, m)
    }
    pub fn log_warn(&self, m: &str) {
        self.log(LogLevel::Warn, m)
    }
    pub fn log_error(&self, m: &str) {
        self.log(LogLevel::Error, m)
    }

    pub fn roblox_rect(&self) -> Option<PxRect> {
        self.roblox.read().map(|w| w.client)
    }

    pub fn roblox_in_front(&self) -> bool {
        self.roblox.read().map(|w| w.visible && w.is_foreground).unwrap_or(false)
    }

    pub fn ensure_roblox_focus(&self) -> bool {
        if !self.roblox_in_front() {
            return false;
        }
        if self.platform.window.game_foreground() {
            return true;
        }
        if !self.platform.window.focus() {
            return false;
        }
        let _ = self.sleep_ms(100);
        true
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().clone()
    }

    pub fn sleep(&self, d: Duration) -> bool {
        let end = Instant::now() + d;
        while Instant::now() < end {
            if !self.alive() {
                return false;
            }
            let left = end - Instant::now();
            std::thread::sleep(left.min(Duration::from_millis(20)));
            self.touch();
        }
        self.alive()
    }

    pub fn sleep_ms(&self, ms: u32) -> bool {
        self.sleep(Duration::from_millis(ms as u64))
    }

    pub fn release_inputs(&self) {
        let input = &self.platform.input;
        input.button(MouseButton::Left, false);
        input.button(MouseButton::Right, false);
    }
}
