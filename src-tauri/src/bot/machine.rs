use crate::config::{SeqId, StartFrom};
use crate::events::BotState;

use super::actions;
use super::ctx::Ctx;
use super::{recorder, Job};

pub fn run(ctx: &Ctx, job: Job) {
    ctx.log_debug("Loop thread started");
    match job {
        Job::Cycle { from } => cycle(ctx, from),
        Job::Once { seq, from, only } => once(ctx, seq, from, only),
        Job::Record { seq } => recorder::record(ctx, seq),
    }
}

fn preflight(ctx: &Ctx, seqs: &[SeqId]) -> bool {
    let s = ctx.settings.read();
    let problems: Vec<String> = seqs.iter().flat_map(|q| s.problems(*q)).collect();
    drop(s);
    for p in &problems {
        ctx.log_error(p);
    }
    if !problems.is_empty() {
        ctx.log_error("Fix the steps above in Binds, then start again");
        if matches!(*ctx.job.lock(), Some(Job::Cycle { .. })) {
            ctx.webhook.error(&format!("The macro could not start:\n• {}", problems.join("\n• ")));
        }
    }
    problems.is_empty()
}

fn once(ctx: &Ctx, seq: SeqId, from: usize, only: bool) {
    let whole = from == 0 && !only;
    if whole && !preflight(ctx, &[seq]) {
        return;
    }
    if !actions::wait_for_roblox(ctx) || !actions::ensure_front(ctx) {
        return;
    }
    let what = if only {
        format!("{} step {}", seq.label(), from + 1)
    } else if from > 0 {
        format!("{} from step {}", seq.label(), from + 1)
    } else {
        seq.label().to_string()
    };
    ctx.set_state(BotState::TestRun, Some(what.clone()));
    if whole && seq.walks() && !actions::prepare_route(ctx) {
        return;
    }
    ctx.set_state(BotState::TestRun, Some(what.clone()));
    let end = if only { Some(from + 1) } else { None };
    if actions::run_range(ctx, seq, from, end) {
        ctx.log_info(&format!("Test run of {what} finished"));
    }
}

fn start_index(order: &[SeqId], from: StartFrom) -> usize {
    let want = match from {
        StartFrom::First => return 0,
        StartFrom::Spawn => SeqId::Macro,
        StartFrom::Lobby => SeqId::Lobby,
        StartFrom::Leave => SeqId::Leave,
    };
    order.iter().position(|q| *q == want).unwrap_or(0)
}

fn order_text(order: &[SeqId]) -> String {
    order.iter().map(|q| q.label()).collect::<Vec<_>>().join(" → ")
}

fn cycle(ctx: &Ctx, from: StartFrom) {
    let (order, needed) = {
        let s = ctx.settings.read();
        (s.active_order(), s.needed_sequences())
    };
    if !preflight(ctx, &needed) {
        return;
    }
    if !actions::wait_for_roblox(ctx) || !actions::ensure_front(ctx) {
        return;
    }
    ctx.disarm_timer();
    ctx.clear_joined();
    ctx.log_info(&format!("Run order: {} (repeats)", order_text(&order)));
    ctx.webhook.started(&order_text(&order));
    let mut i = start_index(&order, from);
    if order[i] != SeqId::Lobby && order[i] != SeqId::Leave {
        ctx.mark_joined();
    }
    while ctx.running() {
        let seq = order[i % order.len()];
        let ok = run_block(ctx, seq);
        if !ctx.running() {
            return;
        }
        if !ok {
            ctx.release_inputs();
            let tripped = ctx.disarm_timer();
            if tripped {
                ctx.session.lock().timer_trips += 1;
            } else {
                ctx.log_warn(&format!("{} did not finish; leaving to lobby to reset", seq.label()));
                if !ctx.sleep_ms(1000) {
                    return;
                }
            }
            ctx.emit_stats();
            match order.iter().position(|q| *q == SeqId::Leave) {
                Some(li) if seq != SeqId::Leave => i = li,
                _ => {
                    if !leave_now(ctx) {
                        return;
                    }
                    i = order.iter().position(|q| *q == SeqId::Lobby).unwrap_or(i + 1);
                }
            }
            continue;
        }
        i = (i + 1) % order.len();
    }
}

fn leave_now(ctx: &Ctx) -> bool {
    ctx.disarm_timer();
    ctx.set_state(BotState::LeavingToLobby, None);
    let ok = actions::run_sequence(ctx, SeqId::Leave);
    if ok {
        after_leave(ctx);
    }
    ok || !ctx.running()
}

fn after_leave(ctx: &Ctx) {
    ctx.disarm_timer();
    ctx.clear_joined();
    ctx.session.lock().cycles += 1;
    ctx.emit_stats();
}

fn run_block(ctx: &Ctx, seq: SeqId) -> bool {
    match seq {
        SeqId::Leave => {
            ctx.disarm_timer();
            ctx.set_state(BotState::LeavingToLobby, None);
            let ok = actions::run_sequence(ctx, SeqId::Leave);
            if ok {
                after_leave(ctx);
            }
            ok
        }
        SeqId::Lobby => {
            ctx.disarm_timer();
            ctx.set_state(BotState::Rejoining, None);
            actions::run_sequence(ctx, SeqId::Lobby) && {
                ctx.mark_joined();
                ctx.arm_timer();
                let wait = ctx.settings.read().server.after_join_wait_ms;
                ctx.set_state(BotState::WaitingForSpawn, Some(format!("{:.0}s for the server to load", wait as f32 / 1000.0)));
                ctx.sleep_ms(wait)
            }
        }
        SeqId::Macro | SeqId::Buy => {
            ctx.arm_timer();
            let (every, laps) = {
                let s = ctx.settings.read();
                (s.buy.every, ctx.session.lock().laps)
            };
            let shop = seq == SeqId::Buy || (every > 0 && laps > 0 && laps % every == 0 && ctx.session.lock().buys < laps / every);
            let target = if shop { SeqId::Buy } else { SeqId::Macro };
            let delay = ctx.settings.read().route.start_delay_ms;
            if !(actions::prepare_route(ctx) && ctx.sleep_ms(delay)) {
                return false;
            }
            ctx.set_state(BotState::Farming, Some(target.label().into()));
            if shop {
                ctx.log_info(&format!("Buy run (after {laps} routes)"));
            }
            if !actions::run_sequence(ctx, target) {
                return false;
            }
            {
                let mut s = ctx.session.lock();
                if shop {
                    s.buys += 1;
                } else {
                    s.laps += 1;
                }
            }
            ctx.emit_stats();
            if !shop {
                let every = ctx.settings.read().webhook.every_routes;
                let stats = ctx.session.lock().stats();
                if every > 0 && stats.laps > 0 && stats.laps % every == 0 {
                    ctx.webhook.progress(&stats, &ctx.recent_logs());
                }
            }
            true
        }
    }
}
