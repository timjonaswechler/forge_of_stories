use super::SplashscreenCamera;
use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

/// Plugin for managing the splashscreen camera
/// Spawns a static 3D camera positioned to view the logo animation
pub struct SplashscreenCameraPlugin;

impl Plugin for SplashscreenCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), spawn_splashscreen_camera)
            .add_systems(
                OnExit(GameState::Splashscreen),
                cleanup::<SplashscreenCamera>,
            );
    }
}

fn spawn_splashscreen_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10.0, 8.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        SplashscreenCamera,
        Name::new("Splashscreen Camera"),
    ));
}
