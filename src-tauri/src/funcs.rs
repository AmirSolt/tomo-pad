use crate::app_state::SharedAppState;
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

pub fn open_osk(app: &AppHandle) {
    let state_handle = app.state::<SharedAppState>();

    #[cfg(target_os = "windows")]
    {
        unsafe {
            let hwnd = GetForegroundWindow();
            if let Ok(mut state) = state_handle.lock() {
                state.target_hwnd = hwnd.0 as isize;
            }
        }
    }

    if let Ok(mut state) = state_handle.lock() {
        state.osk_open = true;
    }
    let _ = app.emit("osk_visibility_changed", true);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        // let _ = window.set_focus();
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

pub fn activate(app: &AppHandle) {
    let state_handle = app.state::<SharedAppState>();
    if let Ok(mut state) = state_handle.lock() {
        state.active = true;
    }
    let _ = app.emit("app_active_changed", true);
    println!("Active toggled: true");

    open_osk(app)
}

pub fn deactivate(app: &AppHandle) {
    let state_handle = app.state::<SharedAppState>();
    if let Ok(mut state) = state_handle.lock() {
        state.active = false;
    }
    let _ = app.emit("app_active_changed", false);
    println!("Active toggled: false");

    close_osk(app)
}
