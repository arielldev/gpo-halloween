use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use tauri::{AppHandle, Emitter, Manager};

use crate::bot::Bot;
use crate::config::{Settings, Store};
use crate::core::platform::{self, Platform};
use crate::core::types::WindowInfo;
use crate::events::BotEvent;
use crate::{hotkeys, tray, windows};

pub struct AppState {
    pub platform: Platform,
    pub settings: Arc<RwLock<Settings>>,
    pub roblox: Arc<RwLock<Option<WindowInfo>>>,
    pub store: Arc<Store>,
    pub bot: Arc<Bot>,
    pub overlay_session: Mutex<Option<serde_json::Value>>,
    pub panel_requested: AtomicBool,
    events_tx: Sender<BotEvent>,
    events_rx: Mutex<Option<Receiver<BotEvent>>>,
}

pub fn data_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("gpo-halloween")
}

pub fn build_state() -> AppState {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    init_logging(&dir);

    let store = Arc::new(Store::new(dir));
    let settings = Arc::new(RwLock::new(store.load()));
    let platform = platform::build();
    let roblox = Arc::new(RwLock::new(platform.window.find()));
    let (tx, rx) = unbounded::<BotEvent>();
    let bot = Bot::new(platform.clone(), Arc::clone(&settings), Arc::clone(&roblox), tx.clone(), Arc::clone(&store));

    AppState {
        platform,
        settings,
        roblox,
        store,
        bot,
        overlay_session: Mutex::new(None),
        panel_requested: AtomicBool::new(false),
        events_tx: tx,
        events_rx: Mutex::new(Some(rx)),
    }
}

pub fn setup(app: &AppHandle, st: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(rx) = st.events_rx.lock().take() {
        spawn_event_forwarder(app.clone(), rx);
    }
    spawn_roblox_watcher(st.platform.clone(), Arc::clone(&st.roblox), st.events_tx.clone());
    spawn_panel_fallback(app.clone());

    tray::build(app)?;
    if let Err(e) = hotkeys::register(app, &st.settings.read().hotkeys) {
        tracing::warn!("hotkeys: {e}");
    }
    windows::init(app);
    tracing::info!("GPO Halloween started");
    Ok(())
}

fn init_logging(dir: &std::path::Path) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let file = tracing_appender::rolling::daily(dir.join("logs"), "gpo-halloween.log");
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .try_init();
}

fn spawn_event_forwarder(app: AppHandle, rx: Receiver<BotEvent>) {
    std::thread::Builder::new()
        .name("event-forwarder".into())
        .spawn(move || {
            for ev in rx.iter() {
                match &ev {
                    BotEvent::SettingsChanged(s) => {
                        let _ = app.emit(ev.channel(), s.as_ref());
                        continue;
                    }
                    BotEvent::State { state, .. } => {
                        tray::update(&app, *state);
                        windows::on_bot_state(&app, state.is_active());
                    }
                    BotEvent::Roblox(info) => windows::on_roblox_changed(&app, *info),
                    _ => {}
                }
                let _ = app.emit(ev.channel(), &ev);
            }
        })
        .expect("spawn event forwarder");
}

fn spawn_panel_fallback(app: AppHandle) {
    std::thread::Builder::new()
        .name("panel-fallback".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(8));
            let st = app.state::<AppState>();
            if !st.panel_requested.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::warn!("frontend did not report ready; showing panel anyway");
                windows::show_panel(&app);
            }
        })
        .expect("spawn panel fallback");
}

fn spawn_roblox_watcher(platform: Platform, roblox: Arc<RwLock<Option<WindowInfo>>>, tx: Sender<BotEvent>) {
    std::thread::Builder::new()
        .name("roblox-watcher".into())
        .spawn(move || {
            let mut last: Option<WindowInfo> = None;
            let mut first = true;
            let mut unfocused_ticks = 0u32;
            loop {
                let mut now = platform.window.find();
                let lost_focus = matches!((last, now), (Some(l), Some(n)) if l.is_foreground && !n.is_foreground && l.client == n.client && l.visible == n.visible);
                if lost_focus {
                    unfocused_ticks += 1;
                    if unfocused_ticks < 4 {
                        now = last;
                    }
                } else {
                    unfocused_ticks = 0;
                }
                if first || now != last {
                    first = false;
                    *roblox.write() = now;
                    let _ = tx.send(BotEvent::Roblox(now));
                    last = now;
                }
                std::thread::sleep(Duration::from_millis(if now.is_some() { 50 } else { 500 }));
            }
        })
        .expect("spawn roblox watcher");
}
