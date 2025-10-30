//! InGame Input Layer
//!
//! Handles input processing and auto-transition logic for the InGame.

use crate::{GameState, utils::cleanup};
use app::LOG_MAIN;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for InGame input handling
pub(super) struct InGameInputPlugin;

impl Plugin for InGameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<InGameContext>()
            .add_systems(OnEnter(GameState::InGame), setup_input)
            .add_systems(
                Update,
                (handle_skip_input,).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>);
    }
}

/// Input context for InGame controls
#[derive(Component, Default)]
pub(super) struct InGameContext;

/// Action to skip the InGame
#[derive(InputAction)]
#[action_output(bool)]
struct SkipInGame;

/// Sets up input context and timer
fn setup_input(mut commands: Commands) {
    // Spawn input context entity
    commands.spawn((
        Name::new("InGame Input Context"),
        InGameContext,
        actions!(
            InGameContext[(
                Action::<SkipInGame>::default(),
                bindings![KeyCode::Space, KeyCode::Enter, KeyCode::Escape],
            )]
        ),
    ));

    info!(target: LOG_MAIN, "InGame input initialized");
}

/// Handles user input to skip the InGame
fn handle_skip_input(
    mut commands: Commands,
    actions: Query<(&ActionValue, &ActionOf<InGameContext>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (value, _) in &actions {
        if value.as_bool() {
            info!(
                target: LOG_MAIN,
                "InGame skipped by user input, transitioning to MainMenu"
            );
            next_state.set(GameState::MainMenu);
            break;
        }
    }
}
