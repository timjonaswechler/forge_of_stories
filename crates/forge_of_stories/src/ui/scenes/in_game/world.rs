mod assets;
mod lighting;
mod visual_spawners;

use crate::networking::LocalPlayer;
use crate::ui::normal_vector::LocalCoordinateSystem;
use crate::ui::scenes::in_game::cameras::InGameCamera;
use crate::{GameState, utils::cleanup};
pub use assets::RenderAssets;
use bevy::prelude::*;
pub use lighting::LightingPlugin;
pub use visual_spawners::VisualSpawnersPlugin;

#[derive(Component)]
pub struct InGameWorld;

/// Main world plugin that coordinates all rendering aspects
pub struct InGameWorldPlugin;

impl Plugin for InGameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LightingPlugin, VisualSpawnersPlugin))
            .add_systems(OnEnter(GameState::InGame), setup_render_assets)
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameWorld>);
    }
}

fn setup_render_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    use bevy::math::primitives::Capsule3d;

    let player_mesh = meshes.add(Mesh::from(Capsule3d::default()));

    commands.insert_resource(RenderAssets { player_mesh });
}

fn update_local_player_gizmo(
    mut commands: Commands,
    // Finde unseren Spieler und die Kamera
    local_player_query: Single<Entity, With<LocalPlayer>>,
    camera_query: Single<&Transform, With<InGameCamera>>,
) {
    if let player_entity = local_player_query {
        if let camera_transform = camera_query {
            // Füge/aktualisiere die Gizmo-Komponente auf unserer Spieler-Entität
            commands
                .entity(player_entity.entity())
                .insert(LocalCoordinateSystem {
                    origin: camera_transform.translation, // oder Spielerposition
                    forward: Some(camera_transform.forward().as_vec3()),
                    right: None,
                    up: None,
                    length: 2.0,
                });
        }
    }
}
