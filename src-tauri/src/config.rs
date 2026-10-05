use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::types::{MouseButton, RelPoint, RelRect};

pub const SETTINGS_VERSION: u32 = 8;
pub const CODE_PLACEHOLDER: &str = "{code}";
const BUNDLED_DEFAULTS: &str = include_str!("../defaults.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeqId {
    Lobby,
    Macro,
    Leave,
    Buy,
}

impl SeqId {
    pub fn label(self) -> &'static str {
        match self {
            SeqId::Lobby => "Lobby",
            SeqId::Macro => "Macro",
            SeqId::Leave => "Leave to lobby",
            SeqId::Buy => "Buy",
        }
    }

    pub fn walks(self) -> bool {
        matches!(self, SeqId::Macro | SeqId::Buy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBlock {
    pub seq: SeqId,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

impl RunBlock {
    pub fn new(seq: SeqId) -> Self {
        Self { seq, enabled: true }
    }
}

pub fn default_run_order() -> Vec<RunBlock> {
    vec![RunBlock::new(SeqId::Lobby), RunBlock::new(SeqId::Macro), RunBlock::new(SeqId::Leave)]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Buy {
    pub steps: Vec<Step>,
    pub every: u32,
}

impl Default for Buy {
    fn default() -> Self {
        Self { steps: buy_template(), every: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Click {
        point: Option<RelPoint>,
        #[serde(default)]
        button: MouseButton,
    },
    Key {
        key: String,
        #[serde(default = "default_hold_ms")]
        hold_ms: u32,
    },
    Type {
        text: String,
    },
    Paste {
        text: String,
    },
    Scroll {
        amount: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        point: Option<RelPoint>,
    },
    Interact,
    Wait,
}

fn default_hold_ms() -> u32 {
    60
}

fn yes() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub action: Action,
    #[serde(default)]
    pub wait_ms: u32,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

impl Step {
    pub fn new(action: Action, wait_ms: u32, note: &str) -> Self {
        Self { action, wait_ms, enabled: true, note: note.to_string() }
    }

    pub fn unset_point(&self) -> bool {
        self.enabled && matches!(self.action, Action::Click { point: None, .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Server {
    pub code: String,
    pub after_join_wait_ms: u32,
}

impl Default for Server {
    fn default() -> Self {
        Self { code: String::new(), after_join_wait_ms: 20000 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keys {
    pub interact: String,
    pub equip: String,
}

impl Default for Keys {
    fn default() -> Self {
        Self { interact: "e".into(), equip: "6".into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Setup {
    pub click_to_move: bool,
    pub spawn_set: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub enabled: bool,
    pub out_steps: u32,
    pub in_steps: u32,
    pub step_delay_ms: u32,
    pub tilt_px: i32,
    pub settle_ms: u32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { enabled: true, out_steps: 40, in_steps: 0, step_delay_ms: 25, tilt_px: 600, settle_ms: 600 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Sequence {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnFinish {
    Rejoin,
    Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartFrom {
    Spawn,
    Lobby,
    Leave,
    First,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Route {
    pub steps: Vec<Step>,
    pub house_wait_ms: u32,
    pub interact_hold_ms: u32,
    pub start_delay_ms: u32,
    pub on_finish: OnFinish,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            house_wait_ms: 5000,
            interact_hold_ms: 100,
            start_delay_ms: 1000,
            on_finish: OnFinish::Rejoin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Elapsed,
    Remaining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Timer {
    pub enabled: bool,
    pub region: RelRect,
    pub mode: TimerMode,
    pub stop_at_s: u32,
    pub confirm_reads: u32,
    pub interval_ms: u32,
    pub ocr_scale: u32,
    pub fallback: bool,
    pub fallback_after_misses: u32,
    pub fallback_stop_s: u32,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            enabled: true,
            region: RelRect { x: 0.86, y: 0.92, w: 0.13, h: 0.07 },
            mode: TimerMode::Elapsed,
            stop_at_s: 290,
            confirm_reads: 3,
            interval_ms: 1000,
            ocr_scale: 3,
            fallback: true,
            fallback_after_misses: 5,
            fallback_stop_s: 280,
        }
    }
}

impl Timer {
    pub fn reached(&self, secs: u32) -> bool {
        match self.mode {
            TimerMode::Elapsed => secs >= self.stop_at_s,
            TimerMode::Remaining => secs <= self.stop_at_s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordMode {
    Replace,
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Recording {
    pub target: SeqId,
    pub mode: RecordMode,
    pub record_keys: bool,
    pub camera_first: bool,
    pub collapse_typing: bool,
    pub round_ms: u32,
}

impl Default for Recording {
    fn default() -> Self {
        Self { target: SeqId::Macro, mode: RecordMode::Replace, record_keys: true, camera_first: true, collapse_typing: true, round_ms: 50 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub toggle: String,
    pub overlay: String,
    pub quit: String,
    pub hide_hud: String,
    pub record: String,
    pub run_leave: String,
    pub run_lobby: String,
    pub run_macro: String,
}

impl Hotkeys {
    pub fn sanitize(&mut self) {
        let d = Hotkeys::default();
        let fix = |v: &mut String, def: String| {
            let bad = ["Media", "Audio", "Launch", "Browser", "Volume"].iter().any(|p| v.contains(p));
            if bad {
                *v = def;
            }
        };
        fix(&mut self.toggle, d.toggle);
        fix(&mut self.overlay, d.overlay);
        fix(&mut self.quit, d.quit);
        fix(&mut self.hide_hud, d.hide_hud);
        fix(&mut self.record, d.record);
        fix(&mut self.run_leave, d.run_leave);
        fix(&mut self.run_lobby, d.run_lobby);
        fix(&mut self.run_macro, d.run_macro);
    }
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle: "F1".into(),
            overlay: "F2".into(),
            quit: "F3".into(),
            hide_hud: "F4".into(),
            record: "F5".into(),
            run_leave: "F6".into(),
            run_lobby: "F7".into(),
            run_macro: "F8".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ui {
    pub hud_offset: RelPoint,
    pub hud_visible: bool,
    pub panel_offset: RelPoint,
    pub panel_size: [u32; 2],
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            hud_offset: RelPoint { x: 0.5, y: 0.0 },
            hud_visible: true,
            panel_offset: RelPoint { x: 1.0, y: 0.5 },
            panel_size: [460, 680],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Watchdog {
    pub enabled: bool,
    pub heartbeat_timeout_s: f32,
    pub max_restarts: u32,
    pub restart_backoff_s: f32,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self { enabled: true, heartbeat_timeout_s: 30.0, max_restarts: 5, restart_backoff_s: 3.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub setup: Setup,
    pub server: Server,
    pub keys: Keys,
    pub camera: Camera,
    pub lobby: Sequence,
    #[serde(rename = "macro")]
    pub route: Route,
    pub leave: Sequence,
    pub buy: Buy,
    pub run_order: Vec<RunBlock>,
    pub timer: Timer,
    pub recording: Recording,
    pub hotkeys: Hotkeys,
    pub ui: Ui,
    pub watchdog: Watchdog,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            setup: Setup::default(),
            server: Server::default(),
            keys: Keys::default(),
            camera: Camera::default(),
            lobby: Sequence { steps: lobby_template() },
            route: Route::default(),
            leave: Sequence { steps: leave_template() },
            buy: Buy::default(),
            run_order: default_run_order(),
            timer: Timer::default(),
            recording: Recording::default(),
            hotkeys: Hotkeys::default(),
            ui: Ui::default(),
            watchdog: Watchdog::default(),
        }
    }
}

fn click(point: Option<RelPoint>, wait_ms: u32, note: &str) -> Step {
    Step::new(Action::Click { point, button: MouseButton::Left }, wait_ms, note)
}

pub fn lobby_template() -> Vec<Step> {
    vec![
        click(Some(RelPoint { x: 0.5, y: 0.5 }), 2500, "Click to proceed (opens the menu)"),
        click(None, 2000, "Private server join"),
        click(None, 1000, "Private server code box"),
        Step::new(Action::Paste { text: CODE_PLACEHOLDER.into() }, 2000, "Pastes your server code"),
        click(None, 2000, REGULAR_CONFIRM_NOTE),
        click(None, 2000, REGULAR_NOTE),
        click(None, 2000, "First Sea server (joins)"),
    ]
}

fn walk(x: f32, y: f32, note: &str) -> Step {
    Step::new(Action::Click { point: Some(RelPoint { x, y }), button: MouseButton::Right }, 1000, note)
}

pub fn buy_template() -> Vec<Step> {
    vec![
        walk(0.504_166_66, 0.777_006_9, "Walk to the shop NPC (1/6)"),
        walk(0.503_125, 0.777_006_9, "Walk to the shop NPC (2/6)"),
        walk(0.504_166_66, 0.777_006_9, "Walk to the shop NPC (3/6)"),
        walk(0.502_604_2, 0.779_980_2, "Walk to the shop NPC (4/6)"),
        walk(0.478_125, 0.796_828_57, "Walk to the shop NPC (5/6)"),
        walk(0.498_437_5, 0.758_176_4, "At the shop NPC (6/6)"),
        Step::new(Action::Scroll { amount: 4, point: None }, 200, "Scroll the shop list (pick where the list is)"),
        click(None, 1000, "The item to buy"),
        click(None, 1500, "Accept"),
    ]
}

pub fn leave_template() -> Vec<Step> {
    vec![
        click(None, 800, "Menu toggle (opens the accordion)"),
        click(None, 0, "Main menu"),
        Step::new(Action::Wait, 6000, "Wait for the title screen"),
    ]
}

const REGULAR_NOTE: &str = "Game type: Regular";
const REGULAR_CONFIRM_NOTE: &str = "Game type: Regular (confirms the pasted code)";

fn double_regular(steps: &mut Vec<Step>) {
    let Some(i) = steps.iter().position(|s| s.note == REGULAR_NOTE && matches!(s.action, Action::Click { .. })) else { return };
    if i > 0 && steps[i - 1].note == REGULAR_CONFIRM_NOTE {
        return;
    }
    let mut first = steps[i].clone();
    first.note = REGULAR_CONFIRM_NOTE.into();
    steps.insert(i, first);
}

fn untouched_template(steps: &[Step]) -> bool {
    steps.iter().all(|s| !matches!(s.action, Action::Click { point: Some(_), .. }))
}

impl Settings {
    pub fn factory() -> Settings {
        match serde_json::from_str::<Settings>(BUNDLED_DEFAULTS) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("defaults.json parse failed ({e}); using built-in defaults");
                Settings::default()
            }
        }
    }

    pub fn steps(&self, seq: SeqId) -> &Vec<Step> {
        match seq {
            SeqId::Lobby => &self.lobby.steps,
            SeqId::Macro => &self.route.steps,
            SeqId::Leave => &self.leave.steps,
            SeqId::Buy => &self.buy.steps,
        }
    }

    pub fn steps_mut(&mut self, seq: SeqId) -> &mut Vec<Step> {
        match seq {
            SeqId::Lobby => &mut self.lobby.steps,
            SeqId::Macro => &mut self.route.steps,
            SeqId::Leave => &mut self.leave.steps,
            SeqId::Buy => &mut self.buy.steps,
        }
    }

    pub fn active_order(&self) -> Vec<SeqId> {
        let mut order: Vec<SeqId> = self.run_order.iter().filter(|b| b.enabled).map(|b| b.seq).collect();
        if order.is_empty() {
            order = default_run_order().into_iter().map(|b| b.seq).collect();
        }
        order
    }

    pub fn needed_sequences(&self) -> Vec<SeqId> {
        let mut need: Vec<SeqId> = Vec::new();
        let mut extra = self.active_order();
        if self.timer.enabled {
            extra.extend([SeqId::Leave, SeqId::Lobby]);
        }
        if self.buy.every > 0 && extra.contains(&SeqId::Macro) {
            extra.push(SeqId::Buy);
        }
        for q in extra {
            if !need.contains(&q) {
                need.push(q);
            }
        }
        need
    }

    pub fn problems(&self, seq: SeqId) -> Vec<String> {
        let mut out = Vec::new();
        let steps = self.steps(seq);
        if !steps.iter().any(|s| s.enabled) {
            out.push(format!("{} has no steps yet", seq.label()));
        }
        for (i, s) in steps.iter().enumerate() {
            if s.unset_point() {
                out.push(format!("{} step {} has no click point", seq.label(), i + 1));
            }
            if let Action::Key { key, .. } = &s.action {
                if s.enabled && crate::core::types::Key::parse(key).is_none() {
                    out.push(format!("{} step {} has an unknown key \"{key}\"", seq.label(), i + 1));
                }
            }
        }
        if seq == SeqId::Lobby {
            if self.server.code.trim().is_empty() {
                out.push("Private server code is empty".into());
            }
            let code = self.server.code.trim();
            let enters_code = |text: &str| text.contains(CODE_PLACEHOLDER) || (!code.is_empty() && text.trim() == code);
            if !steps.iter().any(|s| s.enabled && matches!(&s.action, Action::Type { text } | Action::Paste { text } if enters_code(text))) {
                out.push("Lobby never enters the server code (add a Paste step with {code})".into());
            }
        }
        out
    }
}

pub const SEQUENCE_FILE_KIND: &str = "gpo-halloween-sequence";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceFile {
    pub kind: String,
    pub version: u32,
    pub seq: SeqId,
    pub steps: Vec<Step>,
}

pub fn parse_sequence(json: &str) -> Result<(Option<SeqId>, Vec<Step>), String> {
    if let Ok(f) = serde_json::from_str::<SequenceFile>(json) {
        return Ok((Some(f.seq), f.steps));
    }
    if let Ok(steps) = serde_json::from_str::<Vec<Step>>(json) {
        return Ok((None, steps));
    }
    if let Ok(seq) = serde_json::from_str::<Sequence>(json) {
        if !seq.steps.is_empty() {
            return Ok((None, seq.steps));
        }
    }
    Err("Not a sequence file. Export one from Binds first.".into())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid preset name")]
    BadName,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    pub fn export_path(&self) -> PathBuf {
        self.dir.join("settings-export.json")
    }

    fn presets_dir(&self) -> PathBuf {
        self.dir.join("presets")
    }

    fn stats_path(&self) -> PathBuf {
        self.dir.join("stats.json")
    }

    pub fn load_stats(&self) -> crate::events::Lifetime {
        fs::read_to_string(self.stats_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_stats(&self, l: &crate::events::Lifetime) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.stats_path(), &serde_json::to_vec_pretty(l)?)
    }

    pub fn load(&self) -> Settings {
        let mut settings = match fs::read_to_string(self.settings_path()) {
            Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_else(|e| {
                tracing::warn!("settings parse failed ({e}); using defaults");
                Settings::factory()
            }),
            Err(_) => Settings::factory(),
        };
        if settings.version < SETTINGS_VERSION {
            if settings.version < 2 {
                if untouched_template(&settings.lobby.steps) {
                    settings.lobby.steps = lobby_template();
                }
                if untouched_template(&settings.leave.steps) {
                    settings.leave.steps = leave_template();
                }
            }
            if settings.version < 3 {
                if settings.camera.tilt_px == 0 {
                    settings.camera.tilt_px = Camera::default().tilt_px;
                }
                settings.camera.out_steps = settings.camera.out_steps.max(Camera::default().out_steps);
            }
            if settings.version < 4 {
                double_regular(&mut settings.lobby.steps);
            }
            if settings.version < 5 {
                settings.hotkeys.sanitize();
            }
            if settings.version < 7 {
                settings.route.on_finish = OnFinish::Rejoin;
                settings.timer.enabled = true;
            }
            if settings.version < 8 && untouched_template(&settings.buy.steps) {
                settings.buy.steps = buy_template();
            }
            settings.version = SETTINGS_VERSION;
            let _ = self.save(&settings);
        }
        settings
    }

    pub fn save(&self, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.settings_path(), &serde_json::to_vec_pretty(s)?)
    }

    pub fn export(&self, s: &Settings) -> Result<PathBuf, ConfigError> {
        fs::create_dir_all(&self.dir)?;
        let path = self.export_path();
        write_atomic(&path, &serde_json::to_vec_pretty(s)?)?;
        Ok(path)
    }

    pub fn export_sequence(&self, seq: SeqId, steps: &[Step]) -> Result<PathBuf, ConfigError> {
        let dir = self.dir.join("sequences");
        fs::create_dir_all(&dir)?;
        let name = match seq {
            SeqId::Lobby => "lobby",
            SeqId::Macro => "macro",
            SeqId::Leave => "leave-to-lobby",
            SeqId::Buy => "buy",
        };
        let path = dir.join(format!("{name}-{}.json", crate::events::now_ms() / 1000));
        let file = SequenceFile { kind: SEQUENCE_FILE_KIND.into(), version: SETTINGS_VERSION, seq, steps: steps.to_vec() };
        write_atomic(&path, &serde_json::to_vec_pretty(&file)?)?;
        Ok(path)
    }

    pub fn list_presets(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(self.presets_dir()) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "json") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out
    }

    fn preset_path(&self, name: &str) -> Result<PathBuf, ConfigError> {
        let ok = !name.is_empty()
            && name.len() <= 64
            && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ');
        if !ok {
            return Err(ConfigError::BadName);
        }
        Ok(self.presets_dir().join(format!("{name}.json")))
    }

    pub fn save_preset(&self, name: &str, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(self.presets_dir())?;
        write_atomic(&self.preset_path(name)?, &serde_json::to_vec_pretty(s)?)
    }

    pub fn load_preset(&self, name: &str) -> Result<Settings, ConfigError> {
        let s = fs::read_to_string(self.preset_path(name)?)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn delete_preset(&self, name: &str) -> Result<(), ConfigError> {
        fs::remove_file(self.preset_path(name)?)?;
        Ok(())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_defaults() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, SETTINGS_VERSION);
        assert_eq!(back.lobby.steps, s.lobby.steps);
        assert!(j.contains("\"macro\""));
    }

    #[test]
    #[ignore = "regenerates defaults.json from Settings::default()"]
    fn write_defaults_json() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults.json");
        let mut json = serde_json::to_string_pretty(&Settings::default()).unwrap();
        json.push('\n');
        fs::write(path, json).unwrap();
    }

    #[test]
    fn bundled_defaults_parse() {
        let parsed: Settings = serde_json::from_str(BUNDLED_DEFAULTS).expect("defaults.json must be valid");
        assert_eq!(parsed.version, SETTINGS_VERSION);
    }

    #[test]
    fn partial_json_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{"macro":{"house_wait_ms":4000}}"#).unwrap();
        assert_eq!(s.route.house_wait_ms, 4000);
        assert_eq!(s.route.interact_hold_ms, 100);
        assert_eq!(s.timer.stop_at_s, 290);
    }

    #[test]
    fn step_json_shape() {
        let st: Step = serde_json::from_str(r#"{"kind":"click","point":{"x":0.5,"y":0.25},"wait_ms":1200}"#).unwrap();
        assert_eq!(st.action, Action::Click { point: Some(RelPoint { x: 0.5, y: 0.25 }), button: MouseButton::Left });
        assert_eq!(st.wait_ms, 1200);
        let st: Step = serde_json::from_str(r#"{"kind":"interact"}"#).unwrap();
        assert_eq!(st.action, Action::Interact);
        let st: Step = serde_json::from_str(r#"{"kind":"key","key":"Escape"}"#).unwrap();
        assert_eq!(st.action, Action::Key { key: "Escape".into(), hold_ms: 60 });
        let j = serde_json::to_string(&Step::new(Action::Wait, 500, "")).unwrap();
        assert_eq!(j, r#"{"kind":"wait","wait_ms":500}"#);
    }

    #[test]
    fn migration_swaps_only_unpicked_templates() {
        let dir = std::env::temp_dir().join(format!("gpo-halloween-mig-{}", std::process::id()));
        let store = Store::new(dir.clone());
        let mut old = Settings::default();
        old.version = 1;
        old.lobby.steps = vec![click(None, 0, "old")];
        old.leave.steps = vec![click(Some(RelPoint { x: 0.1, y: 0.2 }), 0, "picked by user")];
        store.save(&old).unwrap();
        let s = store.load();
        assert_eq!(s.lobby.steps, lobby_template());
        assert_eq!(s.leave.steps[0].note, "picked by user");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn literal_server_code_counts_as_entering_it() {
        let mut s = Settings::default();
        s.server.code = "ABC123xyz".into();
        for st in s.lobby.steps.iter_mut() {
            if let Action::Click { point, .. } = &mut st.action {
                *point = Some(RelPoint { x: 0.5, y: 0.5 });
            }
            if let Action::Paste { text } = &mut st.action {
                *text = "ABC123xyz".into();
            }
        }
        assert!(s.problems(SeqId::Lobby).is_empty());
        for st in s.lobby.steps.iter_mut() {
            if let Action::Paste { text } = &mut st.action {
                *text = "someothercode".into();
            }
        }
        assert!(s.problems(SeqId::Lobby).iter().any(|m| m.contains("never enters")));
    }

    #[test]
    fn migration_v3_sets_camera_view_and_equip() {
        let dir = std::env::temp_dir().join(format!("gpo-halloween-mig3-{}", std::process::id()));
        let store = Store::new(dir.clone());
        let json = r#"{"version":2,"keys":{"interact":"e"},"camera":{"enabled":true,"out_steps":30,"tilt_px":0}}"#;
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings.json"), json).unwrap();
        let s = store.load();
        assert_eq!(s.camera.tilt_px, 600);
        assert_eq!(s.camera.out_steps, 40);
        assert_eq!(s.keys.equip, "6");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn migration_v4_clicks_regular_twice_with_same_point() {
        let dir = std::env::temp_dir().join(format!("gpo-halloween-mig4-{}", std::process::id()));
        let store = Store::new(dir.clone());
        let mut old = Settings::default();
        old.version = 3;
        old.lobby.steps = vec![
            Step::new(Action::Paste { text: CODE_PLACEHOLDER.into() }, 800, ""),
            click(Some(RelPoint { x: 0.38, y: 0.51 }), 1000, REGULAR_NOTE),
            click(Some(RelPoint { x: 0.42, y: 0.52 }), 0, "First Sea server (joins)"),
        ];
        store.save(&old).unwrap();
        let s = store.load();
        assert_eq!(s.lobby.steps.len(), 4);
        assert_eq!(s.lobby.steps[1].action, s.lobby.steps[2].action);
        assert_eq!(s.lobby.steps[1].wait_ms, 1000);
        assert_eq!(s.lobby.steps[1].note, REGULAR_CONFIRM_NOTE);
        let again = store.load();
        assert_eq!(again.lobby.steps.len(), 4);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn media_key_hotkeys_reset_to_defaults() {
        let mut h = Hotkeys { run_lobby: "MediaPlayPause".into(), ..Hotkeys::default() };
        h.run_macro = "F9".into();
        h.sanitize();
        assert_eq!(h.run_lobby, "F7");
        assert_eq!(h.run_macro, "F9");
    }

    #[test]
    fn sequence_files_round_trip_and_accept_plain_arrays() {
        let s = Settings::default();
        let dir = std::env::temp_dir().join(format!("gpo-halloween-seq-{}", std::process::id()));
        let store = Store::new(dir.clone());
        let path = store.export_sequence(SeqId::Lobby, &s.lobby.steps).unwrap();
        let (seq, steps) = parse_sequence(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(seq, Some(SeqId::Lobby));
        assert_eq!(steps, s.lobby.steps);
        let (seq, steps) = parse_sequence(r#"[{"kind":"interact"}]"#).unwrap();
        assert_eq!(seq, None);
        assert_eq!(steps.len(), 1);
        assert!(parse_sequence(r#"{"hello":1}"#).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn run_order_defaults_and_needed_sequences() {
        let mut s = Settings::default();
        assert_eq!(s.active_order(), vec![SeqId::Lobby, SeqId::Macro, SeqId::Leave]);
        assert!(!s.needed_sequences().contains(&SeqId::Buy));
        s.buy.every = 3;
        assert!(s.needed_sequences().contains(&SeqId::Buy));
        s.run_order = vec![RunBlock::new(SeqId::Macro)];
        s.timer.enabled = true;
        let need = s.needed_sequences();
        assert!(need.contains(&SeqId::Leave) && need.contains(&SeqId::Lobby));
        s.run_order.iter_mut().for_each(|b| b.enabled = false);
        assert_eq!(s.active_order().len(), 3);
        let j = serde_json::to_string(&Settings::default().run_order).unwrap();
        assert_eq!(j, r#"[{"seq":"lobby"},{"seq":"macro"},{"seq":"leave"}]"#);
    }

    #[test]
    fn scroll_point_is_optional_in_json() {
        let st: Step = serde_json::from_str(r#"{"kind":"scroll","amount":-3}"#).unwrap();
        assert_eq!(st.action, Action::Scroll { amount: -3, point: None });
        let j = serde_json::to_string(&Step::new(Action::Scroll { amount: 2, point: None }, 0, "")).unwrap();
        assert!(!j.contains("point"));
        assert!(!buy_template().iter().any(|s| matches!(s.action, Action::Key { .. })));
    }

    #[test]
    fn disabled_steps_do_not_block_start() {
        let mut s = Settings::default();
        s.leave.steps = vec![click(Some(RelPoint { x: 0.1, y: 0.1 }), 0, "menu"), click(None, 0, "Confirm")];
        assert!(!s.problems(SeqId::Leave).is_empty());
        s.leave.steps[1].enabled = false;
        assert!(s.problems(SeqId::Leave).is_empty());
        let j = serde_json::to_string(&s.leave.steps).unwrap();
        assert_eq!(j.matches("\"enabled\":false").count(), 1);
        assert!(!j.contains("\"enabled\":true"));
    }

    #[test]
    fn problems_flag_unset_points_and_code() {
        let s = Settings::default();
        let p = s.problems(SeqId::Lobby);
        assert!(p.iter().any(|m| m.contains("no click point")));
        assert!(p.iter().any(|m| m.contains("server code is empty")));
        assert!(s.problems(SeqId::Macro).iter().any(|m| m.contains("no steps")));
    }
}
