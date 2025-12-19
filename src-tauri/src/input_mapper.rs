use crate::funcs;
use enigo::{Axis as EnigoAxis, Button as MouseButton, Coordinate, Direction, Enigo, Key, Keyboard, Mouse};
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

    let (btn, direction) = match event.event {
        EventType::ButtonPressed(b, _) => (b, Direction::Press),
        EventType::ButtonReleased(b, _) => (b, Direction::Release),
        _ => return,
    };

    match btn {
        Button::Start => {
            if direction == Direction::Press {
                if osk_open{
                    funcs::close_osk(app);
                }else{
                    funcs::open_osk(app);
                }
            }
        }
        Button::RightTrigger2 => {
            let _ = enigo.button(MouseButton::Left, direction);
        }
        Button::LeftTrigger2 => {
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
            let scale = 18.0;
            let move_x = (axis_x * scale) as i32;
            let move_y = (-axis_y * scale) as i32; // Y is usually inverted on gamepads vs screen

            let _ = enigo.move_mouse(move_x, move_y, Coordinate::Rel);
        }

        let scroll_x = gamepad.value(Axis::RightStickX);
        let scroll_y = gamepad.value(Axis::RightStickY);

        if scroll_x.abs() > 0.1 || scroll_y.abs() > 0.1 {
            let scroll_scale = 1.0;
            let s_x = (scroll_x * scroll_scale) as i32;
            let s_y = (scroll_y * scroll_scale * -1.0)  as i32;

            if s_x != 0 {
                let _ = enigo.scroll(s_x, EnigoAxis::Horizontal);
            }
            if s_y != 0 {
                let _ = enigo.scroll(s_y, EnigoAxis::Vertical);
            }
        }
    }
}
