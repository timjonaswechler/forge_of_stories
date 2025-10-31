//! Player movement input handling on the client side.

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::GameState;
use crate::ui::components::menu_allows_input;

/// Plugin for player movement input
pub(super) struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            send_player_input
                .run_if(in_state(GameState::InGame))
                .run_if(menu_allows_input),
        );
    }
}

/// Reads keyboard input and sends it to the server
fn send_player_input(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut forward = 0.0;
    let mut right = 0.0;

    // Forward/backward (W/S or Arrow Up/Down)
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        forward += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        forward -= 1.0;
    }

    // Left/right (A/D or Arrow Left/Right)
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        right += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        right -= 1.0;
    }

    // Jump
    let jump = keyboard.pressed(KeyCode::Space);

    // Create input vector (forward = -Z, right = +X in world space)
    let input_vec = Vec2::new(right, -forward);

    // Normalize if length > 1 (diagonal movement)
    let direction = if input_vec.length() > 1.0 {
        input_vec.normalize()
    } else {
        input_vec
    };

    // Only send if there's actual input
    if direction.length() > 0.01 || jump {
        commands.client_trigger(game_server::messages::PlayerInput { direction, jump });
    }
}
