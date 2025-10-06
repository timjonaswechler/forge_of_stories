//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - `aether` (dedicated server binary)
//! - `EmbeddedServer` (client-hosted server in `forge_of_stories`)
//!
//! By centralizing server logic here, we ensure both deployment modes run identical
//! gameplay code, preventing desync and reducing maintenance overhead.

use bevy::prelude::*;

pub mod movement;
pub mod protocol;
pub mod savegame;
pub mod systems;
pub mod world;
pub mod world_setup;

/// Plugin bundle containing all server-side gameplay systems.
///
/// This should be added to any Bevy App that needs to run server logic,
/// whether it's a dedicated server or an embedded server.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use server_logic::ServerLogicPlugin;
///
/// App::new()
///     .add_plugins(ServerLogicPlugin)
///     .run();
/// ```
pub struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add server tick rate configuration
            .insert_resource(ServerConfig::default())
            .insert_resource(movement::PlayerInputQueue::default())
            // Configure system sets for server pipeline
            .configure_sets(
                FixedUpdate,
                (
                    ServerSet::Input,
                    ServerSet::Simulation,
                    ServerSet::Replication,
                    ServerSet::Output,
                )
                    .chain(),
            )
            // World initialization (runs once at startup)
            .add_systems(Startup, world_setup::initialize_world)
            .add_systems(
                Startup,
                world_setup::spawn_world.after(world_setup::initialize_world),
            )
            .add_systems(
                FixedUpdate,
                world_setup::mark_world_initialized
                    .run_if(resource_exists::<world_setup::WorldInitialized>)
                    .in_set(ServerSet::Simulation),
            )
            // Movement systems
            .add_systems(
                FixedUpdate,
                movement::process_player_input.in_set(ServerSet::Input),
            )
            .add_systems(
                FixedUpdate,
                movement::apply_velocity.in_set(ServerSet::Simulation),
            )
            // Server systems
            .add_systems(
                FixedUpdate,
                systems::heartbeat_system.in_set(ServerSet::Simulation),
            );
    }
}

/// System sets for server execution pipeline.
///
/// These run in `FixedUpdate` at a fixed tick rate (default: 20 TPS).
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ServerSet {
    /// Process incoming client inputs and commands.
    Input,
    /// Run gameplay simulation (physics, AI, game rules).
    Simulation,
    /// Build state snapshots/deltas for clients.
    Replication,
    /// Send state updates to clients.
    Output,
}

/// Server configuration resource.
#[derive(Resource, Debug, Clone)]
pub struct ServerConfig {
    /// Target ticks per second (TPS).
    pub tick_rate: f64,
    /// Maximum number of connected clients.
    pub max_clients: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            tick_rate: 20.0,
            max_clients: 16,
        }
    }
}

/// Server statistics resource (optional, for monitoring).
#[derive(Resource, Debug, Default)]
pub struct ServerStats {
    /// Total number of ticks processed.
    pub total_ticks: u64,
    /// Number of currently connected clients.
    pub connected_clients: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(ServerLogicPlugin);

        // Verify config resource exists
        assert!(app.world().contains_resource::<ServerConfig>());
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.tick_rate, 20.0);
        assert_eq!(config.max_clients, 16);
    }
}
