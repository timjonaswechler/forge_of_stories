//! InGame Input Layer
//!
//! Handles input processing for the InGame scene.

use crate::{
    GameState,
    ui::{cameras::cursor::CursorState, components::InGameMenuState},
    utils::cleanup,
};
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
                (handle_menu_toggle, manage_cursor_state).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>);
    }
}

/// Input context for InGame controls
#[derive(Component, Default)]
pub(super) struct InGameContext;

/// Action to toggle the in-game menu
#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMenu;

/// Sets up input context
fn setup_input(mut commands: Commands) {
    commands.spawn((
        Name::new("InGame Input Context"),
        InGameContext,
        actions!(InGameContext[(Action::<ToggleMenu>::default(), bindings![KeyCode::Escape],)]),
    ));

    info!(target: LOG_MAIN, "InGame input initialized");
}

/// Handles ESC key to toggle the in-game menu
fn handle_menu_toggle(
    actions: Query<(&ActionValue, &ActionOf<InGameContext>)>,
    mut menu: ResMut<InGameMenuState>,
) {
    for (value, _) in &actions {
        if value.as_bool() {
            menu.toggle();
            info!(
                target: LOG_MAIN,
                "In-game menu toggled: {}",
                if menu.is_open() { "open" } else { "closed" }
            );
            break;
        }
    }
}

/// Manages cursor state based on menu state
fn manage_cursor_state(menu: Res<InGameMenuState>, mut cursor: ResMut<CursorState>) {
    if menu.is_changed() {
        *cursor = if menu.is_open() {
            CursorState::FREE
        } else {
            CursorState::LOCKED
        };
    }
}
