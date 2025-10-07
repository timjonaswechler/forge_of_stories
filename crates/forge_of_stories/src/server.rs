//! Embedded server implementation for client-hosted gameplay.
//!
//! The `EmbeddedServer` is a thin wrapper around `GameServer` that manages
//! the server lifecycle within the client process. This enables:
//! - Singleplayer mode (loopback transport, zero network overhead)
//! - Multiplayer hosting (QUIC or Steam transport)
//! - Shared server logic with dedicated server
//! Embedded server module for client-hosted gameplay.

use bevy::ecs::prelude::*;
use game_server::{GameServer, ServerState};
use server::ServerEndpointConfiguration;
use server::transport::quic::QuicServerTransport;
use shared::transport::{
    LoopbackClientTransport, LoopbackPair, LoopbackServerTransport, TransportOrchestrator,
};
use tracing::{debug, info};

/// Configuration for how the embedded server should operate.
#[derive(Debug, Clone)]
pub enum ServerMode {
    /// In-memory loopback (singleplayer, no network).
    Loopback,
    /// QUIC transport (LAN/WAN multiplayer).
    Quic { bind_address: String, port: u16 },
    /// Steam P2P transport (Steam friends multiplayer).
    Steam {
        lobby_name: String,
        max_players: u32,
        is_public: bool,
    },
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
    Loopback(#[from] shared::transport::LoopbackError),
    #[error("QUIC transport error: {0}")]
    QuicTransport(String),
}

/// Embedded server resource that runs in the client process.
///
/// This is a thin wrapper around `GameServer` that provides a simpler
/// API for client-hosted gameplay.
#[derive(Resource)]
pub struct EmbeddedServer {
    /// The underlying game server
    server: GameServer,

    /// Server operational mode
    mode: ServerMode,

    /// Loopback client transport (for host player connection)
    /// We only store the client side since the server side is owned by GameServer
    loopback_client: Option<shared::transport::LoopbackClientTransport>,
}

impl EmbeddedServer {
    /// Creates a new embedded server with the specified mode.
    ///
    /// # Arguments
    /// * `mode` - The server mode (Loopback, QUIC, or Steam)
    ///
    /// # Returns
    /// A new `EmbeddedServer` ready to be started.
    pub fn new(mode: ServerMode) -> Result<Self, ServerError> {
        match &mode {
            ServerMode::Loopback => Self::new_loopback(),
            ServerMode::Quic { bind_address, port } => Self::new_quic(bind_address.clone(), *port),
            ServerMode::Steam { .. } => {
                // TODO: Implement Steam transport
                Err(ServerError::Config(
                    "Steam transport not yet implemented".into(),
                ))
            }
        }
    }

    /// Creates a new embedded server in loopback-only mode (singleplayer).
    fn new_loopback() -> Result<Self, ServerError> {
        info!("Creating embedded server in loopback mode (singleplayer)");

        // Create loopback transport pair and destructure it
        let loopback_pair = TransportOrchestrator::create_loopback_pair();
        let LoopbackPair {
            client: host,
            server: loopback_server,
        } = loopback_pair;

        // Create GameServer in embedded mode with the server side
        let game_server = GameServer::start_embedded(loopback_server, None);

        Ok(Self {
            server: game_server,
            mode: ServerMode::Loopback,
            loopback_client: Some(host),
        })
    }

