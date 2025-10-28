//! Splashscreen Camera Layer
//!
//! Handles camera positioning and configuration for the splashscreen.
//! Currently uses the global scene camera managed by the CameraPlugin.

use crate::GameState;
use bevy::prelude::*;

/// Plugin for splashscreen camera setup
pub(super) struct SplashscreenCameraPlugin;

impl Plugin for SplashscreenCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), setup_camera);
    }
}

/// Sets up camera position for the splashscreen
///
/// Currently this is handled by the global CameraPlugin's mode switching system.
/// This module exists for future scene-specific camera customization if needed.
fn setup_camera() {
    // Camera positioning is currently handled by:
    // - cameras::CameraPlugin (global camera management)
    // - cameras::switch_to_splashscreen_mode (triggered by OnEnter(GameState::Splashscreen))
    // - cameras::handle_camera_mode_changes (applies CameraDefaults.splashscreen)

    // Future enhancements could include:
    // - Scene-specific camera animations
    // - Custom camera effects (bloom, vignette, etc.)
    // - Dynamic camera positioning based on logo placement
}
