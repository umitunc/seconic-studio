#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc_commands;
mod local_ws_bridge;
mod scanner_manager;

use local_ws_bridge::run_local_ws_bridge;
use scanner_manager::ScannerManagerState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

fn main() {
    let state = ScannerManagerState::new();
    let ws_rx = state.event_tx.subscribe();

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            tauri::async_runtime::spawn(run_local_ws_bridge(ws_rx));

            let show_item = MenuItem::with_id(app, "show", "Pencereyi Göster", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Tepsiye Küçült", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip(format!(
                    "Sekonic Studio v{} — Hazır (ws://127.0.0.1:17441)",
                    env!("CARGO_PKG_VERSION")
                ))
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                tray_builder = tray_builder.icon(icon);
            }
            let _tray = tray_builder.build(app)?;

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            ipc_commands::list_available_ports,
            ipc_commands::get_bridge_status,
            ipc_commands::toggle_simulator_mode,
            ipc_commands::start_scan,
            ipc_commands::stop_scan,
            ipc_commands::convert_legacy_bldm,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("tauri run error: {e}"));
}
