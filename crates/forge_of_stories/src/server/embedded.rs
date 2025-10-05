//! Embedded server implementation for client-hosted gameplay.
//!
//! The `EmbeddedServer` runs a complete server instance within the client process,
//! using a separate Bevy `World` for server-side ECS. This enables:
//! - Singleplayer mode (loopback transport, zero network overhead)
//! - Multiplayer hosting (QUIC or Steam transport)
//! - Shared server logic with dedicated server (`aether`)

use bevy::ecs::prelude::*;
use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;
use bevy::time::Time;
use network_shared::transport::{LoopbackPair, TransportOrchestrator};

/// Configuration for how the embedded server should operate.
#[derive(Debug, Clone)]
pub enum ServerMode {
    /// In-memory loopback (singleplayer, no network).
    Loopback,
    /// QUIC transport (LAN/WAN multiplayer).
    Quic { bind_address: String, port: u16 },
    /// Steam P2P transport (Steam friends multiplayer).
    #[cfg(feature = "steamworks")]
    Steam {
        lobby_name: String,
        max_players: u32,
        is_public: bool,
    },
}

/// Current lifecycle state of the embedded server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is initializing.
    Starting,
    /// Server is running and processing ticks.
    Running,
    /// Server is paused (singleplayer only).
    Paused,
    /// Server is shutting down gracefully.
    ShuttingDown,
    /// Server has stopped.
    Stopped,
}

/// Error types for embedded server operations.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Server is not in the correct state: expected {expected:?}, got {actual:?}")]
    InvalidState {
        expected: ServerState,
        actual: ServerState,
    },
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Loopback transport error: {0}")]
    Loopback(#[from] network_shared::transport::LoopbackError),
}

/// Embedded server resource that runs in the client process.
///
/// The server maintains a separate Bevy `World` for server-side entities and systems,
/// completely isolated from the client's rendering and input systems.
#[derive(Resource)]
pub struct EmbeddedServer {
    /// Server operational mode.
    mode: ServerMode,

    /// Separate Bevy World for server-side ECS.
    world: World,

    /// Server tick schedule (runs at fixed rate, e.g., 20 TPS).
    schedule: Schedule,

    /// Current server state.
    state: ServerState,

    /// Loopback transport pair (only used in Loopback mode).
    loopback: Option<LoopbackPair>,
}

impl EmbeddedServer {
    /// Creates a new embedded server with the specified mode.
    ///
    /// # Arguments
    /// * `mode` - The server mode (Loopback, QUIC, or Steam)
    ///
    /// # Returns
    /// A new `EmbeddedServer` in the `Starting` state.
    pub fn new(mode: ServerMode) -> Result<Self, ServerError> {
        let world = World::new();
        let schedule = Schedule::default();

        // Create loopback transport if in loopback mode using orchestrator
        let loopback = match &mode {
            ServerMode::Loopback => Some(TransportOrchestrator::create_loopback_pair()),
            _ => None,
        };

        let mut server = Self {
            mode,
            world,
            schedule,
            state: ServerState::Starting,
            loopback,
        };

        // Initialize the server
        server.initialize()?;

        Ok(server)
    }

    /// Initializes the server world and starts the transport.
    fn initialize(&mut self) -> Result<(), ServerError> {
        // Initialize server world with server logic resources
        use server_logic::{movement, world_setup, ServerConfig};

        self.world.insert_resource(ServerConfig::default());
        self.world.insert_resource(movement::PlayerInputQueue::default());
        self.world
            .insert_resource(Time::<bevy::time::Fixed>::from_hz(20.0));

        // Run world initialization (normally run at Startup)
        world_setup::initialize_world_direct(&mut self.world);
        world_setup::spawn_world_direct(&mut self.world);

        // Add server systems to schedule
        self.schedule
            .add_systems((
                movement::process_player_input,
                movement::apply_velocity,
                server_logic::systems::heartbeat_system,
            ));

        // Start the appropriate transport
        match &self.mode {
            ServerMode::Loopback => {
                if let Some(loopback) = &mut self.loopback {
                    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
                    loopback.server.start(tx)?;
                }
            }
            ServerMode::Quic { bind_address, port } => {
                // TODO: Initialize QUIC server transport
                return Err(ServerError::Config(format!(
                    "QUIC mode not yet implemented (would bind to {}:{})",
                    bind_address, port
                )));
            }
            #[cfg(feature = "steamworks")]
            ServerMode::Steam { .. } => {
                // TODO: Initialize Steam server transport
                return Err(ServerError::Config(
                    "Steam mode not yet implemented".to_string(),
                ));
            }
        }

        self.state = ServerState::Running;
        Ok(())
    }

    /// Ticks the server world, running all server-side systems.
    ///
    /// Should be called from the client's `FixedUpdate` schedule.
    pub fn tick(&mut self) {
        if self.state != ServerState::Running {
            return;
        }

        // Run the server schedule
        self.schedule.run(&mut self.world);

        // Process network events
        match &self.mode {
            ServerMode::Loopback => {
                if let Some(loopback) = &mut self.loopback {
                    // Poll for incoming messages
                    let _events = loopback.server.poll_events();
                    // TODO: Process events and update world state
                }
            }
            _ => {
                // TODO: Process QUIC/Steam network events
            }
        }
    }

