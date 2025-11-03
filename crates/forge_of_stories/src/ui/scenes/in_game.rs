//! In-Game Scene
//!
//! Main gameplay scene with HUD, world, cameras, and input handling.

mod cameras;
mod hud;
mod input;
pub mod world;

use bevy::prelude::*;

/// Main plugin for the in-game scene
pub struct InGameScenePlugin;

impl Plugin for InGameScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all sub-plugins
            .add_plugins((
                hud::InGameHUDPlugin,
                world::InGameWorldPlugin,
                cameras::InGameCamerasPlugin,
                input::InGameInputPlugin,
            ));
    }
}
