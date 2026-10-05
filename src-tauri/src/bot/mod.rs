pub mod actions;
pub mod ctx;
pub mod machine;
pub mod recorder;
pub mod session;
pub mod timer_watch;
pub mod watchdog;

use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};
use serde::Serialize;

use crate::config::{SeqId, Settings, StartFrom, Store};
use crate::core::platform::Platform;
use crate::core::types::WindowInfo;
use crate::events::{BotEvent, BotState};
use ctx::Ctx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Job {
    Cycle { from: StartFrom },
    Once { seq: SeqId, from: usize, only: bool },
    Record { seq: SeqId },
}

pub struct Bot {
    ctx: Arc<Ctx>,
    thread: Mutex<Option<JoinHandle<()>>>,
    helpers: Mutex<Vec<JoinHandle<()>>>,
    lock: Mutex<()>,
}

fn join_unless_self(h: JoinHandle<()>) {
    if h.thread().id() != std::thread::current().id() {
        let _ = h.join();
    }
}

impl Bot {
    pub fn new(
        platform: Platform,
        settings: Arc<RwLock<Settings>>,
        roblox: Arc<RwLock<Option<WindowInfo>>>,
        events: Sender<BotEvent>,
        store: Arc<Store>,
    ) -> Arc<Self> {
        let ctx = Arc::new(Ctx::new(platform, settings, roblox, events, store));
        Arc::new(Self { ctx, thread: Mutex::new(None), helpers: Mutex::new(Vec::new()), lock: Mutex::new(()) })
    }

    pub fn ctx(&self) -> &Arc<Ctx> {
        &self.ctx
    }

    pub fn is_running(&self) -> bool {
        self.ctx.running()
    }

    pub fn state(&self) -> BotState {
        self.ctx.state()
    }

    pub fn job(&self) -> Option<Job> {
        *self.ctx.job.lock()
    }

    pub fn start(self: &Arc<Self>, job: Job) -> bool {
        let _g = self.lock.lock();
        if self.is_running() {
            return false;
        }
        self.reap();
        *self.ctx.job.lock() = Some(job);
        if let Job::Cycle { .. } = job {
            {
                let mut sess = self.ctx.session.lock();
                *sess = sess.next_session();
            }
            self.ctx.emit_stats();
        }
        self.spawn(job);
        self.ctx.log_info(match job {
            Job::Cycle { .. } => "Started",
            Job::Once { .. } => "Test run started",
            Job::Record { .. } => "Recording started",
        });
        true
    }

    pub fn stop(&self) {
        let _g = self.lock.lock();
        let was = self.is_running();
        self.ctx.set_running(false);
        self.reap();
        self.ctx.disarm_timer();
        self.ctx.release_inputs();
        if matches!(self.job(), Some(Job::Cycle { .. })) {
            self.ctx.session.lock().finish();
            self.ctx.emit_stats();
        }
        *self.ctx.job.lock() = None;
        self.ctx.set_state(BotState::Stopped, None);
        if was {
            self.ctx.log_info("Stopped");
        }
    }

    pub fn toggle(self: &Arc<Self>) {
        if self.is_running() {
            self.stop();
        } else {
            self.start(Job::Cycle { from: StartFrom::First });
        }
    }

    pub fn record_toggle(self: &Arc<Self>, seq: Option<SeqId>) {
        match self.job() {
            Some(Job::Record { .. }) if self.is_running() => self.stop(),
            _ if self.is_running() => self.ctx.log_warn("Stop the macro before recording"),
            _ => {
                let seq = seq.unwrap_or(self.ctx.settings.read().recording.target);
                self.start(Job::Record { seq });
            }
        }
    }

    pub fn run_once(self: &Arc<Self>, seq: SeqId) {
        self.run_steps(seq, 0, false);
    }

    pub fn run_steps(self: &Arc<Self>, seq: SeqId, from: usize, only: bool) {
        if self.is_running() {
            self.ctx.log_warn("Stop the macro before a test run");
            return;
        }
        self.start(Job::Once { seq, from, only });
    }

    fn reap(&self) {
        let helpers: Vec<_> = self.helpers.lock().drain(..).collect();
        for h in helpers {
            join_unless_self(h);
        }
        if let Some(h) = self.thread.lock().take() {
            join_unless_self(h);
        }
    }

    fn spawn(self: &Arc<Self>, job: Job) {
        self.ctx.set_running(true);
        self.ctx.touch();
        let ctx = Arc::clone(&self.ctx);
        let h = std::thread::Builder::new()
            .name("bot-loop".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| machine::run(&ctx, job)));
                ctx.release_inputs();
                if result.is_err() {
                    ctx.log_error("Bot loop panicked; stopped");
                }
                if ctx.running() {
                    ctx.set_running(false);
                    ctx.disarm_timer();
                    if let Job::Cycle { .. } = job {
                        ctx.session.lock().finish();
                        ctx.emit_stats();
                    }
                    *ctx.job.lock() = None;
                    ctx.set_state(BotState::Stopped, None);
                }
            })
            .expect("spawn bot loop");
        *self.thread.lock() = Some(h);

        if let Job::Cycle { .. } = job {
            let ctx = Arc::clone(&self.ctx);
            let timer = std::thread::Builder::new()
                .name("bot-timer".into())
                .spawn(move || timer_watch::run(&ctx))
                .expect("spawn timer watcher");
            let mut helpers = self.helpers.lock();
            helpers.push(timer);
            if self.ctx.settings.read().watchdog.enabled {
                let ctx = Arc::clone(&self.ctx);
                let weak: Weak<Bot> = Arc::downgrade(self);
                let on_stuck: Box<dyn FnOnce(String) + Send> = Box::new(move |reason| {
                    if let Some(bot) = weak.upgrade() {
                        std::thread::spawn(move || bot.restart_loop(&reason));
                    }
                });
                let wd = std::thread::Builder::new()
                    .name("bot-watchdog".into())
                    .spawn(move || watchdog::run(&ctx, on_stuck))
                    .expect("spawn watchdog");
                helpers.push(wd);
            }
        }
    }

    pub fn restart_loop(self: &Arc<Self>, reason: &str) {
        let _g = self.lock.lock();
        if !self.is_running() {
            return;
        }
        let attempt = {
            let mut s = self.ctx.session.lock();
            s.restarts += 1;
            s.restarts
        };
        let (max, backoff) = {
            let s = self.ctx.settings.read();
            (s.watchdog.max_restarts, s.watchdog.restart_backoff_s)
        };
        self.ctx.emit(BotEvent::Recovery { attempt, reason: reason.to_string() });
        if attempt > max {
            self.ctx.log_error(&format!("Restart limit reached ({max}); stopping"));
            drop(_g);
            self.stop();
            return;
        }
        self.ctx.log_warn(&format!("Restarting #{attempt}: {reason}; leaving to lobby"));
        self.ctx.set_state(BotState::Recovering, Some(reason.to_string()));
        self.ctx.set_running(false);
        self.reap();
        self.ctx.disarm_timer();
        self.ctx.release_inputs();
        std::thread::sleep(std::time::Duration::from_secs_f32(backoff));
        let job = Job::Cycle { from: StartFrom::Leave };
        *self.ctx.job.lock() = Some(job);
        self.spawn(job);
        self.ctx.emit_stats();
    }
}
