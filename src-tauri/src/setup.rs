use crate::{funcs, gamepad, tray};
use tauri::{App, Manager};

pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tray
    tray::create_tray(app.handle())?;

    // Start gamepad listener
    gamepad::init_gamepad_listener(app.handle().clone());

    // Configure main window
    if let Some(window) = app.get_webview_window("main") {
        // Set always on top
        let _ = window.set_always_on_top(true);

        // Position at bottom right
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen_size = monitor.size();
            let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 800, height: 300 });
            
            const MARGIN: i32 = 50;

            let x = screen_size.width as i32 - window_size.width as i32 - MARGIN;
            let y = screen_size.height as i32 - window_size.height as i32 - MARGIN;
            
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        }

        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::HasWindowHandle;
            if let Ok(handle) = window.window_handle() {
                let raw = handle.as_raw();
                if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = raw {
                    use windows::Win32::Foundation::HWND;
                    use windows::Win32::UI::WindowsAndMessaging::{
                        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOMOVE,
                        SWP_NOSIZE, SWP_NOACTIVATE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
                    };

                    let hwnd = HWND(win32_handle.hwnd.get() as _);
                    unsafe {
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        let new_style = ex_style | (WS_EX_NOACTIVATE.0 as isize) | (WS_EX_TOOLWINDOW.0 as isize);
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
                        
                        let _ = SetWindowPos(
                            hwnd,
                            Some(HWND_TOPMOST),
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                    }
                }
            }
        }

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
}
