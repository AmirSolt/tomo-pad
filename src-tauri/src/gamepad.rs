use crate::app_state::SharedAppState;
use crate::input_mapper;
use enigo::{Enigo, Settings};
use gilrs::{Button, EventType, Gilrs};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub fn init_gamepad_listener(app: AppHandle) {
    thread::spawn(move || {
        let mut gilrs = match Gilrs::new() {
            Ok(g) => {
                println!("Gilrs initialized successfully");
                g
            }
            Err(e) => {
                eprintln!("Failed to init gilrs: {}", e);
                return;
            }
        };

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to init enigo: {:?}", e);
                return;
            }
        };

        loop {
            // Process all pending events
            while let Some(event) = gilrs.next_event() {
                // Suppress combo events
                if let EventType::ButtonPressed(btn, _) = event.event {
                    if btn == Button::Start || btn == Button::Select {
                        let gamepad = gilrs.gamepad(event.id);
                        if gamepad.is_pressed(Button::Start) && gamepad.is_pressed(Button::Select) {
                            continue;
                        }
                    }
                }

                let state_handle = app.state::<SharedAppState>();
                let (active, osk_open) = {
                    let state = state_handle.lock().unwrap();
                    (state.active, state.osk_open)
                };

                // Forward to mapper (lock is released now)
                input_mapper::handle_input(&event, active, osk_open, &app, &mut enigo);
            }

            // Check combo on all connected gamepads
            let state_handle = app.state::<SharedAppState>();
            let mut should_close_osk = false;
            let mut run_mouse_update = false;

            {
                let mut state = state_handle.lock().unwrap();
                let mut combo_pressed = false;

                for (_id, gamepad) in gilrs.gamepads() {
                    let start = gamepad.is_pressed(Button::Start);
                    let select = gamepad.is_pressed(Button::Select);

                    if start || select {
                        // println!("Gamepad {}: Start={}, Select={}", id, start, select);
                    }

                    if start && select {
                        combo_pressed = true;
                        break;
                    }
                }

                if combo_pressed {
                    if !state.toggle_guard {
                        state.active = !state.active;
                        state.toggle_guard = true;
                        state.last_toggle_time = Some(Instant::now());

                        // Emit active changed event
                        let _ = app.emit("app_active_changed", state.active);
                        println!("Active toggled: {}", state.active);

                        if !state.active {
                            state.osk_open = false;
                            let _ = app.emit("osk_visibility_changed", false);
                            should_close_osk = true;
                        }
                    }
                } else {
                    // Reset guard if combo is NOT pressed
                    state.toggle_guard = false;
                }

                if state.active {
                    run_mouse_update = true;
                }
            }

            if should_close_osk {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            if run_mouse_update {
                input_mapper::update_mouse(&gilrs, &mut enigo);
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
