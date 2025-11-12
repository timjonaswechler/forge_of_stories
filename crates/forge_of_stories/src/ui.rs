pub mod components;
pub mod normal_vector;
pub mod scenes;

use bevy::{input_focus::InputFocus, prelude::*};

use components::InGameMenuState;
use normal_vector::draw_local_coordinate_systems;
use scenes::ScenePlugin;

/// Main UI plugin that coordinates cameras, scenes, and UI systems
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScenePlugin)
            .init_resource::<InGameMenuState>()
            .init_resource::<InputFocus>()
            // Debug helper for normal vectors
            .add_systems(Update, draw_local_coordinate_systems);
    }
}
