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
use server::transport::{ServerTransport, SteamServerTransport};
use shared::ClientId;
use shared::transport::LoopbackServerTransport;
use uuid::Uuid;

/// The host player always uses this client ID (UUID with all zeros)
pub const HOST_CLIENT_ID: ClientId = Uuid::nil();

pub mod movement;
pub mod network;
pub mod protocol;
pub mod savegame;
pub mod systems;
pub mod world;
pub mod world_setup;

// Re-export transport types for use in other crates
#[cfg(feature = "steamworks")]
pub use server::transport::SteamServerTransport;
pub use server::transport::quic::QuicServerTransport as QuicTransport;

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
                    ServerSet::Receive,
                    ServerSet::Simulation,
                    ServerSet::Replication,
                    ServerSet::Transmit,
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
                movement::process_player_input.in_set(ServerSet::Receive),
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

/// Resource holding the transport event receiver channel.
///
/// The GameServer polls this in tick() to receive events from all transports.
#[derive(Resource)]
struct TransportEventReceiver(tokio::sync::mpsc::UnboundedReceiver<shared::TransportEvent>);

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

pub enum ExternalTransport {
    Quic(QuicServerTransport),
    Steam(SteamServerTransport),
}

/// Server operational mode defining which transports are active.
pub enum ServerMode {
    /// Embedded mode: Loopback transport for the host player + QUIC transport for remote clients.
    /// Used when a player hosts a game from the client application via LAN/WAN.
    Embedded {
        loopback: LoopbackServerTransport,
        external: Option<ExternalTransport>,
    },

    /// Dedicated mode with QUIC: Only QUIC transport, no local player.
    /// Used for standalone dedicated servers accessible via IP:Port.
    DedicatedQuic { external: QuicServerTransport },
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
        // Insert Bevy core resources
        world.insert_resource(Time::<Fixed>::from_hz(20.0)); // 20 TPS server tick rate
        world.insert_resource(Time::<Real>::default());

        // Insert game resources
        world.insert_resource(ServerConfig::default());
        world.insert_resource(movement::PlayerInputQueue::default());
        world.insert_resource(world::PlayerColorAssigner::default());

        // Insert network resources
        world.insert_resource(network::NetworkEvents::default());
        world.insert_resource(network::OutgoingMessages::default());
        world.insert_resource(network::ConnectedClients::default());

        // Initialize world directly (spawn ground, etc.)
        world_setup::initialize_world_direct(world);
        world_setup::spawn_world_direct(world);

        // Configure system sets for server pipeline
        schedule.configure_sets(
            (
                ServerSet::Receive,
                ServerSet::Simulation,
                ServerSet::Replication,
                ServerSet::Transmit,
            )
                .chain(),
        );

        // Network systems
        schedule.add_systems(network::process_network_events.in_set(ServerSet::Receive));
        schedule.add_systems(network::broadcast_world_state.in_set(ServerSet::Replication));

        // Movement systems
        schedule.add_systems(movement::process_player_input.in_set(ServerSet::Receive));
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
    pub fn start_embedded(
        mut loopback: LoopbackServerTransport,
        mut external: Option<ExternalTransport>,
    ) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        Self::initialize_server(&mut world, &mut schedule);

        // Create a channel for transport events
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

        // Store the event receiver in a resource so we can poll it in tick()
        world.insert_resource(TransportEventReceiver(event_rx));

        // Start the loopback transport ONLY if we have external transport (LAN mode)
        // In singleplayer, we don't start it because we inject the event manually
        let is_singleplayer = external.is_none();
        if !is_singleplayer {
            if let Err(e) = loopback.start(event_tx.clone()) {
                tracing::error!("Failed to start loopback transport: {:?}", e);
            } else {
                tracing::info!("Started loopback transport for LAN host");
            }
        }

