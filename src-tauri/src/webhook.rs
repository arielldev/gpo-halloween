use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
use serde_json::{json, Value};

use crate::config::Settings;
use crate::events::{LogLevel, LogLine, Stats};

pub const DISCORD_INVITE: &str = "https://discord.gg/unPZxXAtfb";

const COLOR_ORANGE: u32 = 0xFF7A1A;
const COLOR_GREEN: u32 = 0x22C55E;
const COLOR_RED: u32 = 0xEF4444;
const COLOR_GREY: u32 = 0x6B7280;

pub struct WebhookQueue {
    tx: Sender<Value>,
    settings: Arc<RwLock<Settings>>,
}

impl WebhookQueue {
    pub fn start(settings: Arc<RwLock<Settings>>) -> Arc<Self> {
        let (tx, rx) = unbounded::<Value>();
        let s2 = Arc::clone(&settings);
        std::thread::Builder::new()
            .name("webhook".into())
            .spawn(move || worker(rx, s2))
            .expect("spawn webhook worker");
        Arc::new(Self { tx, settings })
    }

    fn enabled(&self) -> bool {
        let s = self.settings.read();
        s.webhook.enabled && valid_url(&s.webhook.url)
    }

    fn send(&self, embed: Value) {
        if self.enabled() {
            let _ = self.tx.send(json!({ "username": "GPO Halloween", "embeds": [embed] }));
        }
    }

    pub fn test(&self) -> Result<(), String> {
        let url = self.settings.read().webhook.url.trim().to_string();
        if !valid_url(&url) {
            return Err("Paste a Discord webhook URL (https://discord.com/api/webhooks/…)".into());
        }
        post(
            &url,
            &json!({
                "username": "GPO Halloween",
                "embeds": [embed("🎃 Webhook connected", &format!("GPO Halloween can post to this channel.\n\n{}", community()), COLOR_GREEN, vec![])]
            }),
        )
    }

    pub fn progress(&self, stats: &Stats, logs: &[LogLine]) {
        let every = self.settings.read().webhook.every_routes;
        let mut fields = stat_fields(stats);
        if let Some(l) = log_field(logs) {
            fields.push(l);
        }
        self.send(embed(
            &format!("🎃 Progress · {} routes this run", stats.laps),
            &format!("Update every {every} routes.\n\n{}", community()),
            COLOR_ORANGE,
            fields,
        ));
    }

    pub fn started(&self, order: &str) {
        if self.settings.read().webhook.start_stop {
            self.send(embed("▶️ Macro started", &format!("Run order: **{order}** (repeats)\n\n{}", community()), COLOR_GREEN, vec![]));
        }
    }

    pub fn stopped(&self, stats: &Stats, logs: &[LogLine]) {
        if !self.settings.read().webhook.start_stop {
            return;
        }
        let mut fields = stat_fields(stats);
        if let Some(l) = log_field(logs) {
            fields.push(l);
        }
        self.send(embed("⏹️ Macro stopped", &community(), COLOR_GREY, fields));
    }

    pub fn error(&self, msg: &str) {
        if self.settings.read().webhook.errors {
            self.send(embed("⚠️ Macro needs attention", &format!("{msg}\n\n{}", community()), COLOR_RED, vec![]));
        }
    }
}

pub fn valid_url(url: &str) -> bool {
    let u = url.trim();
    ["https://discord.com/api/webhooks/", "https://discordapp.com/api/webhooks/", "https://canary.discord.com/api/webhooks/", "https://ptb.discord.com/api/webhooks/"]
        .iter()
        .any(|p| u.starts_with(p))
}

fn community() -> String {
    format!("💬 **Join the community:** {DISCORD_INVITE}")
}

fn stat_fields(st: &Stats) -> Vec<Value> {
    let t = &st.total;
    vec![
        field("🏠 Houses", &format!("{}  ·  all time {}", st.houses, t.houses)),
        field("🔁 Routes", &format!("{}  ·  all time {}", st.laps, t.laps)),
        field("🚪 Rejoins", &format!("{}  ·  all time {}", st.cycles, t.cycles)),
        field("🛍️ Shop trips", &format!("{}  ·  all time {}", st.buys, t.buys)),
        field("⏱️ Timer stops", &format!("{}  ·  all time {}", st.timer_trips, t.timer_trips)),
        field("🕒 Time farming", &format!("{}  ·  all time {}", fmt_runtime(st.runtime_s), fmt_runtime(t.runtime_s))),
    ]
}

fn log_field(logs: &[LogLine]) -> Option<Value> {
    let lines: Vec<String> = logs
        .iter()
        .rev()
        .filter(|l| l.level != LogLevel::Debug)
        .take(8)
        .map(|l| {
            let tag = match l.level {
                LogLevel::Warn => "! ",
                LogLevel::Error => "x ",
                _ => "",
            };
            let mut m = format!("{tag}{}", l.msg);
            if m.len() > 110 {
                m.truncate(107);
                m.push_str("...");
            }
            m
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    let mut body: Vec<String> = lines.into_iter().rev().collect();
    while body.join("\n").len() > 980 {
        body.remove(0);
    }
    Some(json!({ "name": "📜 Recent activity", "value": format!("```\n{}\n```", body.join("\n")), "inline": false }))
}

fn embed(title: &str, desc: &str, color: u32, fields: Vec<Value>) -> Value {
    json!({
        "title": title,
        "description": desc,
        "color": color,
        "fields": fields,
        "footer": { "text": format!("GPO Halloween v{}", env!("CARGO_PKG_VERSION")) },
        "timestamp": iso_now(),
    })
}

fn field(name: &str, value: &str) -> Value {
    json!({ "name": name, "value": value, "inline": true })
}

fn fmt_runtime(s: u64) -> String {
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn iso_now() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86400) as i64);
    let rem = secs % 86400;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, (rem % 3600) / 60, rem % 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn post(url: &str, body: &Value) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;
    let resp = client.post(url.trim()).json(body).send().map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Discord returned {}", resp.status().as_u16()))
    }
}

fn worker(rx: Receiver<Value>, settings: Arc<RwLock<Settings>>) {
    for body in rx.iter() {
        let url = settings.read().webhook.url.clone();
        if !valid_url(&url) {
            continue;
        }
        let mut delay = Duration::from_secs(1);
        for _ in 0..3 {
            if post(&url, &body).is_ok() {
                break;
            }
            std::thread::sleep(delay);
            delay *= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_urls_are_validated() {
        assert!(valid_url("https://discord.com/api/webhooks/123/abc"));
        assert!(valid_url(" https://discordapp.com/api/webhooks/1/x "));
        assert!(!valid_url("https://example.com/hook"));
        assert!(!valid_url(""));
    }

    #[test]
    fn date_is_iso() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20367), (2025, 10, 6));
        assert!(iso_now().ends_with('Z'));
    }

    #[test]
    fn log_field_fits_discord_limits() {
        let logs: Vec<LogLine> = (0..40).map(|i| LogLine { ts: 0, level: LogLevel::Info, msg: format!("line {i} {}", "x".repeat(200)) }).collect();
        let f = log_field(&logs).unwrap();
        assert!(f["value"].as_str().unwrap().len() <= 1024);
    }
}
