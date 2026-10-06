use serde::{Deserialize, Serialize};

use crate::config::{SeqId, Settings};
use crate::core::types::WindowInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotState {
    Stopped,
    WaitingForRoblox,
    LeavingToLobby,
    Rejoining,
    WaitingForSpawn,
    CameraSetup,
    Farming,
    TestRun,
    Recording,
    Recovering,
}

impl BotState {
    pub fn is_active(self) -> bool {
        !matches!(self, BotState::Stopped)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Lifetime {
    pub buys: u32,
    pub houses: u32,
    pub laps: u32,
    pub cycles: u32,
    pub timer_trips: u32,
    pub runtime_s: u64,
    pub sessions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub buys: u32,
    pub houses: u32,
    pub laps: u32,
    pub cycles: u32,
    pub timer_trips: u32,
    pub runtime_s: u64,
    pub restarts: u32,
    pub total: Lifetime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogLine {
    pub ts: u64,
    pub level: LogLevel,
    pub msg: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerSource {
    Ocr,
    Clock,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerReading {
    pub seconds: Option<u32>,
    pub source: TimerSource,
    pub raw: String,
    pub armed: bool,
    pub tripped: bool,
    #[serde(default)]
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BotEvent {
    State { state: BotState, detail: Option<String> },
    Stats(Stats),
    Log(LogLine),
    Progress { seq: SeqId, index: usize, total: usize },
    Timer(TimerReading),
    Recording { seq: SeqId, steps: usize },
    Recovery { attempt: u32, reason: String },
    Roblox(Option<WindowInfo>),
    SettingsChanged(Box<Settings>),
}

impl BotEvent {
    pub fn channel(&self) -> &'static str {
        match self {
            BotEvent::State { .. } => "bot:state",
            BotEvent::Stats(_) => "bot:stats",
            BotEvent::Log(_) => "bot:log",
            BotEvent::Progress { .. } => "bot:progress",
            BotEvent::Timer(_) => "bot:timer",
            BotEvent::Recording { .. } => "bot:recording",
            BotEvent::Recovery { .. } => "bot:recovery",
            BotEvent::Roblox(_) => "roblox:changed",
            BotEvent::SettingsChanged(_) => "settings:changed",
        }
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
