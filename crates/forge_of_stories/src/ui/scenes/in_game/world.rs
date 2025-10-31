mod assets;
mod lighting;
mod visual_spawners;

use crate::{GameState, utils::cleanup};
use bevy::prelude::*;

pub use assets::RenderAssets;
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
