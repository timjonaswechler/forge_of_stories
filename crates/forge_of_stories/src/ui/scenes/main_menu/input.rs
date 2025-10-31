//! Main Menu Input Layer
//!
//! Handles server connection logic and state transitions for the main menu.

use crate::client::LocalClientId;
use crate::{GameState, utils::cleanup};
use app::LOG_CLIENT_HOST;
use bevy::prelude::*;
use bevy_replicon_renet::{netcode::NetcodeClientTransport, renet::RenetClient};

/// Input context marker for main menu
#[derive(Component, Default)]
pub struct MainMenuContext;

/// Plugin for main menu input and connection handling
pub(super) struct MainMenuInputPlugin;

impl Plugin for MainMenuInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), log_state_entry)
            .add_systems(
                Update,
                wait_for_server_ready.run_if(in_state(GameState::ConnectingToServer)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_game_resources)
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuContext>);
    }
}

/// Logs when entering the main menu state
fn log_state_entry(state: Res<State<GameState>>) {
    info!(target: app::LOG_MAIN, "Entered state: {:?}", state.get());
}

/// Waits for the embedded server to be ready, then transitions to InGame
///
/// This system runs in the ConnectingToServer state and monitors the
/// ServerHandle resource. Once the server is ready, it transitions to InGame.
fn wait_for_server_ready(
    server: Option<Res<game_server::ServerHandle>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(server) = server {
        if server.is_ready() {
            info!(
                target: LOG_CLIENT_HOST,
                "Server is ready! Transitioning to InGame state"
            );
            next_state.set(GameState::InGame);
        }
    } else {
        error!(
            target: LOG_CLIENT_HOST,
            "No ServerHandle resource found while waiting for server!"
        );
    }
}

fn cleanup_game_resources(
    mut commands: Commands,
    server_handle: Option<Res<game_server::ServerHandle>>,
) {
    // Server stoppen
    if let Some(server) = server_handle {
        // Shutdown-Logik hier
        commands.remove_resource::<game_server::ServerHandle>();
    }

    // Netzwerk-Ressourcen entfernen
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
    commands.remove_resource::<LocalClientId>();
}
