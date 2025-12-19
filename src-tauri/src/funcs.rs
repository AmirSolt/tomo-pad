use crate::app_state::SharedAppState;
use tauri::{AppHandle, Emitter, Manager};

pub fn open_osk(app: &AppHandle) {
    let state_handle = app.state::<SharedAppState>();
    if let Ok(mut state) = state_handle.lock() {
        state.osk_open = true;
    }
    let _ = app.emit("osk_visibility_changed", true);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn close_osk(app: &AppHandle) {
    let state_handle = app.state::<SharedAppState>();
    if let Ok(mut state) = state_handle.lock() {
        state.osk_open = false;
    }
    let _ = app.emit("osk_visibility_changed", false);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
