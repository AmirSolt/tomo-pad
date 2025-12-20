mod osk;
mod system;

pub use osk::{OskState, update_osk_stick};
pub use system::update_mouse;

use enigo::Enigo;
use gilrs::Event;
use tauri::AppHandle;

pub fn handle_input(
    event: &Event,
    active: bool,
    osk_open: bool,
    app: &AppHandle,
    enigo: &mut Enigo,
) {
    if !active {
        return;
    }

    if osk_open {
        osk::handle_osk_input(event, app, enigo);
        return;
    } else {
        system::handle_system_input(event, app, enigo);
        return;
    }
}
