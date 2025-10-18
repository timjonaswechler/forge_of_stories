//! Client-side systems for bevy_replicon integration.
//!
//! Handles:
//! - Player input sending via bevy_replicon Messages
//! - Rendering of replicated entities
//! - Camera following

use crate::InGame;
use crate::ui::normal_vector::NormalArrowVisual;
use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED},
    prelude::*,
};
use game_server::{Player, PlayerInput, Position};
/// Marker component for client-side player mesh rendering.
#[derive(Component)]
pub struct PlayerMesh;

/// System that sends player input to the server using bevy_replicon's Message system.
///
/// This system runs every frame when in-game and the menu is closed.
pub fn send_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_writer: Option<MessageWriter<PlayerInput>>,
) {
    // Return early if the message writer is not yet initialized
    let Some(mut input_writer) = input_writer else {
        return;
    };

    // Calculate movement direction from WASD/Arrow keys
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Normalize diagonal movement
    if direction.length() > 0.01 {
        direction = direction.normalize();
    }

    // Send input to server
    input_writer.write(PlayerInput {
        direction,
        jump: keyboard.pressed(KeyCode::Space),
    });
}

/// System that spawns mesh rendering for newly replicated player entities.
///
/// When the server replicates a Player entity to the client, this system
/// adds the mesh and material components for rendering.
pub fn spawn_player_meshes(
    mut commands: Commands,
    new_players: Query<(Entity, &Player), Added<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, player) in &new_players {
        info!("Spawning mesh for player {:?}", entity);

        // Add mesh and material to the replicated entity
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: player.color,
                ..default()
            })),
            PlayerMesh,
            InGame,
        ));
    }
}
#[derive(Component)]
pub struct DebugArrows;

/// System that updates player mesh positions from replicated Position components.
///
/// This runs every frame to smoothly update visual positions.
pub fn update_player_positions(
    mut commands: Commands,
    mut players: Query<(&Position, &mut Transform), With<Player>>,
) {
    for (position, mut transform) in &mut players {
        transform.translation = position.translation;
        let head_origin = position.translation + Vec3::new(0.0, 1.0, 0.0);
        commands.spawn((
            NormalArrowVisual {
                origin: head_origin,
                direction: Vec3::Y, // Normale ist der normalisierte Vektor vom Kugelzentrum zum Punkt
                length: 0.5,
                color: RED,
            },
            InGame,
            DebugArrows,
        ));

        // Pfeil am Körper (X-Achse)
        let body_origin = position.translation + Vec3::new(0.5, 0.0, 0.0);
        commands.spawn((
            NormalArrowVisual {
                origin: body_origin,
                direction: Vec3::X,
                length: 0.5,
                color: GREEN,
            },
            InGame,
            DebugArrows,
        ));

        // Pfeil am Äquator (Z-Achse)
        let hip_origin = position.translation + Vec3::new(0.0, 0.0, 0.5);
        commands.spawn((
            NormalArrowVisual {
                origin: hip_origin,
                direction: Vec3::Z,
                length: 0.5,
                color: BLUE,
            },
            InGame,
            DebugArrows,
        ));
    }
}

/// System that despawns player meshes when players disconnect.
pub fn despawn_player_meshes(
    mut commands: Commands,
    mut removed_players: RemovedComponents<Player>,
) {
    for entity in removed_players.read() {
        info!("Despawning mesh for player {:?}", entity);
        commands.entity(entity).despawn();
    }
}

/// System that follows the local player with the camera.
///
/// For now, follows the first player (in singleplayer, this is the host).
/// In the future, this should follow the player owned by this client.
pub fn follow_local_player(
    players: Query<&Position, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    // For now, just follow the first player we find
    if let Some(player_pos) = players.iter().next() {
        if let Ok(mut camera_transform) = camera.single_mut() {
            // Position camera behind and above the player
            let target = player_pos.translation;
            let offset = Vec3::new(0.0, 5.0, 10.0);
            camera_transform.translation = target + offset;
            camera_transform.look_at(target, Vec3::Y);
        }
    }
}
