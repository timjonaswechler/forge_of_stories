use bevy::prelude::*;

/// UI color constants for buttons
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

/// Resource to track in-game menu state (ESC menu)
#[derive(Resource, Default)]
pub struct InGameMenuState {
    open: bool,
}

impl InGameMenuState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_closed(&self) -> bool {
        !self.open
    }

    pub fn set_open(&mut self) {
        self.open = true;
    }

    pub fn set_closed(&mut self) {
        self.open = false;
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
}
