use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Emitter,
    image::Image,
};
use crate::funcs;
use crate::app_state::SharedAppState;

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> 
where
    AppHandle<R>: Manager<tauri::Wry>,
{
    let show_osk_i = MenuItem::with_id(app, "show_osk", "Show OSK", true, None::<&str>)?;
    let hide_osk_i = MenuItem::with_id(app, "hide_osk", "Hide OSK", true, None::<&str>)?;
    let toggle_active_i = MenuItem::with_id(app, "toggle_active", "Toggle Active", true, None::<&str>)?;
    let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_osk_i, &hide_osk_i, &toggle_active_i, &exit_i])?;

    // Assuming icon.ico exists in icons folder
    let icon = Image::from_bytes(include_bytes!("../icons/icon.ico")).expect("Failed to load icon");

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show_osk" => {
                    funcs::open_osk(app);
                }
                "hide_osk" => {
                    funcs::close_osk(app);
                }
                "toggle_active" => {
                    let state_handle = app.state::<SharedAppState>();
                    let mut state = state_handle.lock().unwrap();
                    state.active = !state.active;
                    let _ = app.emit("app_active_changed", state.active);
                }
                "exit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
             if let TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
             }
        })
        .build(app);

    Ok(())
}
