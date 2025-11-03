//! Menu input handling.
//!
//! Handles ESC key to toggle the in-game menu and manage cursor state.

use crate::GameState;
use crate::ui::components::InGameMenuState;
use crate::ui::scenes::in_game::cameras::CursorState;
use bevy::prelude::*;

/// Plugin for menu input handling.
///
/// Toggles the in-game menu when ESC is pressed and manages cursor lock state.
pub struct MenuInputPlugin;

impl Plugin for MenuInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_menu_toggle.run_if(in_state(GameState::InGame)),
        );
    }
}

/// Handles ESC key to toggle the in-game menu.
fn handle_menu_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cursor: ResMut<CursorState>,
    mut menu: ResMut<InGameMenuState>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        menu.toggle();
        *cursor = if menu.is_open() {
            CursorState::FREE
        } else {
            CursorState::LOCKED
        };
    }
}
