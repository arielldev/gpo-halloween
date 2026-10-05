use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::app::AppState;
use crate::events::BotState;
use crate::windows;

const TRAY_ID: &str = "main";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Start / stop macro", true, None::<&str>)?;
    let panel = MenuItem::with_id(app, "panel", "Open / hide panel", true, None::<&str>)?;
    let hud = MenuItem::with_id(app, "hud", "Show / hide HUD", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &panel, &hud, &PredefinedMenuItem::separator(app)?, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().expect("tray icon"))
        .tooltip("GPO Halloween")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| {
            let st = app.state::<AppState>();
            match ev.id().as_ref() {
                "toggle" => {
                    let bot = std::sync::Arc::clone(&st.bot);
                    std::thread::spawn(move || bot.toggle());
                }
                "panel" => windows::toggle_panel(app),
                "hud" => {
                    let mut s = st.settings.read().clone();
                    s.ui.hud_visible = !s.ui.hud_visible;
                    let v = s.ui.hud_visible;
                    *st.settings.write() = s.clone();
                    let _ = st.store.save(&s);
                    windows::set_hud_visible(app, v);
                }
                "quit" => {
                    st.bot.stop();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                windows::show_panel(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

pub fn update(app: &AppHandle, state: BotState) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let label = match state {
        BotState::Stopped => "Idle",
        BotState::WaitingForRoblox => "Waiting for Roblox",
        BotState::Recording => "Recording",
        BotState::TestRun => "Test run",
        BotState::Recovering => "Recovering",
        BotState::LeavingToLobby | BotState::Rejoining | BotState::WaitingForSpawn => "Rejoining",
        _ => "Farming",
    };
    let _ = tray.set_tooltip(Some(format!("GPO Halloween: {label}")));
}
