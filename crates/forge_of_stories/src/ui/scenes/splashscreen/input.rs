//! Splashscreen Input Layer
//!
//! Handles input processing and auto-transition logic for the splashscreen.

use crate::{GameState, utils::cleanup, utils::remove};
use app::LOG_MAIN;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for splashscreen input handling
pub(super) struct SplashscreenInputPlugin;

impl Plugin for SplashscreenInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<SplashscreenContext>()
            .add_systems(OnEnter(GameState::Splashscreen), setup_input)
            .add_systems(
                Update,
                // (handle_skip_input, auto_transition_to_main_menu)
                handle_skip_input.run_if(in_state(GameState::Splashscreen)),
            )
            .add_systems(
                OnExit(GameState::Splashscreen),
                (remove::<SplashscreenTimer>, cleanup::<SplashscreenContext>),
            );
    }
}

/// Input context for splashscreen controls
#[derive(Component, Default)]
pub(super) struct SplashscreenContext;

/// Action to skip the splashscreen
#[derive(InputAction)]
#[action_output(bool)]
struct SkipSplashscreen;

/// Timer resource for automatic transition to main menu
#[derive(Resource)]
struct SplashscreenTimer(Timer);

/// Sets up input context and timer
fn setup_input(mut commands: Commands) {
    // Spawn input context entity
    commands.spawn((
        Name::new("Splashscreen Input Context"),
        SplashscreenContext,
        actions!(
            SplashscreenContext[(
                Action::<SkipSplashscreen>::default(),
                bindings![KeyCode::Space, KeyCode::Enter, KeyCode::Escape],
            )]
        ),
    ));

    // Initialize auto-transition timer
    commands.insert_resource(SplashscreenTimer(Timer::from_seconds(3.0, TimerMode::Once)));

    info!(target: LOG_MAIN, "Splashscreen input initialized");
}

/// Handles user input to skip the splashscreen
fn handle_skip_input(
    mut commands: Commands,
    actions: Query<(&ActionValue, &ActionOf<SplashscreenContext>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (value, _) in &actions {
        if value.as_bool() {
            info!(
                target: LOG_MAIN,
                "Splashscreen skipped by user input, transitioning to MainMenu"
            );
            next_state.set(GameState::MainMenu);
            commands.remove_resource::<SplashscreenTimer>();
            break;
        }
    }
}

/// Automatically transitions to main menu after timer expires
fn auto_transition_to_main_menu(
    time: Res<Time>,
    mut timer: ResMut<SplashscreenTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        info!(
            target: LOG_MAIN,
            "Splashscreen auto-transition timeout reached, transitioning to MainMenu"
        );
        next_state.set(GameState::MainMenu);
    }
}

/// Cleans up input resources on exit
fn cleanup_input_resources(mut commands: Commands) {
    commands.remove_resource::<SplashscreenTimer>();
}
