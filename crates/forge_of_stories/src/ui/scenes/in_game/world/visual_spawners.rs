use super::InGameWorld;
use super::RenderAssets;
use crate::GameState;
use app::LOG_CLIENT;
use bevy::math::primitives::Cuboid;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use game_server::{GroundPlane, GroundPlaneSize, Player};

/// Plugin for spawning visuals for replicated entities
pub struct VisualSpawnersPlugin;

impl Plugin for VisualSpawnersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_ground_plane_visuals, spawn_player_visuals).run_if(in_state(GameState::InGame)),
        );
    }
}

/// Marker to track which entities already have visuals spawned
#[derive(Component)]
struct HasVisuals;

fn spawn_ground_plane_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planes: Query<(Entity, &Transform, &GroundPlaneSize), (With<GroundPlane>, Without<HasVisuals>)>,
) {
    for (entity, transform, size) in &planes {
        // info!(
        //     target: LOG_CLIENT,
        //     "Spawning visuals for ground plane at {:?} with size {}x{}x{}",
        //     position.translation, size.width, size.height, size.depth
        // );

        let mesh = meshes.add(Mesh::from(Cuboid::new(size.width, size.height, size.depth)));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.45, 0.25),
            perceptual_roughness: 1.1,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(transform.translation),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
            InGameWorld,
        ));
    }
}

fn spawn_player_visuals(
    mut commands: Commands,
    assets: Res<RenderAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Player, &Transform), Without<HasVisuals>>,
) {
    for (entity, player, transform) in &players {
        // info!(
        //     target: LOG_CLIENT,
        //     "Spawning visuals for player with color {:?} at {:?}",
        //     player.color, position.translation
        // );

        let material = materials.add(StandardMaterial {
            base_color: player.color,
            ..default()
        });

        let mut transform = Transform::from_translation(transform.translation);
        transform.rotation = Quat::from_euler(
            bevy::math::EulerRot::XYZ,
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
        );

        commands.entity(entity).insert((
            Mesh3d(assets.player_mesh.clone()),
            MeshMaterial3d(material),
            transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
            InGameWorld,
        ));
    }
}

#[cfg(not(debug_assertions))]
pub fn debug_replicated_entities() {
    // No-op in release builds
}
