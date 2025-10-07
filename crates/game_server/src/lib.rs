//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - `aether` (dedicated server binary)
//! - `EmbeddedServer` (client-hosted server in `forge_of_stories`)
//!
//! By centralizing server logic here, we ensure both deployment modes run identical
//! gameplay code, preventing desync and reducing maintenance overhead.

use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;
use bevy::prelude::*;
use server::transport::quic::QuicServerTransport;
use shared::transport::LoopbackServerTransport;

pub mod movement;
pub mod network;
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
/// use game_server::ServerLogicPlugin;
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

// ==============================================================================
// GameServer - Unified Server Implementation
// ==============================================================================

/// Unified game server implementation that can run in different modes.
///
/// This is the core server that handles all gameplay logic, physics, and state management.
/// It can be instantiated in different modes:
/// - **EmbeddedQuic**: Runs in the client process with loopback for the host + QUIC for remote clients
/// - **EmbeddedSteam**: Runs in the client process with loopback for the host + Steam for remote clients
/// - **DedicatedQuic**: Runs standalone with only QUIC transport
/// - **DedicatedSteam**: Runs standalone with only Steam transport
pub struct GameServer {
    /// Server operational mode
    mode: ServerMode,

    /// Separate Bevy World for server-side ECS
    world: World,

    /// Server tick schedule (runs at fixed rate, e.g., 20 TPS)
    schedule: Schedule,

    /// Current server state
    state: ServerState,
}

/// Server operational mode defining which transports are active.
pub enum ServerMode {
    /// Embedded mode: Loopback transport for the host player + QUIC transport for remote clients.
    /// Used when a player hosts a game from the client application via LAN/WAN.
    EmbeddedQuic {
        loopback: LoopbackServerTransport,
        external: QuicServerTransport,
    },

    /// Embedded mode: Loopback transport for the host player + Steam transport for remote clients.
    /// Used when a player hosts a game via Steam.
    #[cfg(feature = "steamworks")]
    EmbeddedSteam {
        loopback: LoopbackServerTransport,
        external: server::transport::SteamServerTransport,
    },

    /// Dedicated mode with QUIC: Only QUIC transport, no local player.
    /// Used for standalone dedicated servers accessible via IP:Port.
    DedicatedQuic { external: QuicServerTransport },

    /// Dedicated mode with Steam: Only Steam transport, no local player.
    /// Used for standalone dedicated servers accessible via Steam.
    #[cfg(feature = "steamworks")]
    DedicatedSteam {
        external: server::transport::SteamServerTransport,
    },
}

/// Current lifecycle state of the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is initializing
    Starting,
    /// Server is running and processing ticks
    Running,
    /// Server is paused (embedded singleplayer only)
    Paused,
    /// Server is shutting down gracefully
    ShuttingDown,
    /// Server has stopped
    Stopped,
}

impl GameServer {
    /// Internal helper to initialize the server world and schedule.
    ///
    /// This sets up all resources, system sets, and systems needed for server gameplay.
    fn initialize_server(world: &mut World, schedule: &mut Schedule) {
        // Insert resources
        world.insert_resource(ServerConfig::default());
        world.insert_resource(movement::PlayerInputQueue::default());
        world.insert_resource(world::PlayerColorAssigner::default());

        // Configure system sets for server pipeline
        schedule.configure_sets(
            (
                ServerSet::Input,
                ServerSet::Simulation,
                ServerSet::Replication,
                ServerSet::Output,
            )
                .chain(),
        );

        // World initialization systems
        schedule.add_systems(world_setup::initialize_world);
        schedule.add_systems(world_setup::spawn_world.after(world_setup::initialize_world));
        schedule.add_systems(
            world_setup::mark_world_initialized
                .run_if(resource_exists::<world_setup::WorldInitialized>)
                .in_set(ServerSet::Simulation),
        );

        // Movement systems
        schedule.add_systems(movement::process_player_input.in_set(ServerSet::Input));
        schedule.add_systems(movement::apply_velocity.in_set(ServerSet::Simulation));

        // Server systems
        schedule.add_systems(systems::heartbeat_system.in_set(ServerSet::Simulation));
    }

    /// Creates a new embedded server instance with QUIC transport.
    ///
    /// Use this when the client application wants to host a game over LAN/WAN.
    /// The loopback transport handles the local host player, while QUIC handles remote clients.
    ///
    /// # Arguments
    /// * `loopback` - Loopback transport for the host player
    /// * `quic` - QUIC transport for remote clients
    pub fn start_embedded_quic(
        loopback: LoopbackServerTransport,
        external: QuicServerTransport,
    ) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        Self::initialize_server(&mut world, &mut schedule);

        Self {
            mode: ServerMode::EmbeddedQuic { loopback, external },
            world,
            schedule,
            state: ServerState::Starting,
        }
    }

    /// Creates a new embedded server instance with Steam transport.
    ///
    /// Use this when the client application wants to host a game via Steam.
    /// The loopback transport handles the local host player, while Steam handles remote clients.
    ///
    /// # Arguments
    /// * `loopback` - Loopback transport for the host player
    /// * `external` - Steam transport for remote clients
    #[cfg(feature = "steamworks")]
    pub fn start_embedded_steam(
        loopback: LoopbackServerTransport,
        external: server::transport::SteamServerTransport,
    ) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // TODO: Initialize server systems, resources, world state

        Self {
            mode: ServerMode::EmbeddedSteam { loopback, external },
            world,
            schedule,
            state: ServerState::Starting,
        }
    }

    /// Creates a new dedicated server instance with QUIC transport.
    ///
    /// Use this for standalone dedicated servers accessible via IP:Port.
    /// No local player, only remote clients via QUIC.
    ///
    /// # Arguments
    /// * `external` - QUIC transport for all clients
    pub fn start_dedicated_quic(external: QuicServerTransport) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // TODO: Initialize server systems, resources, world state

        Self {
            mode: ServerMode::DedicatedQuic { external },
            world,
            schedule,
            state: ServerState::Starting,
        }
    }

    /// Advances the server simulation by one tick.
    ///
    /// This should be called at a fixed rate (e.g., 20 TPS) to process:
    /// - Incoming network messages
    /// - Player input
    /// - Physics simulation
    /// - Game logic
    /// - Outgoing state updates
    pub fn tick(&mut self) {
        if self.state != ServerState::Running {
            return;
        }

        // Run the server schedule (all gameplay systems)
        self.schedule.run(&mut self.world);

        // Apply deferred commands (spawning/despawning entities, etc.)
        self.world.flush();
    }

    /// Stops the server gracefully.
    ///
    /// This will:
    /// - Disconnect all clients
    /// - Save world state (if applicable)
    /// - Clean up resources
    pub fn stop(&mut self) {
        self.state = ServerState::ShuttingDown;

        // TODO: Implement proper cleanup:
        // - Call transport.stop() to disconnect all clients
        // - Save world state if needed
        // - Clean up resources
        // This will be implemented when we integrate transport event handling

        // Clear the world
        self.world.clear_all();

        self.state = ServerState::Stopped;
    }

    /// Returns the current server state.
    pub fn state(&self) -> ServerState {
        self.state
    }

    /// Returns a reference to the server world (for inspection/debugging).
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the server world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

// ==============================================================================
// Server Configuration & Resources
// ==============================================================================

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
