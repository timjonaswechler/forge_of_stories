//! Main Menu Camera Layer
//!
//! Handles camera positioning and configuration for the main menu.
//! Currently uses the global scene camera managed by the CameraPlugin.

use crate::GameState;
use bevy::prelude::*;

/// Plugin for main menu camera setup
pub(super) struct MainMenuCameraPlugin;

impl Plugin for MainMenuCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_camera);
    }
}

/// Sets up camera position for the main menu
///
/// Currently this is handled by the global CameraPlugin's mode switching system.
/// This module exists for future scene-specific camera customization if needed.
fn setup_camera() {
    // Camera positioning is currently handled by:
    // - cameras::CameraPlugin (global camera management)
    // - cameras::switch_to_main_menu_mode (triggered by OnEnter(GameState::MainMenu))
    // - cameras::handle_camera_mode_changes (applies CameraDefaults.main_menu)

    // Future enhancements could include:
    // - Cinematic camera movements (pan around the background scene)
    // - Dynamic camera positioning based on UI layout
    // - Custom camera effects (depth of field, bloom, etc.)
    // - Camera shake or parallax effects on button hover
}
