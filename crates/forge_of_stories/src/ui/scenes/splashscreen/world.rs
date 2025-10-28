//! Splashscreen 3D World Layer
//!
//! Contains all 3D entities for the splashscreen (logo mesh, lighting, animations).

use crate::GameState;
use bevy::prelude::*;

/// Plugin for splashscreen 3D world content
pub(super) struct SplashscreenWorldPlugin;

impl Plugin for SplashscreenWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), spawn_world)
            .add_systems(
                Update,
                animate_logo.run_if(in_state(GameState::Splashscreen)),
            );
    }
}

/// Marker component for splashscreen 3D world entities
#[derive(Component)]
pub(super) struct SplashscreenWorld;

/// Component for logo animation behavior
#[derive(Component)]
struct LogoAnimator {
    rotation_speed: f32,
}

/// Spawns the 3D world content (logo, lights, etc.)
fn spawn_world(mut commands: Commands) {
    // Spawn the logo mesh
    commands.spawn((
        Mesh3d(Handle::default()), // TODO: Replace with actual logo mesh asset
        Transform::from_xyz(0.0, 0.0, 0.0),
        SplashscreenWorld,
        LogoAnimator {
            rotation_speed: 1.0,
        },
        Name::new("Logo"),
    ));

    // Spawn directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        SplashscreenWorld,
        Name::new("Directional Light"),
    ));

    // Optional: Spawn ambient light
    commands.spawn((
        AmbientLight {
            color: Color::srgb(0.8, 0.8, 1.0),
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        SplashscreenWorld,
        Name::new("Ambient Light"),
    ));
}

/// Animates the logo rotation
fn animate_logo(time: Res<Time>, mut query: Query<(&mut Transform, &LogoAnimator)>) {
    for (mut transform, animator) in &mut query {
        transform.rotate_y(animator.rotation_speed * time.delta_secs());
    }
}
