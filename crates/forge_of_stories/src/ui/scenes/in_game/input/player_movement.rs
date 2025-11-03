//! Player movement input handling (local to client, sends to server)
use super::super::cameras::cursor::CursorState;
use crate::GameState;
use crate::networking::LocalPlayer;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_server::PlayerInput;

/// Plugin für Spieler-Movement-Input
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<PlayerInput>(Channel::Unreliable)
            .add_systems(
                Update,
                send_player_input
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(ClientState::Connected))
                    .run_if(resource_equals(CursorState::LOCKED)), // Nur wenn Cursor locked
            );
    }
}

/// System das WASD Input sammelt und an Server sendet
fn send_player_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    local_player: Query<&Transform, With<LocalPlayer>>,
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
    // let input = PlayerInput {
    //     direction,
    //     jump: keyboard.pressed(KeyCode::Space), // oder anderer Jump-Button
    // };

    commands.client_trigger(PlayerInput {
        direction,
        jump: keyboard.pressed(KeyCode::Space),
    });
}
