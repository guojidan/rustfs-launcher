mod commands;
mod config;
mod error;
mod process;
mod state;

use log;
use state::{add_app_log, set_app_handle, terminate_rustfs_process};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting RustFS Launcher");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            set_app_handle(app.handle().clone());
            add_app_log("RustFS Launcher started".to_string());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                println!("Main window shown and focused");
            } else {
                println!("Warning: Main window not found");
            }

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                add_app_log("Application closing, terminating RustFS process...".to_string());
                terminate_rustfs_process();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::launch_rustfs,
            commands::validate_config,
            commands::get_app_logs,
            commands::get_rustfs_logs,
            commands::diagnose_rustfs_binary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