    /// Creates a new embedded server with QUIC transport (LAN/WAN multiplayer).
    fn new_quic(bind_address: String, port: u16) -> Result<Self, ServerError> {
        info!(
            "Creating embedded server with QUIC transport on {}:{}",
            bind_address, port
        );

        // Create loopback transport for the host player
        let loopback_pair = TransportOrchestrator::create_loopback_pair();
        let LoopbackPair {
            client: host,
            server: loopback_server,
        } = loopback_pair;

        // Create QUIC transport for remote clients
        let endpoint_config =
            ServerEndpointConfiguration::from_string(&format!("{}:{}", bind_address, port))
                .map_err(|e| ServerError::QuicTransport(format!("Invalid bind address: {}", e)))?;

        // Create channel configuration for gameplay
        let channels = game_server::protocol::channels::create_gameplay_channels();

        // Create transport capabilities (QUIC supports all features)
        let capabilities = shared::TransportCapabilities::new(
            true, // reliable_streams
            true, // unreliable_streams
            true, // datagrams
            8,    // max_channels
        );

        let quic = game_server::ExternalTransport::Quic(game_server::QuicTransport::new(
            endpoint_config,
            channels,
            capabilities,
        ));

        // Create GameServer in embedded mode
        let server = GameServer::start_embedded(loopback_server, Some(quic));

        Ok(Self {
            server,
            mode: ServerMode::Quic { bind_address, port },
            loopback_client: Some(host),
        })
    }
    /// Stops external transport
    pub fn stop_external(&mut self) {
        self.server.remove_external();
    }

    /// Advances the server simulation by one tick.
    ///
    /// This should be called at a fixed rate (e.g., 20 TPS).
    pub fn tick(&mut self) {
        self.server.tick();
    }

    /// Stops the server gracefully.
    pub fn stop(&mut self) {
        self.server.remove_external();
        self.server.stop();
    }

    /// Returns the current server state.
    pub fn state(&self) -> ServerState {
        self.server.state()
    }

    /// Returns the server mode.
    pub fn mode(&self) -> &ServerMode {
        &self.mode
    }

    /// Returns a reference to the loopback pair (for host player connection).
    pub fn loopback_client(&self) -> Option<&LoopbackClientTransport> {
        self.loopback_client.as_ref()
    }

    /// Returns a reference to the server world (for inspection/debugging).
    pub fn world(&self) -> &bevy::ecs::world::World {
        self.server.world()
    }

    /// Returns a mutable reference to the server world.
    pub fn world_mut(&mut self) -> &mut bevy::ecs::world::World {
        self.server.world_mut()
    }

    /// Pauses the server (singleplayer only).
    ///
    /// This prevents the server from processing ticks until resumed.
    pub fn pause(&mut self) {
        // TODO: Implement pause logic in GameServer
        debug!("Pause requested (not yet implemented)");
    }

    /// Resumes the server after being paused.
    pub fn resume(&mut self) {
        // TODO: Implement resume logic in GameServer
        debug!("Resume requested (not yet implemented)");
    }

    /// Sends player input to the server.
    ///
    /// This enqueues the input to be processed on the next server tick.
    pub fn send_player_input(&mut self, player_id: u64, input: game_server::movement::PlayerInput) {
        use game_server::movement::PlayerInputQueue;

        // Get or insert the input queue
        if let Some(mut queue) = self
            .server
            .world_mut()
            .get_resource_mut::<PlayerInputQueue>()
        {
            queue.inputs.insert(player_id, input);
        }
    }

    /// Loads a world from file.
    pub fn load_world(&mut self, path: &std::path::Path) -> Result<(), ServerError> {
        use game_server::savegame::load_world_from_file;

        load_world_from_file(self.server.world_mut(), path)
            .map_err(|e| ServerError::Config(format!("Failed to load world: {}", e)))?;

        info!("Loaded world from {:?}", path);
        Ok(())
    }

    /// Saves the world to file.
    pub fn save_world(&mut self, path: &std::path::Path) -> Result<(), ServerError> {
        use game_server::savegame::save_world_to_file;

        save_world_to_file(self.server.world_mut(), path)
            .map_err(|e| ServerError::Config(format!("Failed to save world: {}", e)))?;

        info!("Saved world to {:?}", path);
        Ok(())
    }
}

/// System function to tick the embedded server.
///
/// Add this to your Bevy app's FixedUpdate schedule to automatically tick the server.
pub fn tick_embedded_server(mut server: ResMut<EmbeddedServer>) {
    server.tick();
}
