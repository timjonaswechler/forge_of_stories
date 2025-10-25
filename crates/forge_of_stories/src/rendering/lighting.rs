use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

/// Plugin for managing scene lighting
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_lighting)
            .add_systems(OnExit(GameState::InGame), cleanup::<LightingEntity>);
    }
}

/// Marker component for lighting entities
#[derive(Component)]
struct LightingEntity;

fn setup_lighting(mut commands: Commands, mut ambient_light: Option<ResMut<AmbientLight>>) {
    // Configure ambient light
    if let Some(mut ambient_light) = ambient_light {
        ambient_light.brightness = 3_000.0;
        ambient_light.color = Color::srgb(1.0, 1.0, 1.0);
    }

    // Spawn directional light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 3_000.0,
            ..default()
        },
        Transform::from_xyz(-12.0, 18.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        LightingEntity,
        Name::new("Main Directional Light"),
    ));
}
