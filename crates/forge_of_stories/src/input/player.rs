//! Player movement input handling.
//!
//! Sends player input (WASD, Space) to the server via bevy_replicon events.

use crate::GameState;
use crate::networking::LocalPlayer;
use crate::ui::scenes::in_game::cameras::{ActiveCameraMode, CursorState, InGameCamera};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_server::PlayerMovement;

/// Plugin for player movement input.
///
/// Registers the PlayerInput client event and sends input to the server
/// when the player is in-game, connected, and the cursor is locked.
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<PlayerMovement>(Channel::Unreliable)
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
    mut player: Single<(Entity, &mut Transform), With<LocalPlayer>>,
    camera: Single<(&Transform, &ActiveCameraMode), (With<InGameCamera>, Without<LocalPlayer>)>,
) {
    // TODO: Je anch kamera werden unterschiedlichen INput logiken aktiv und gesendet
    // wenn man in Fist Person ist kann man den Spiler direkt über WASD bewegen und die rotation des Spielers ist direkt von der Maus Abhängig
    // Wenn man in Thirdperson ist kann man den Spieler durch klicken auf dem bildschirm auf die richtige position bringen. Rotation wird automatisch gemacht.
    // man kann aber auch mit wasd bewegen hier wird auch der Spieler in die richtung rotiert in die man sich bewegbt.
    // in PAN Mode kann man die die Kamera frei vom Spieler durch WASD oder durch zeihen der Maus bewegen.
    // if camera.1.mode == CameraMode::FirstPerson {

    // let mut transform = *player.1;
    // transform.rotation = camera.0.rotation;
    //

    let camera_transform = camera.0;
    let (_, yaw, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    // Spieler nur um Y-Rotation drehen (relativ zur Kamera)
    let player_rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);

    // Bewegungsrichtungen basierend auf Kamera-Rotation berechnen
    let forward = player_rotation * Vec3::NEG_Z;
    let right = player_rotation * Vec3::X;

    let mut movement = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        movement += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement += right;
    }

    // Wenn Bewegung stattfindet, normalisieren und skalieren
    if movement.length() > 0.0 {
        movement = movement.normalize() * 1.0;

        // WICHTIG: Lokale Transform direkt aktualisieren
        player.1.translation += movement;
        player.1.rotation = player_rotation;
    }

    // Send input to server
    commands.client_trigger(PlayerMovement {
        transform: *player.1,
        jump: keyboard.pressed(KeyCode::Space),
    });
}
