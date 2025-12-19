use crate::funcs;
use enigo::{Button as MouseButton, Coordinate, Direction, Enigo, Key, Keyboard, Mouse};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
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
        handle_osk_input(event, app);
        return;
    } else {
        handle_system_input(event, app, enigo);
        return;
    }
}

fn handle_osk_input(event: &Event, app: &AppHandle) {
    // Check for B button to close OSK
    if let EventType::ButtonPressed(Button::East, _) = event.event {
        funcs::close_osk(app);
        return;
    }
}

fn handle_system_input(event: &Event, app: &AppHandle, enigo: &mut Enigo) {
    let (btn, direction) = match event.event {
        EventType::ButtonPressed(b, _) => (b, Direction::Press),
        EventType::ButtonReleased(b, _) => (b, Direction::Release),
        _ => return,
    };

    match btn {
        Button::Start => {
            if direction == Direction::Press {
                funcs::open_osk(app);
            }
        }
        Button::South => {
            let _ = enigo.button(MouseButton::Left, direction);
        }
        Button::East => {
            let _ = enigo.button(MouseButton::Right, direction);
        }
        Button::DPadUp => {
            let _ = enigo.key(Key::UpArrow, direction);
        }
        Button::DPadDown => {
            let _ = enigo.key(Key::DownArrow, direction);
        }
        Button::DPadLeft => {
            let _ = enigo.key(Key::LeftArrow, direction);
        }
        Button::DPadRight => {
            let _ = enigo.key(Key::RightArrow, direction);
        }
        _ => {}
    }
}

pub fn update_mouse(gilrs: &Gilrs, enigo: &mut Enigo) {
    for (_id, gamepad) in gilrs.gamepads() {
        let axis_x = gamepad.value(Axis::LeftStickX);
        let axis_y = gamepad.value(Axis::LeftStickY);

        if axis_x.abs() > 0.1 || axis_y.abs() > 0.1 {
            // Scale sensitivity
            let scale = 10.0;
            let move_x = (axis_x * scale) as i32;
            let move_y = (-axis_y * scale) as i32; // Y is usually inverted on gamepads vs screen

            let _ = enigo.move_mouse(move_x, move_y, Coordinate::Rel);
        }
    }
}
