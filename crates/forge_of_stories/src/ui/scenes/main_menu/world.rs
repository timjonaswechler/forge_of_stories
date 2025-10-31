//! Main Menu 3D World Layer
//!
//! Contains all 3D entities for the main menu (background scene, environment, effects).

use crate::{GameState, utils::cleanup};
use bevy::color::palettes::css::*;
use bevy::prelude::*;

/// Plugin for main menu 3D world content
pub(super) struct MainMenuWorldPlugin;

impl Plugin for MainMenuWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_world)
            .add_systems(
                Update,
                (animate_background, rotate_ambient_objects).run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::MainMenu),
                (cleanup::<MainMenuWorld>, verify_cleanup_after).chain(),
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
fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::srgb(RED.red, RED.green, RED.blue))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        MainMenuWorld,
        Name::new("MainMenu Circle"),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        MainMenuWorld,
        Name::new("MainMenu Cube"),
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

fn verify_cleanup_after(query: Query<(Entity, &Name), With<MainMenuWorld>>) {
    if query.iter().count() > 0 {
        error!("⚠️  MainMenu entities still exist after cleanup!");
        for (entity, name) in &query {
            error!("  - {:?}: {}", entity, name.as_str());
        }
    } else {
        info!("✅ MainMenu cleanup verified - all entities removed");
    }
}
