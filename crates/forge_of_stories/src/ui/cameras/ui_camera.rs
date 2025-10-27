use bevy::camera::ClearColorConfig;
use bevy::prelude::*;

/// Marker component for the dedicated UI camera.
/// This camera should always remain active and render after 3D cameras.
#[derive(Component)]
pub struct UiCamera;

/// Plugin that ensures a single persistent UI camera exists.
/// Spawns a Camera2d with:
/// - high render order (renders above 3D cameras)
/// - no clear color (keeps the 3D scene visible behind UI)
pub struct UiCameraPlugin;

impl Plugin for UiCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui_camera_once);
    }
}

fn spawn_ui_camera_once(mut commands: Commands, existing: Query<Entity, With<UiCamera>>) {
    // Avoid duplicate UI cameras (e.g. during hot reload or multiple app setups)
    if existing.is_empty() {
        commands.spawn((
            Camera2d::default(),
            Camera {
                order: 10,
                clear_color: ClearColorConfig::Default,
                ..default()
            },
            UiCamera,
            Name::new("UI Camera"),
        ));
    }
}
