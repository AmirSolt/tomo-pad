use crate::app_state::SharedAppState;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(serde::Deserialize)]
pub struct KeyPayload {
    phase: String,
    key: Option<String>,
    scan_code: Option<u16>,
    text: Option<String>,
    modifiers: Option<Vec<String>>,
}

#[tauri::command]
pub fn send_key(app_handle: tauri::AppHandle, state: tauri::State<SharedAppState>, payload: KeyPayload) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE,
            KEYEVENTF_EXTENDEDKEY, SendInput, VIRTUAL_KEY,
        };
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};
        use windows::Win32::Foundation::HWND;
        use raw_window_handle::HasWindowHandle;
        use std::mem::size_of;

        // Safety: Check if we are stealing focus and restore target
        let current_fg = unsafe { GetForegroundWindow() };
        let target = state.lock().unwrap().target_hwnd;
        
        if let Some(window) = app_handle.get_webview_window("main") {
             if let Ok(handle) = window.window_handle() {
                 let raw = handle.as_raw();
                 if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = raw {
                     let my_hwnd = win32_handle.hwnd.get();
                     if current_fg.0 as isize == my_hwnd {
                         // We are focused! Switch back to target.
                         if target != 0 {
                             unsafe { let _ = SetForegroundWindow(HWND(target as _)); }
                             // Small delay to allow focus switch
                             std::thread::sleep(std::time::Duration::from_millis(10));
                         }
                     } else if target != 0 && (current_fg.0 as isize) != target {
                         // Focus changed to another window. Update target?
                         // Or just send to whatever is foreground (default behavior of SendInput).
                         // If we want to stick to target, we should switch.
                         // But usually user wants to type where they clicked.
                         // So we update our target to current foreground.
                         state.lock().unwrap().target_hwnd = current_fg.0 as isize;
                     }
                 }
             }
        }

        let mut inputs = Vec::new();
        let mut flags = KEYEVENTF_SCANCODE;
        if payload.phase == "up" {
            flags |= KEYEVENTF_KEYUP;
        }

        // Helper to create input for a scan code
        let create_input = |sc: u16, flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS| -> INPUT {
            let mut key_flags = flags;
            if (sc & 0xFF00) == 0xE000 {
                key_flags |= KEYEVENTF_EXTENDEDKEY;
            }
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: sc & 0xFF,
                        dwFlags: key_flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        };

        // Handle modifiers
        if let Some(modifiers) = &payload.modifiers {
            for modifier in modifiers {
                let sc = match modifier.as_str() {
                    "shift" => 0x2A,
                    "ctrl" => 0x1D,
                    "alt" => 0x38,
                    "win" => 0xE05B,
                    _ => 0,
                };
                if sc != 0 {
                    // If phase is down, press modifier first.
                    // If phase is up, release modifier last (so we push to inputs later? No, inputs are executed in order).
                    // Wait, if phase is up, we want KeyUp Key, then KeyUp Modifier.
                    // If phase is down, we want KeyDown Modifier, then KeyDown Key.
                    
                    let mod_flags = if payload.phase == "up" { KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP } else { KEYEVENTF_SCANCODE };
                    
                    if payload.phase == "down" {
                        inputs.push(create_input(sc, mod_flags));
                    }
                }
            }
        }

        if let Some(sc) = payload.scan_code {
            inputs.push(create_input(sc, flags));
        } else if let Some(key_str) = payload.key {
             let sc = match key_str.as_str() {
                 "{enter}" => 0x1C,
                 "{bksp}" => 0x0E,
                 "{space}" => 0x39,
                 "{tab}" => 0x0F,
                 "{esc}" => 0x01,
                 "{shift}" => 0x2A,
                 "{lock}" => 0x3A,
                 "{arrowup}" => 0xE048,
                 "{arrowdown}" => 0xE050,
                 "{arrowleft}" => 0xE04B,
                 "{arrowright}" => 0xE04D,
                 _ => 0
             };
             
             if sc != 0 {
                inputs.push(create_input(sc, flags));
             }
        } else if let Some(text) = payload.text {
             if payload.phase == "down" || payload.phase == "repeat" {
                 for c in text.encode_utf16() {
                     let input_down = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    inputs.push(input_down);
                    
                    let input_up = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    inputs.push(input_up);
                 }
             }
        }

        // Handle modifiers release (if phase is up)
        if let Some(modifiers) = &payload.modifiers {
            if payload.phase == "up" {
                for modifier in modifiers.iter().rev() { // Release in reverse order?
                    let sc = match modifier.as_str() {
                        "shift" => 0x2A,
                        "ctrl" => 0x1D,
                        "alt" => 0x38,
                        "win" => 0xE05B,
                        _ => 0,
                    };
                    if sc != 0 {
                        let mod_flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                        inputs.push(create_input(sc, mod_flags));
                    }
                }
            }
        }

        if !inputs.is_empty() {
            unsafe {
                SendInput(&inputs, size_of::<INPUT>() as i32);
            }
        }
    }
}
