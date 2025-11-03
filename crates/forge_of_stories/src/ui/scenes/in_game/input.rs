//! InGame Input Layer
//!
//! Handles input processing for the InGame scene.
mod player_movement;

use super::cameras::cursor::CursorState;
use crate::{
    GameState,
    ui::{components::InGameMenuState, scenes::in_game::input::player_movement::PlayerInputPlugin},
    utils::cleanup,
};
use app::LOG_MAIN;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for InGame input handling
pub(super) struct InGameInputPlugin;

impl Plugin for InGameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<InGameContext>()
            .add_plugins(PlayerInputPlugin)
            .add_systems(OnEnter(GameState::InGame), setup_input)
            .add_systems(
                Update,
                handle_menu_toggle.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>);
    }
}

/// Input context for InGame controls
#[derive(Component, Default)]
pub(super) struct InGameContext;

/// Sets up input context
fn setup_input(mut commands: Commands) {
    commands.spawn((Name::new("InGame Input Context"), InGameContext));

    info!(target: LOG_MAIN, "InGame input initialized");
}

/// Handles ESC key to toggle the in-game menu
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
