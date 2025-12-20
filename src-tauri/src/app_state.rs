use std::sync::Mutex;
use std::time::Instant;

pub struct AppState {
    pub active: bool,
    pub osk_open: bool,
    pub toggle_guard: bool,
    pub last_toggle_time: Option<Instant>,
    pub target_hwnd: isize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active: false,
            osk_open: false,
            toggle_guard: false,
            last_toggle_time: None,
            target_hwnd: 0,
        }
    }
}

pub type SharedAppState = Mutex<AppState>;
