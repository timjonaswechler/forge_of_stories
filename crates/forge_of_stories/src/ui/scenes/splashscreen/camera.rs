// scenes/splashscreen/camera.rs

use crate::GameState;
use bevy::prelude::*;

pub(super) struct SplashscreenCameraPlugin;

impl Plugin for SplashscreenCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), spawn_camera)
            .add_systems(OnExit(GameState::Splashscreen), cleanup);
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

fn cleanup(mut commands: Commands, cameras: Query<Entity, With<SplashscreenCamera>>) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
}
