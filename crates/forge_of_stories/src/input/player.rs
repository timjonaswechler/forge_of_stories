//! Player movement input handling.
//!
//! Sends player input (WASD, Space) to the server via bevy_replicon events.

use crate::GameState;
use crate::networking::LocalPlayer;
use crate::ui::scenes::in_game::cameras::CursorState;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_server::PlayerInput;

/// Plugin for player movement input.
///
/// Registers the PlayerInput client event and sends input to the server
/// when the player is in-game, connected, and the cursor is locked.
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<PlayerInput>(Channel::Unreliable)
            .add_systems(
                Update,
                send_player_input
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(ClientState::Connected))
                    .run_if(resource_equals(CursorState::LOCKED)),
            );
    }
}

/// System that collects WASD + Space input and sends it to the server.
fn send_player_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    _local_player: Query<&Transform, With<LocalPlayer>>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    // Normalize direction
    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    // Send input to server
    commands.client_trigger(PlayerInput {
        direction,
        jump: keyboard.pressed(KeyCode::Space),
    });
}
