// scenes/main_menu/camera.rs

use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

pub(super) struct MainMenuCameraPlugin;

impl Plugin for MainMenuCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_camera)
            .add_systems(Update, animate_orbit.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuCamera>);
    }
}

#[derive(Component)]
struct MainMenuCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
        Name::new("Main Menu Camera"),
    ));
}

fn animate_orbit(time: Res<Time>, mut cameras: Query<&mut Transform, With<MainMenuCamera>>) {
    for mut transform in &mut cameras {
        let radius = 8.0;
        let speed = 0.1;
        let angle = time.elapsed_secs() * speed;

        transform.translation = Vec3::new(angle.cos() * radius, 1.5, angle.sin() * radius);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
