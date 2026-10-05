use std::time::{Duration, Instant};

use crate::events::{Lifetime, Stats};

#[derive(Debug)]
pub struct Session {
    pub buys: u32,
    pub houses: u32,
    pub laps: u32,
    pub cycles: u32,
    pub timer_trips: u32,
    pub restarts: u32,
    started: Instant,
    ended: Option<Instant>,
    base: Lifetime,
}

impl Session {
    pub fn new() -> Self {
        Self::with_base(Lifetime::default())
    }

    pub fn with_base(base: Lifetime) -> Self {
        let now = Instant::now();
        Self { buys: 0, houses: 0, laps: 0, cycles: 0, timer_trips: 0, restarts: 0, started: now, ended: Some(now), base }
    }

    pub fn lifetime(&self) -> Lifetime {
        Lifetime {
            buys: self.base.buys + self.buys,
            houses: self.base.houses + self.houses,
            laps: self.base.laps + self.laps,
            cycles: self.base.cycles + self.cycles,
            timer_trips: self.base.timer_trips + self.timer_trips,
            runtime_s: self.base.runtime_s + self.runtime().as_secs(),
            sessions: self.base.sessions,
        }
    }

    pub fn next_session(&self) -> Session {
        let mut base = self.lifetime();
        base.sessions += 1;
        let mut s = Session::with_base(base);
        s.ended = None;
        s
    }

    pub fn finish(&mut self) {
        if self.ended.is_none() {
            self.ended = Some(Instant::now());
        }
    }

    pub fn runtime(&self) -> Duration {
        self.ended.unwrap_or_else(Instant::now).duration_since(self.started)
    }

    pub fn stats(&self) -> Stats {
        Stats {
            buys: self.buys,
            houses: self.houses,
            laps: self.laps,
            cycles: self.cycles,
            timer_trips: self.timer_trips,
            runtime_s: self.runtime().as_secs(),
            restarts: self.restarts,
            total: self.lifetime(),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_accumulates_across_sessions() {
        let mut s = Session::new().next_session();
        s.houses = 4;
        s.cycles = 1;
        s.finish();
        let next = s.next_session();
        assert_eq!(next.lifetime().houses, 4);
        assert_eq!(next.lifetime().sessions, 2);
        assert_eq!(next.houses, 0);
    }
}
