mod assets;
mod lighting;
mod visual_spawners;

use crate::GameState;
use bevy::prelude::*;

pub use assets::RenderAssets;
pub use lighting::LightingPlugin;
pub use visual_spawners::VisualSpawnersPlugin;

/// Main rendering plugin that coordinates all rendering aspects
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LightingPlugin, VisualSpawnersPlugin))
            .add_systems(OnEnter(GameState::InGame), setup_render_assets)
            .add_systems(OnExit(GameState::InGame), cleanup_render_assets);
    }
}

fn setup_render_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    use bevy::math::primitives::Capsule3d;

    let player_mesh = meshes.add(Mesh::from(Capsule3d::default()));

    commands.insert_resource(RenderAssets { player_mesh });
}

fn cleanup_render_assets(mut commands: Commands) {
    commands.remove_resource::<RenderAssets>();
}
