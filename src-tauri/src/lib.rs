pub mod app;
pub mod bot;
pub mod commands;
pub mod config;
pub mod core;
pub mod events;
pub mod hotkeys;
pub mod tray;
pub mod windows;

use tauri::Manager;

pub fn run() {
    let state = app::build_state();
    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            windows::show_panel(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let st = app.state::<app::AppState>();
            app::setup(&handle, &st)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "panel" {
                    api.prevent_close();
                    windows::hide_panel(window.app_handle());
                }
                if window.label() == "guide" {
                    api.prevent_close();
                    windows::hide_guide(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::bot_start,
            commands::bot_stop,
            commands::bot_toggle,
            commands::run_once,
            commands::record_toggle,
            commands::settings_get,
            commands::settings_set,
            commands::settings_reset,
            commands::settings_json,
            commands::settings_export,
            commands::settings_import,
            commands::sequence_export,
            commands::sequence_import,
            commands::sequences_dir,
            commands::preset_list,
            commands::preset_save,
            commands::preset_load,
            commands::preset_delete,
            commands::overlay_open,
            commands::overlay_commit,
            commands::overlay_cancel,
            commands::overlay_pending,
            commands::panel_placement_changed,
            commands::region_preview,
            commands::timer_test,
            commands::hud_set_offset,
            commands::hud_toggle,
            commands::panel_show,
            commands::panel_hide,
            commands::panel_toggle,
            commands::panel_visible,
            commands::guide_open,
            commands::guide_hide,
            commands::app_quit,
            commands::open_url,
            commands::open_path,
            commands::data_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                    if let Some(p) = app.get_webview_window("panel") {
                        let _ = p.hide();
                    }
                }
            }
        });
}