        // Start external transport if present
        if let Some(ref mut ext) = external {
            match ext {
                ExternalTransport::Quic(quic) => {
                    if let Err(e) = quic.start(event_tx.clone()) {
                        tracing::error!("Failed to start QUIC transport: {:?}", e);
                        // Return error instead of continuing
                        panic!("Failed to start QUIC transport: {:?}", e);
                    } else {
                        tracing::info!("QUIC transport started successfully");
                    }
                }
                ExternalTransport::Steam(_steam) => {
                    tracing::error!("Steam transport not yet implemented");
                }
            }
        }

        // In singleplayer mode (no external transport), inject a PeerConnected event
        // for the loopback client since there's no actual network connection
        if external.is_none() {
            if let Some(mut events) = world.get_resource_mut::<network::NetworkEvents>() {
                events.events.push(shared::TransportEvent::PeerConnected {
                    client: HOST_CLIENT_ID,
                });
                tracing::info!(
                    "Injected PeerConnected event for singleplayer loopback client (ID: {})",
                    HOST_CLIENT_ID
                );
            }
        }

        tracing::info!("GameServer starting in Embedded mode");

        Self {
            mode: ServerMode::Embedded { loopback, external },
            world,
            schedule,
            state: ServerState::Running, // Start in Running state immediately
        }
    }

    /// Adds an external transport (QUIC or Steam) to an embedded server.
    ///
    /// This allows the host to open their server to remote players.
    /// Only works on Embedded mode servers.
    pub fn add_external(mut self, external: ExternalTransport) -> Self {
        match self.mode {
            ServerMode::Embedded { loopback, .. } => {
                self.mode = ServerMode::Embedded {
                    loopback,
                    external: Some(external),
                };
            }
            ServerMode::DedicatedQuic { .. } => {
                tracing::warn!("Cannot add external transport to dedicated server");
            }
        }
        self
    }

    /// Removes the external transport from an embedded server.
    ///
    /// This closes the server to remote players, keeping only the loopback connection.
    /// All external clients will be disconnected.
    pub fn remove_external(&mut self) {
        match &mut self.mode {
            ServerMode::Embedded { external, .. } => {
                if external.is_some() {
                    tracing::error!("Disconnect all clients over the closing connection first");
                    // TODO: Properly disconnect all external clients before removing transport
                }
                *external = None;
            }
            ServerMode::DedicatedQuic { .. } => {
                tracing::warn!("Cannot remove external transport from dedicated server");
            }
        }
    }

    /// Creates a new dedicated server instance with QUIC transport.
    ///
    /// Use this for standalone dedicated servers accessible via IP:Port.
    /// No local player, only remote clients via QUIC.
    ///
    /// # Arguments
    /// * `external` - QUIC transport for all clients
    pub fn start_dedicated(external: QuicServerTransport) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        Self::initialize_server(&mut world, &mut schedule);

        tracing::info!("GameServer starting in Dedicated mode");

        Self {
            mode: ServerMode::DedicatedQuic { external },
            world,
            schedule,
            state: ServerState::Running, // Start in Running state immediately
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

        // Poll transport events and add them to NetworkEvents
        let mut incoming_events = Vec::new();
        if let Some(mut event_receiver) = self.world.get_resource_mut::<TransportEventReceiver>() {
            // Drain all available events from the transport channel
            while let Ok(event) = event_receiver.0.try_recv() {
                incoming_events.push(event);
            }
        }
        // Add events to NetworkEvents
        if !incoming_events.is_empty() {
            if let Some(mut network_events) =
                self.world.get_resource_mut::<network::NetworkEvents>()
            {
                network_events.events.extend(incoming_events);
            }
        }

        // Update time resources
        if let Some(mut time_fixed) = self.world.get_resource_mut::<Time<Fixed>>() {
            let timestep = time_fixed.timestep();
            time_fixed.advance_by(timestep);
        }
        if let Some(mut time_real) = self.world.get_resource_mut::<Time<Real>>() {
            time_real.update();
        }

        // Run the server schedule (all gameplay systems)
        self.schedule.run(&mut self.world);

        // Apply deferred commands (spawning/despawning entities, etc.)
        self.world.flush();

        // Send outgoing messages to clients
        // First, extract messages and connected clients list
        let messages: Vec<_> = if let Some(mut outgoing) =
            self.world.get_resource_mut::<network::OutgoingMessages>()
        {
            outgoing.messages.drain(..).collect()
        } else {
            Vec::new()
        };

        let connected_client_ids: Vec<Uuid> = self
            .world
            .get_resource::<network::ConnectedClients>()
            .map(|cc| cc.clients.clone())
            .unwrap_or_default();

        // Now send messages without borrowing world
        for (target_client, message) in messages {
            // Serialize the message
            let payload = match bincode::serde::encode_to_vec(&message, bincode::config::standard())
            {
                Ok(bytes) => bytes::Bytes::from(bytes),
                Err(e) => {
                    tracing::error!("Failed to serialize message: {:?}", e);
                    continue;
                }
            };

            // Determine which channel to use (for now, use channel 0)
            let channel = 0;

            let out_msg = shared::OutgoingMessage {
                channel,
                payload: payload.clone(),
            };

            // Log what we're sending
            if let Some(target) = target_client {
                tracing::debug!(
                    "Sending message to client {} on channel {}, {} bytes",
                    target,
                    channel,
                    payload.len()
                );
            } else {
                tracing::debug!(
                    "Broadcasting message to all clients on channel {}, {} bytes",
                    channel,
                    payload.len()
                );
            }

            // Send to target client(s)
            match &mut self.mode {
                ServerMode::Embedded { loopback, external } => {
                    if let Some(client_id) = target_client {
                        // Send to specific client
                        // Check if it's the loopback client (host)
                        if client_id == HOST_CLIENT_ID {
                            if let Err(e) = loopback.send(client_id, out_msg.clone()) {
                                tracing::warn!(
                                    "Failed to send to loopback client {}: {:?}",
                                    client_id,
                                    e
                                );
                            }
                        } else {
                            // Send via external transport
                            if let Some(ext) = external {
                                let send_result = match ext {
                                    ExternalTransport::Quic(quic) => {
                                        quic.send(client_id, out_msg.clone())
                                    }
                                    ExternalTransport::Steam(_steam) => {
                                        tracing::warn!("Steam transport not yet implemented");
                                        continue;
                                    }
                                };
                                if let Err(e) = send_result {
                                    tracing::warn!(
                                        "Failed to send to external client {}: {:?}",
                                        client_id,
                                        e
                                    );
                                }
                            }
                        }
                    } else {
                        // Broadcast to all clients
                        // Send to loopback client (host)
                        if let Err(e) = loopback.send(HOST_CLIENT_ID, out_msg.clone()) {
                            tracing::warn!("Failed to broadcast to loopback client: {:?}", e);
                        }
                        // Send to all external clients
                        if let Some(ext) = external {
                            for &client_id in &connected_client_ids {
                                if client_id != HOST_CLIENT_ID {
                                    // Skip loopback client (host)
                                    let send_result = match ext {
                                        ExternalTransport::Quic(quic) => {
                                            quic.send(client_id, out_msg.clone())
                                        }
                                        ExternalTransport::Steam(_steam) => continue,
                                    };
                                    if let Err(e) = send_result {
                                        tracing::warn!(
                                            "Failed to broadcast to client {}: {:?}",
                                            client_id,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                ServerMode::DedicatedQuic { external } => {
                    if let Some(client_id) = target_client {
                        if let Err(e) = external.send(client_id, out_msg) {
                            tracing::warn!("Failed to send to client {}: {:?}", client_id, e);
                        }
                    } else {
                        // Broadcast to all clients
                        for &client_id in &connected_client_ids {
                            if let Err(e) = external.send(client_id, out_msg.clone()) {
                                tracing::warn!(
                                    "Failed to broadcast to client {}: {:?}",
                                    client_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
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
    Receive,
    /// Run gameplay simulation (physics, AI, game rules).
    Simulation,
    /// Build state snapshots/deltas for clients.
    Replication,
    /// Send state updates to clients.
    Transmit,
    /// Control-Plane (Separate)
    Control,
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
