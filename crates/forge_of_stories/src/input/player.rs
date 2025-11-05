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
    player: Single<(Entity, &Transform), With<LocalPlayer>>,
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

    // Kamera forward/right Vektoren direkt verwenden
    // Projektion auf XZ-Ebene (Y auf 0 setzen) für horizontale Bewegung
    let camera_forward = Vec3::from(*camera_transform.forward());
    let camera_right = Vec3::from(*camera_transform.right());

    let forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let right = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize_or_zero();

    // Rotation für den Spieler (nur Y-Achse)
    let (_, yaw, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    let player_rotation = Quat::from_rotation_y(yaw);

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

    // Normalisieren wenn Bewegung stattfindet
    if movement.length() > 0.0 {
        movement = movement.normalize();
        info!("Camera yaw: {:.2}, Movement vector: {:?}", yaw.to_degrees(), movement);
    }

    // Transform mit Rotation erstellen (Position kommt vom Server)
    let mut input_transform = *player.1;
    input_transform.rotation = player_rotation;

    // Send input to server
    commands.client_trigger(PlayerMovement {
        transform: input_transform,
        movement,
        jump: keyboard.pressed(KeyCode::Space),
    });
}
