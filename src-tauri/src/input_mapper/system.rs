use crate::funcs;
use enigo::{Axis as EnigoAxis, Button as MouseButton, Coordinate, Direction, Enigo, Key, Keyboard, Mouse};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use tauri::AppHandle;

pub fn handle_system_input(event: &Event, app: &AppHandle, enigo: &mut Enigo) {
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

#[derive(Default)]
pub struct MouseState {
    pub x_remainder: f32,
    pub y_remainder: f32,
    pub scroll_x_remainder: f32,
    pub scroll_y_remainder: f32,
}

pub fn update_mouse(gilrs: &Gilrs, enigo: &mut Enigo, state: &mut MouseState) {
    for (_id, gamepad) in gilrs.gamepads() {
        let axis_x = gamepad.value(Axis::LeftStickX);
        let axis_y = gamepad.value(Axis::LeftStickY);

        if axis_x.abs() > 0.1 || axis_y.abs() > 0.1 {
            let base_sens = 1.0;
            let accel_sens = 24.0;

            let raw_x = axis_x * base_sens + axis_x.powi(3) * accel_sens;
            let raw_y = -axis_y * base_sens + (-axis_y).powi(3) * accel_sens;

            let total_x = raw_x + state.x_remainder;
            let total_y = raw_y + state.y_remainder;

            let move_x = total_x as i32;
            let move_y = total_y as i32;

            state.x_remainder = total_x - move_x as f32;
            state.y_remainder = total_y - move_y as f32;

            if move_x != 0 || move_y != 0 {
                let _ = enigo.move_mouse(move_x, move_y, Coordinate::Rel);
            }
        } else {
            state.x_remainder = 0.0;
            state.y_remainder = 0.0;
        }

        let scroll_x = gamepad.value(Axis::RightStickX);
        let scroll_y = gamepad.value(Axis::RightStickY);

        if scroll_x.abs() > 0.1 || scroll_y.abs() > 0.1 {
            let base_scroll = 0.0;
            let accel_scroll = 1.02;

            let raw_sx = scroll_x * base_scroll + scroll_x.powi(3) * accel_scroll;
            let raw_sy = -scroll_y * base_scroll + (-scroll_y).powi(3) * accel_scroll;

            let total_sx = raw_sx + state.scroll_x_remainder;
            let total_sy = raw_sy + state.scroll_y_remainder;

            let s_x = total_sx as i32;
            let s_y = total_sy as i32;

            state.scroll_x_remainder = total_sx - s_x as f32;
            state.scroll_y_remainder = total_sy - s_y as f32;

            if s_x != 0 {
                let _ = enigo.scroll(s_x, EnigoAxis::Horizontal);
            }
            if s_y != 0 {
                let _ = enigo.scroll(s_y, EnigoAxis::Vertical);
            }
        } else {
            state.scroll_x_remainder = 0.0;
            state.scroll_y_remainder = 0.0;
        }
    }
}
