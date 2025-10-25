use super::RenderAssets;
use crate::GameState;
use app::LOG_CLIENT;
use bevy::math::primitives::Cuboid;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use game_server::components::{Player, Position};
use game_server::world::{GroundPlane, GroundPlaneSize};

/// Plugin for spawning visuals for replicated entities
pub struct VisualSpawnersPlugin;

impl Plugin for VisualSpawnersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_ground_plane_visuals,
                spawn_player_visuals,
                update_transforms_from_positions,
            )
                .run_if(in_state(GameState::InGame)),
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
    planes: Query<(Entity, &Position, &GroundPlaneSize), (With<GroundPlane>, Without<HasVisuals>)>,
) {
    for (entity, position, size) in &planes {
        info!(
            target: LOG_CLIENT,
            "Spawning visuals for ground plane at {:?} with size {}x{}x{}",
            position.translation, size.width, size.height, size.depth
        );

        let mesh = meshes.add(Mesh::from(Cuboid::new(size.width, size.height, size.depth)));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.45, 0.25),
            perceptual_roughness: 0.7,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position.translation),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
        ));
    }
}

fn spawn_player_visuals(
    mut commands: Commands,
    assets: Res<RenderAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Player, &Position), Without<HasVisuals>>,
) {
    for (entity, player, position) in &players {
        info!(
            target: LOG_CLIENT,
            "Spawning visuals for player with color {:?} at {:?}",
            player.color, position.translation
        );

        let material = materials.add(StandardMaterial {
            base_color: player.color,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(assets.player_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position.translation),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
        ));
    }
}

fn update_transforms_from_positions(
    mut query: Query<(&Position, &mut Transform), Changed<Position>>,
) {
    for (position, mut transform) in &mut query {
        transform.translation = position.translation;
    }
}

#[cfg(debug_assertions)]
pub fn debug_replicated_entities(
    new_planes: Query<(Entity, &Position), bevy::ecs::query::Added<GroundPlane>>,
    new_players: Query<(Entity, &Player, &Position), bevy::ecs::query::Added<Player>>,
) {
    for (entity, position) in &new_planes {
        info!(
            target: LOG_CLIENT,
            "New GroundPlane {:?} at {:?}",
            entity, position.translation
        );
    }
    for (entity, player, position) in &new_players {
        info!(
            target: LOG_CLIENT,
            "New Player {:?} color {:?} at {:?}",
            entity, player.color, position.translation
        );
    }
}

#[cfg(not(debug_assertions))]
pub fn debug_replicated_entities() {
    // No-op in release builds
}
