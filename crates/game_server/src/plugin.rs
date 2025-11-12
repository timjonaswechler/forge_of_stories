//! Server plugin for the game server.
//!
//! This plugin encapsulates all server-side logic and can be used in:
//! - Embedded server mode (running in a separate thread within the client)
//! - Dedicated server mode (standalone server binary)

use crate::app::LOG_SERVER;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

use crate::gameplay::{
    ClientMessageBuffer, apply_velocity, buffer_player_input, process_buffered_inputs,
    simulate_physics,
};
use crate::network::{
    Port, WorldSpawned, handle_client_connections, handle_client_disconnections, setup_networking,
};
use crate::shared::*;
use crate::world::*;

/// Main server plugin that sets up all server systems and state.
pub struct ServerPlugin {
    pub port: u16,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add replicon plugins
            .add_plugins(RepliconRenetPlugins)
            // Resources
            .insert_resource(Port(self.port))
            .insert_resource(Time::<Fixed>::from_hz(20.0))
            .init_resource::<WorldSpawned>()
            .init_resource::<PlayerColorAssigner>()
            .init_resource::<ClientMessageBuffer>()
            .init_state::<GameplayState>()
            // Register replicated components (must match client!)
            .replicate::<Player>()
            .replicate::<PlayerIdentity>()
            .replicate::<Transform>()
            .replicate::<Velocity>()
            .replicate::<GroundPlane>()
            .replicate::<GroundPlaneSize>()
            // Register client events (must match client!)
            .add_client_event::<PlayerMovement>(Channel::Unreliable)
            // Register observers for client events
            .add_observer(buffer_player_input)
            // Systems
            .add_systems(Startup, setup_networking)
            .add_systems(
                PreUpdate,
                (handle_client_connections, handle_client_disconnections)
                    .in_set(ServerSystems::Receive)
                    .run_if(in_state(ServerState::Running)),
            )
            .add_systems(
                FixedUpdate,
                (process_buffered_inputs, simulate_physics, apply_velocity)
                    .chain()
                    .run_if(in_state(ServerState::Running))
                    .run_if(in_state(GameplayState::Unpaused)),
            )
            .add_systems(OnExit(ServerState::Running), save_world);
    }
}

/// Server gameplay state (pause/unpause).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameplayState {
    #[default]
    Unpaused,
    Paused,
}

/// Placeholder for world save system.
fn save_world() {
    info!(target: LOG_SERVER, "Saving world...");
    // TODO: Implement world saving
}
