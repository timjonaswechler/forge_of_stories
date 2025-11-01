//! Cursor grab/visibility management
//!
//! Scenes can request cursor state changes without managing window queries.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// Resource tracking desired cursor state
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CursorState {
    pub grab_mode: CursorGrabMode,
    pub visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self::FREE // Cursor sichtbar und nicht gelockt beim Start
    }
}

impl CursorState {
    pub const FREE: Self = Self {
        grab_mode: CursorGrabMode::None,
        visible: true,
    };

    pub const LOCKED: Self = Self {
        grab_mode: CursorGrabMode::Locked,
        visible: false,
    };

    pub const CONFINED: Self = Self {
        grab_mode: CursorGrabMode::Confined,
        visible: true,
    };
}

/// System that applies cursor state changes to the window
pub fn apply_cursor_state(
    cursor_state: Res<CursorState>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if !cursor_state.is_changed() {
        return;
    }

    let Ok((mut window, mut cursor)) = window_query.single_mut() else {
        return;
    };

    cursor.grab_mode = cursor_state.grab_mode;
    cursor.visible = cursor_state.visible;

    // Ensure window is focused when grabbing cursor
    if cursor_state.grab_mode != CursorGrabMode::None {
        window.focused = true;
    }
}

/// Helper to set cursor state (use from scenes)
pub fn set_cursor_free(mut cursor: ResMut<CursorState>) {
    *cursor = CursorState::FREE;
}

pub fn set_cursor_locked(mut cursor: ResMut<CursorState>) {
    *cursor = CursorState::LOCKED;
}

pub fn set_cursor_confined(mut cursor: ResMut<CursorState>) {
    *cursor = CursorState::CONFINED;
}
