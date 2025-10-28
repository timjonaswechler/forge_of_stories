//! Main Menu 3D World Layer
//!
//! Contains all 3D entities for the main menu (background scene, environment, effects).

use crate::GameState;
use bevy::prelude::*;

/// Plugin for main menu 3D world content
pub(super) struct MainMenuWorldPlugin;

impl Plugin for MainMenuWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_world)
            .add_systems(
                Update,
                (animate_background, rotate_ambient_objects).run_if(in_state(GameState::MainMenu)),
            );
    }
}

/// Marker component for main menu 3D world entities
#[derive(Component)]
pub(super) struct MainMenuWorld;

/// Component for background animation behavior
#[derive(Component)]
struct BackgroundAnimator {
    rotation_speed: f32,
    float_amplitude: f32,
    float_frequency: f32,
}

/// Component for ambient rotating objects
#[derive(Component)]
struct AmbientRotator {
    axis: Vec3,
    speed: f32,
}

/// Spawns the 3D world content (background scene, lighting, effects)
fn spawn_world(mut commands: Commands) {
    // Spawn background sphere or environment
    // TODO: Replace with actual background mesh (planet, space scene, etc.)
    commands.spawn((
        Mesh3d(Handle::default()),
        Transform::from_xyz(0.0, 0.0, -10.0).with_scale(Vec3::splat(5.0)),
        MainMenuWorld,
        BackgroundAnimator {
            rotation_speed: 0.1,
            float_amplitude: 0.5,
            float_frequency: 0.5,
        },
        Name::new("Background Scene"),
    ));

    // Spawn directional light (key light)
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        MainMenuWorld,
        Name::new("Key Light"),
    ));

    // Spawn ambient light
    commands.spawn((
        AmbientLight {
            color: Color::srgb(0.7, 0.8, 1.0),
            brightness: 150.0,
            affects_lightmapped_meshes: false,
        },
        MainMenuWorld,
        Name::new("Ambient Light"),
    ));

    // Spawn some ambient rotating objects (particles, stars, etc.)
    // TODO: Replace with actual decorative meshes
    for i in 0..3 {
        let angle = (i as f32) * std::f32::consts::TAU / 3.0;
        let radius = 8.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        commands.spawn((
            Mesh3d(Handle::default()),
            Transform::from_xyz(x, 0.0, z).with_scale(Vec3::splat(0.3)),
            MainMenuWorld,
            AmbientRotator {
                axis: Vec3::new((i as f32 * 0.5).sin(), 1.0, (i as f32 * 0.5).cos()).normalize(),
                speed: 0.5 + i as f32 * 0.2,
            },
            Name::new(format!("Ambient Object {}", i)),
        ));
    }
}

/// Animates the background scene (rotation and floating)
fn animate_background(time: Res<Time>, mut query: Query<(&mut Transform, &BackgroundAnimator)>) {
    for (mut transform, animator) in &mut query {
        // Rotate
        transform.rotate_y(animator.rotation_speed * time.delta_secs());

        // Float up and down
        let time_secs = time.elapsed_secs();
        let offset_y = (time_secs * animator.float_frequency).sin() * animator.float_amplitude;
        transform.translation.y = offset_y;
    }
}

/// Rotates ambient decorative objects
fn rotate_ambient_objects(time: Res<Time>, mut query: Query<(&mut Transform, &AmbientRotator)>) {
    for (mut transform, rotator) in &mut query {
        if let Ok(dir) = Dir3::new(rotator.axis) {
            transform.rotate_axis(dir, rotator.speed * time.delta_secs());
        }
    }
}
