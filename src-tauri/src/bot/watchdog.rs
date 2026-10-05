use std::time::Duration;

use crate::events::BotState;

use super::ctx::Ctx;

pub fn max_state_age(s: BotState, after_join_wait_ms: u32) -> Duration {
    Duration::from_secs(match s {
        BotState::LeavingToLobby => 180,
        BotState::Rejoining => 180,
        BotState::WaitingForSpawn => after_join_wait_ms as u64 / 1000 + 90,
        BotState::CameraSetup => 60,
        BotState::Recovering => 60,
        BotState::Farming
        | BotState::TestRun
        | BotState::Recording
        | BotState::WaitingForRoblox
        | BotState::Stopped => 0,
    })
}

pub enum Verdict {
    Ok,
    Stuck(String),
}

pub fn check(ctx: &Ctx) -> Verdict {
    let (hb, join_wait) = {
        let s = ctx.settings.read();
        (Duration::from_secs_f32(s.watchdog.heartbeat_timeout_s), s.server.after_join_wait_ms)
    };
    let age = ctx.heartbeat_age();
    if age > hb {
        return Verdict::Stuck(format!("no heartbeat for {}s", age.as_secs()));
    }
    let state = ctx.state();
    let max = max_state_age(state, join_wait);
    if !max.is_zero() && ctx.state_age() > max {
        return Verdict::Stuck(format!("{state:?} exceeded {}s", max.as_secs()));
    }
    Verdict::Ok
}

pub fn run(ctx: &Ctx, on_stuck: Box<dyn FnOnce(String) + Send>) {
    while ctx.running() {
        std::thread::sleep(Duration::from_secs(2));
        if !ctx.running() {
            return;
        }
        if let Verdict::Stuck(reason) = check(ctx) {
            ctx.log_error(&format!("Watchdog: {reason}"));
            on_stuck(reason);
            return;
        }
    }
}
