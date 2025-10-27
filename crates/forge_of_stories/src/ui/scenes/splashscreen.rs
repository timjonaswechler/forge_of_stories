use crate::GameState;
use crate::utils::{cleanup, remove};
use app::LOG_MAIN;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for managing the splashscreen scene
pub struct SplashscreenScenePlugin;

impl Plugin for SplashscreenScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), setup_splashscreen)
            .add_systems(
                Update,
                (animate_logo, auto_transition_to_main_menu)
                    .run_if(in_state(GameState::Splashscreen)),
            )
            .add_systems(
                OnExit(GameState::Splashscreen),
                (cleanup::<SplashscreenEntity>, remove::<SplashscreenTimer>),
            )
            .add_input_context::<SplashscreenContext>();
    }
}

/// Marker component for splashscreen entities
#[derive(Component)]
struct SplashscreenEntity;

/// Context for splashscreen
#[derive(Component)]
struct SplashscreenContext;

/// Timer for automatic transition to main menu
#[derive(Resource)]
struct SplashscreenTimer(Timer);

fn setup_splashscreen(mut commands: Commands) {
    info!(target: LOG_MAIN, "Setting up splashscreen scene");

    // TODO: Load and spawn 3D logo model here
    // For now, spawn a placeholder cube
    commands.spawn((
        Mesh3d(Handle::default()), // Will be replaced with actual mesh
        Transform::from_xyz(0.0, 0.0, 0.0),
        SplashscreenEntity,
        SplashscreenContext,
        LogoAnimator {
            rotation_speed: 1.0,
        },
        Name::new("Logo Placeholder"),
    ));

    // Auto-transition after 3 seconds
    commands.insert_resource(SplashscreenTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

/// Component for logo animation
#[derive(Component)]
struct LogoAnimator {
    rotation_speed: f32,
}

fn animate_logo(time: Res<Time>, mut query: Query<(&mut Transform, &LogoAnimator)>) {
    for (mut transform, animator) in &mut query {
        transform.rotate_y(animator.rotation_speed * time.delta_secs());
    }
}

fn auto_transition_to_main_menu(
    time: Res<Time>,
    mut timer: ResMut<SplashscreenTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        info!(target: LOG_MAIN, "Splashscreen finished, transitioning to MainMenu");
        next_state.set(GameState::MainMenu);
    }
}
