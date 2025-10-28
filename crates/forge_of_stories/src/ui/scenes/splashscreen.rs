//! Splashscreen Scene
//!
//! This module contains all components for the splashscreen scene:
//! - UI: 2D overlay with text and instructions
//! - World: 3D logo mesh and lighting
//! - Camera: Camera positioning (delegated to global camera system)
//! - Input: Skip functionality and auto-transition logic
//!
//! The scene-first architecture keeps all related code together,
//! making it easy to understand and maintain the complete scene.

mod camera;
mod input;
mod ui;
mod world;

use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

/// Main plugin for the splashscreen scene
///
/// Coordinates all sub-plugins and handles cleanup on exit.
pub struct SplashscreenScenePlugin;

impl Plugin for SplashscreenScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all sub-plugins
            .add_plugins((
                ui::SplashscreenUIPlugin,
                world::SplashscreenWorldPlugin,
                camera::SplashscreenCameraPlugin,
                input::SplashscreenInputPlugin,
            ))
            // Cleanup all scene entities on exit
            .add_systems(
                OnExit(GameState::Splashscreen),
                (
                    cleanup::<ui::SplashscreenUI>,
                    cleanup::<world::SplashscreenWorld>,
                    cleanup::<input::SplashscreenContext>,
                ),
            );
    }
}
