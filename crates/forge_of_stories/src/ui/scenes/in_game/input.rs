//! InGame Input Context
//!
//! Sets up the input context for the InGame scene.
//! Actual input handling is now centralized in the `input/` module.

use crate::app::LOG_MAIN;
use crate::{GameState, utils::cleanup};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for InGame input context setup
pub(super) struct InGameInputPlugin;

impl Plugin for InGameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<InGameContext>()
            .add_systems(OnEnter(GameState::InGame), setup_input)
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>);
    }
}

/// Input context for InGame controls
#[derive(Component, Default)]
pub(super) struct InGameContext;

/// Sets up input context
fn setup_input(mut commands: Commands) {
    commands.spawn((Name::new("InGame Input Context"), InGameContext));
    info!(target: LOG_MAIN, "InGame input context initialized");
}
