//! Main Menu Scene
//!
//! This module contains all components for the main menu scene:
//! - UI: 2D overlay with title, buttons, and menus
//! - World: 3D background scene with environment and effects
//! - Camera: Camera positioning (delegated to global camera system)
//! - Input: Server connection handling and state transitions
//!
//! The scene-first architecture keeps all related code together,
//! making it easy to understand and maintain the complete scene.

mod camera;
mod input;
mod ui;
mod world;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputContextAppExt;

/// Main plugin for the main menu scene
///
/// Coordinates all sub-plugins and handles cleanup on exit.
pub struct MainMenuScenePlugin;

impl Plugin for MainMenuScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all sub-plugins
            .add_plugins((
                ui::MainMenuUIPlugin,
                world::MainMenuWorldPlugin,
                camera::MainMenuCameraPlugin,
                input::MainMenuInputPlugin,
            ))
            // Input context registration
            .add_input_context::<input::MainMenuContext>();
    }
}
