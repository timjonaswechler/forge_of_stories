use super::MainMenuCamera;
use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

/// Event to trigger camera pan animation in main menu
#[derive(Event)]
pub struct CameraPanEvent {
    pub target_position: Vec3,
    pub target_look_at: Vec3,
}

/// Plugin for managing the main menu camera
/// Supports animated panning based on menu selections
pub struct MainMenuCameraPlugin;

impl Plugin for MainMenuCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu_camera)
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuCamera>);
    }
}

fn spawn_main_menu_camera(mut commands: Commands) {
    // Reuse existing camera if available (from splashscreen)
    // Otherwise spawn a new one
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-12.0, 8.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
        Name::new("MainMenu Camera"),
    ));
}
