#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc_commands;
mod local_ws_bridge;
mod scanner_manager;

use local_ws_bridge::run_local_ws_bridge;
use scanner_manager::ScannerManagerState;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
    let state = ScannerManagerState::new();
    let ws_rx = state.event_tx.subscribe();

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            tauri::async_runtime::spawn(run_local_ws_bridge(ws_rx));
            let _tray = TrayIconBuilder::new()
                .tooltip("Sekonic Studio — Hazır (ws://127.0.0.1:17441)")
                .build(app)?;
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
            }
            Ok(())
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
