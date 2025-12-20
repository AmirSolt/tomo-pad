use crate::app_state::SharedAppState;
use crate::input_mapper;
use enigo::{Enigo, Settings};
use gilrs::{Button, EventType, Gilrs};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

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

        let mut osk_state = input_mapper::OskState::default();
        let mut mouse_state = input_mapper::MouseState::default();

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
            let mut run_mouse_update = false;
            let mut run_osk_update = false;

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

            let mut should_toggle = false;
            let mut was_active = false;

            {
                let mut state = state_handle.lock().unwrap();

                if combo_pressed {
                    if !state.toggle_guard {
                        state.toggle_guard = true;
                        state.last_toggle_time = Some(Instant::now());
                        should_toggle = true;
                        was_active = state.active;
                    }
                } else {
                    // Reset guard if combo is NOT pressed
                    state.toggle_guard = false;
                }

                if !should_toggle && state.active {
                    if state.osk_open {
                        run_osk_update = true;
                    } else {
                        run_mouse_update = true;
                    }
                }
            }

            if should_toggle {
                if was_active {
                    crate::funcs::deactivate(&app);
                } else {
                    crate::funcs::activate(&app);
                }

                let state = state_handle.lock().unwrap();
                if state.active {
                    if state.osk_open {
                        run_osk_update = true;
                    } else {
                        run_mouse_update = true;
                    }
                }
            }

            if run_mouse_update {
                input_mapper::update_mouse(&gilrs, &mut enigo, &mut mouse_state);
            } else if run_osk_update {
                input_mapper::update_osk_stick(&gilrs, &app, &mut osk_state);
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
