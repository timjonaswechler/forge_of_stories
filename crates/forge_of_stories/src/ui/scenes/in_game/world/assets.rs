use bevy::prelude::*;

/// Cached handles for meshes reused while in-game
#[derive(Resource)]
pub struct RenderAssets {
    pub player_mesh: Handle<Mesh>,
}
