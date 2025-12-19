mod app_state;
mod gamepad;
mod input_mapper;
mod tray;
mod funcs;

use app_state::AppState;
use std::sync::Mutex;
use tauri::{Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .setup(|app| {
            // Initialize tray
            tray::create_tray(app.handle())?;

            // Start gamepad listener
            gamepad::init_gamepad_listener(app.handle().clone());

            // Configure main window
            if let Some(window) = app.get_webview_window("main") {
                // Set always on top
                let _ = window.set_always_on_top(true);
                
                // Hide initially
                let _ = window.hide();
                
                // Handle close event
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                        
                        let app = window_clone.app_handle();
                        funcs::close_osk(app);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