    /// Pauses the server (singleplayer only).
    ///
    /// # Errors
    /// Returns an error if the server is not in loopback mode or not running.
    pub fn pause(&mut self) -> Result<(), ServerError> {
        if !matches!(self.mode, ServerMode::Loopback) {
            return Err(ServerError::Config(
                "Only loopback servers can be paused".to_string(),
            ));
        }

        if self.state != ServerState::Running {
            return Err(ServerError::InvalidState {
                expected: ServerState::Running,
                actual: self.state,
            });
        }

        self.state = ServerState::Paused;
        Ok(())
    }

    /// Resumes a paused server.
    ///
    /// # Errors
    /// Returns an error if the server is not paused.
    pub fn resume(&mut self) -> Result<(), ServerError> {
        if self.state != ServerState::Paused {
            return Err(ServerError::InvalidState {
                expected: ServerState::Paused,
                actual: self.state,
            });
        }

        self.state = ServerState::Running;
        Ok(())
    }

    /// Shuts down the server gracefully.
    pub fn shutdown(&mut self) -> Result<(), ServerError> {
        self.state = ServerState::ShuttingDown;

        // Stop the transport
        match &mut self.mode {
            ServerMode::Loopback => {
                if let Some(loopback) = &mut self.loopback {
                    loopback.server.stop();
                }
            }
            _ => {
                // TODO: Stop QUIC/Steam transport
            }
        }

        self.state = ServerState::Stopped;
        Ok(())
    }

    /// Returns the current server state.
    pub fn state(&self) -> ServerState {
        self.state
    }

    /// Returns the server mode.
    pub fn mode(&self) -> &ServerMode {
        &self.mode
    }

    /// Returns a reference to the server world (read-only).
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the server world.
    ///
    /// Use with caution - direct world manipulation can break server logic.
    /// Primarily intended for admin commands and debugging.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Returns the loopback transport pair (if in loopback mode).
    ///
    /// This allows the client to connect to the embedded server.
    pub fn loopback_pair(&mut self) -> Option<&mut LoopbackPair> {
        self.loopback.as_mut()
    }

    /// Sends player input to the server for processing.
    ///
    /// In loopback mode, this directly queues the input for the next server tick.
    /// In networked modes, this would send it over the transport.
    pub fn send_player_input(&mut self, player_id: u64, input: server_logic::movement::PlayerInput) {
        use server_logic::movement::PlayerInputQueue;

        if let Some(mut queue) = self.world.get_resource_mut::<PlayerInputQueue>() {
            queue.inputs.insert(player_id, input);
        }
    }
}

/// System that ticks the embedded server.
///
/// Should be added to the client's `FixedUpdate` schedule.
pub fn tick_embedded_server(mut server: ResMut<EmbeddedServer>) {
    server.tick();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_loopback_server() {
        let server = EmbeddedServer::new(ServerMode::Loopback).unwrap();
        assert_eq!(server.state(), ServerState::Running);
        assert!(matches!(server.mode(), ServerMode::Loopback));
    }

    #[test]
    fn test_server_lifecycle() {
        let mut server = EmbeddedServer::new(ServerMode::Loopback).unwrap();

        // Initial state
        assert_eq!(server.state(), ServerState::Running);

        // Pause
        server.pause().unwrap();
        assert_eq!(server.state(), ServerState::Paused);

        // Resume
        server.resume().unwrap();
        assert_eq!(server.state(), ServerState::Running);

        // Shutdown
        server.shutdown().unwrap();
        assert_eq!(server.state(), ServerState::Stopped);
    }

    #[test]
    fn test_loopback_transport_available() {
        let mut server = EmbeddedServer::new(ServerMode::Loopback).unwrap();
        assert!(server.loopback_pair().is_some());
    }

    #[test]
    fn test_tick_does_nothing_when_paused() {
        let mut server = EmbeddedServer::new(ServerMode::Loopback).unwrap();
        server.pause().unwrap();

        // Tick should do nothing when paused
        server.tick();
        assert_eq!(server.state(), ServerState::Paused);
    }

    #[test]
    fn test_pause_only_works_in_loopback_mode() {
        let server = EmbeddedServer::new(ServerMode::Quic {
            bind_address: "127.0.0.1".to_string(),
            port: 7777,
        });

        // QUIC mode not implemented yet, so this should fail
        assert!(server.is_err());
    }

    #[test]
    fn test_world_access() {
        let server = EmbeddedServer::new(ServerMode::Loopback).unwrap();

        // Read-only access
        let _world = server.world();

        // Mutable access (for admin commands)
        let mut server = server;
        let _world_mut = server.world_mut();
    }

    #[test]
    fn test_world_initialization() {
        use server_logic::world::{GroundPlane, Player, PlayerColorAssigner};
        use server_logic::world_setup::WorldInitialized;

        let mut server = EmbeddedServer::new(ServerMode::Loopback).unwrap();

        // Verify world resources were initialized
        assert!(
            server.world().contains_resource::<PlayerColorAssigner>(),
            "PlayerColorAssigner resource should exist"
        );
        assert!(
            server.world().contains_resource::<WorldInitialized>(),
            "WorldInitialized resource should exist"
        );

        // Verify ground plane entity was spawned
        let world = server.world_mut();
        let ground_plane_count = world.query::<&GroundPlane>().iter(world).count();
        assert_eq!(
            ground_plane_count, 1,
            "Exactly one ground plane should be spawned"
        );

        // Verify test player was spawned
        let player_count = world.query::<&Player>().iter(world).count();
        assert_eq!(player_count, 1, "Exactly one test player should be spawned");

        // Verify we have at least 2 entities (ground plane + player)
        assert!(
            server.world().entities().len() >= 2,
            "World should contain at least 2 entities"
        );
    }
}
