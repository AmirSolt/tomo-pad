mod app_state;
mod commands;
mod funcs;
mod gamepad;
mod input_mapper;
mod setup;
mod tray;

use app_state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .setup(setup::init)
        .invoke_handler(tauri::generate_handler![commands::greet, commands::send_key])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
