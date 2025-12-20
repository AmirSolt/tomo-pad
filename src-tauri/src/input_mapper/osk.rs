use crate::funcs;
use enigo::Enigo;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use tauri::{AppHandle, Emitter};

pub struct OskState {
    pub stick_active_x: i32, // 0, 1, -1
    pub stick_active_y: i32,
}

impl Default for OskState {
    fn default() -> Self {
        Self { stick_active_x: 0, stick_active_y: 0 }
    }
}

pub fn handle_osk_input(event: &Event, app: &AppHandle, _enigo: &mut Enigo) {
    let (btn, phase) = match event.event {
        EventType::ButtonPressed(b, _) => (b, "down"),
        EventType::ButtonReleased(b, _) => (b, "up"),
        _ => return,
    };

    match btn {
        Button::Start => {
            if phase == "down" {
                funcs::close_osk(app);
            }
        }
        Button::Select => {
            if phase == "down" {
                let _ = app.emit("osk:nav:shift", ());
            }
        }
        Button::East => {
            if phase == "up" {
                funcs::close_osk(app);
            }
        }
        Button::South => {
            let _ = app.emit("osk:nav:select", serde_json::json!({
                "phase": phase,
                "ts": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
            }));
        }
        Button::DPadUp => emit_move(app, phase, 0, -1),
        Button::DPadDown => emit_move(app, phase, 0, 1),
        Button::DPadLeft => emit_move(app, phase, -1, 0),
        Button::DPadRight => emit_move(app, phase, 1, 0),
        _ => {}
    }
}

fn emit_move(app: &AppHandle, phase: &str, dx: i32, dy: i32) {
    let _ = app.emit("osk:nav:move", serde_json::json!({
        "phase": phase,
        "dx": dx,
        "dy": dy,
        "source": "gamepad",
        "magnitude": 1.0,
        "ts": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }));
}

pub fn update_osk_stick(gilrs: &Gilrs, app: &AppHandle, state: &mut OskState) {
    for (_id, gamepad) in gilrs.gamepads() {
        let x = gamepad.value(Axis::LeftStickX);
        let y = gamepad.value(Axis::LeftStickY);
        let threshold = 0.5;

        // X Axis
        let new_x_dir = if x > threshold { 1 } else if x < -threshold { -1 } else { 0 };
        if new_x_dir != state.stick_active_x {
            // State changed
            if state.stick_active_x != 0 {
                // Stop previous direction
                emit_move(app, "up", state.stick_active_x, 0);
            }
            if new_x_dir != 0 {
                // Start new direction
                emit_move(app, "down", new_x_dir, 0);
            }
            state.stick_active_x = new_x_dir;
        }

        // Y Axis
        // Note: Gamepad Y is usually -1 up, 1 down? Or inverted?
        // Gilrs: "Value of axis. -1.0 to 1.0."
        // Usually Up is -1.0.
        let new_y_dir = if y > threshold { -1 } else if y < -threshold { 1 } else { 0 };
        if new_y_dir != state.stick_active_y {
            if state.stick_active_y != 0 {
                emit_move(app, "up", 0, state.stick_active_y);
            }
            if new_y_dir != 0 {
                emit_move(app, "down", 0, new_y_dir);
            }
            state.stick_active_y = new_y_dir;
        }
    }
}
