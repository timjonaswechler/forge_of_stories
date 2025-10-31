// scenes/splashscreen/camera.rs

use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

pub(super) struct SplashscreenCameraPlugin;

impl Plugin for SplashscreenCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), spawn_camera)
            .add_systems(
                OnExit(GameState::Splashscreen),
                cleanup::<SplashscreenCamera>,
            );
    }
}

#[derive(Component)]
struct SplashscreenCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        SplashscreenCamera,
        Name::new("Splashscreen Camera"),
    ));
}
