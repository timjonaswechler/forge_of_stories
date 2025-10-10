//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - `aether` (dedicated server binary)
//! - `EmbeddedServer` (client-hosted server in `forge_of_stories`)
//!
//! By centralizing server logic here, we ensure both deployment modes run identical
//! gameplay code, preventing desync and reducing maintenance overhead.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;
use bevy::prelude::*;
use server::transport::quic::QuicServerTransport;
use server::transport::{ServerTransport, SteamServerTransport};
use shared::transport::{
    LOOPBACK_CLIENT_ID, LoopbackClientTransport, LoopbackServerTransport, TransportPayload,
};
use shared::{ClientId, TransportError, TransportEvent};
use std::time::SystemTime;
use tracing::error;
use uuid::Uuid;

/// The host player always uses this client ID (UUID with all zeros)
pub const HOST_CLIENT_ID: ClientId = LOOPBACK_CLIENT_ID;

mod error;
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

// ==============================================================================
// Server Control & Commands
// ==============================================================================

/// Commands that can be sent to the server thread.
#[derive(Debug)]
pub enum ServerCommand {
    /// Stop the server gracefully
    Shutdown,
    /// Pause the server (singleplayer only)
    Pause,
    /// Resume the server after pausing
    Resume,
    /// Dynamisch External Transport hinzufügen
    AddExternal(ExternalTransport),
    /// External Transport entfernen
    RemoveExternal,
}

/// Handle to control a running server thread.
///
/// This is the interface between the Bevy app and the GameServer running in its own thread.
/// It provides methods to start, stop, and communicate with the server.
#[derive(Resource)]
pub struct ServerHandle {
    /// Thread handle for the server (None if not running)
    thread_handle: Option<JoinHandle<()>>,

    /// Channel to send commands to the server thread
    command_tx: crossbeam::channel::Sender<ServerCommand>,

    /// Loopback client transport for the host player
    /// This is taken out and inserted into the Bevy app
    loopback_client: Option<LoopbackClientTransport>,

    /// Current server state (shared with thread via Arc)
    state: Arc<AtomicServerState>,

    /// Server mode information (without owning transports)
    mode_info: ServerModeInfo,
}

/// Atomic version of ServerState for thread-safe access.
pub struct AtomicServerState {
    state: std::sync::atomic::AtomicU8,
}

impl AtomicServerState {
    fn new(state: ServerState) -> Self {
        Self {
            state: std::sync::atomic::AtomicU8::new(state as u8),
        }
    }

    fn load(&self) -> ServerState {
        match self.state.load(Ordering::Relaxed) {
            0 => ServerState::Starting,
            1 => ServerState::Running,
            2 => ServerState::Paused,
            3 => ServerState::ShuttingDown,
            4 => ServerState::Stopped,
            _ => ServerState::Stopped,
        }
    }

    fn store(&self, state: ServerState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }
}

impl ServerHandle {
    /// Starts a new embedded server in a separate thread.
    ///
    /// # Arguments
    /// * `loopback` - Loopback transport for the host player
    /// * `external` - Optional external transport (QUIC or Steam) for remote clients
    ///
    /// # Returns
    /// A `ServerHandle` to control the server thread, with the loopback client transport available.
    pub fn start_embedded(
        loopback_client: LoopbackClientTransport,
        loopback_server: LoopbackServerTransport,
        external: Option<ExternalTransport>,
    ) -> Self {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let state = Arc::new(AtomicServerState::new(ServerState::Starting));
        let state_clone = state.clone();

        let mode_info = if external.is_some() {
            ServerModeInfo::Multiplayer
        } else {
            ServerModeInfo::Singleplayer
        };

        let mode = ServerMode::Embedded {
            loopback: loopback_server,
            external,
        };

        // Spawn the server thread
        let thread_handle = thread::spawn(move || {
            let mut server = match GameServer::new_from_mode(mode) {
                Ok(server) => server,
                Err(err) => {
                    error!("Failed to start embedded server: {:?}", err);
                    return;
                }
            };
            server.run(command_rx, state_clone);
        });

        Self {
            thread_handle: Some(thread_handle),
            command_tx,
            loopback_client: Some(loopback_client),
            state,
            mode_info,
        }
    }

    /// Starts a new dedicated server in a separate thread.
    ///
    /// # Arguments
    /// * `external` - QUIC transport for all clients
    ///
    /// # Returns
    /// A `ServerHandle` to control the server thread.
    pub fn start_dedicated(external: QuicServerTransport) -> Self {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let state = Arc::new(AtomicServerState::new(ServerState::Starting));
        let state_clone = state.clone();

        let mode = ServerMode::DedicatedQuic { external };

        // Spawn the server thread
        let thread_handle = thread::spawn(move || {
            let mut server = match GameServer::new_from_mode(mode) {
                Ok(server) => server,
                Err(err) => {
                    error!("Failed to start dedicated server: {:?}", err);
                    return;
                }
            };
            server.run(command_rx, state_clone);
        });

        Self {
            thread_handle: Some(thread_handle),
            command_tx,
            loopback_client: None,
            state,
            mode_info: ServerModeInfo::Dedicated,
        }
    }

    /// Takes the loopback client transport (for use by the host player).
    ///
    /// This should be called once after starting an embedded server to connect the
    /// host player to the server via the loopback transport.
    pub fn take_loopback_client(&mut self) -> Option<LoopbackClientTransport> {
        self.loopback_client.take()
    }

    // In ServerHandle
    /// Adds an external transport (QUIC or Steam) to the server.
    ///
    /// This allows opening a singleplayer game to LAN/WAN multiplayer.
    pub fn add_external(&self, external: ExternalTransport) -> Result<(), String> {
        self.command_tx
            .send(ServerCommand::AddExternal(external))
            .map_err(|e| format!("Failed to send AddExternal command: {}", e))
    }

    /// Removes the external transport from the server.
    ///
    /// This closes the server to remote players, keeping only local connections.
    pub fn remove_external(&self) -> Result<(), String> {
        self.command_tx
            .send(ServerCommand::RemoveExternal)
            .map_err(|e| format!("Failed to send RemoveExternal command: {}", e))
    }

    /// Returns the current server state.
    pub fn state(&self) -> ServerState {
        self.state.load()
    }

    /// Returns the initial mode information used when starting the server.
    pub fn mode_info(&self) -> ServerModeInfo {
        self.mode_info
    }

    /// Sends a shutdown command to the server thread.
    pub fn shutdown(&self) {
        let _ = self.command_tx.send(ServerCommand::Shutdown);
    }

    /// Pauses the server (singleplayer only).
    pub fn pause(&self) {
        let _ = self.command_tx.send(ServerCommand::Pause);
    }

    /// Resumes the server after pausing.
    pub fn resume(&self) {
        let _ = self.command_tx.send(ServerCommand::Resume);
    }

    /// Waits for the server thread to finish (blocking).
    ///
    /// This should be called during shutdown to ensure clean termination.
    pub fn join(mut self) -> Result<(), String> {
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| "Server thread panicked".to_string())
        } else {
            Ok(())
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Send shutdown command if the thread is still running
        let _ = self.command_tx.send(ServerCommand::Shutdown);

        // Wait for thread to finish (with timeout)
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
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

    metrics: Metrics,
}

#[derive(Debug)]
pub enum ExternalTransport {
    Quic(QuicServerTransport),
    Steam(SteamServerTransport),
}

impl ExternalTransport {
    fn poll_events(&mut self, output: &mut Vec<TransportEvent>) {
        match self {
            ExternalTransport::Quic(quic) => quic.poll_events(output),
            ExternalTransport::Steam(steam) => steam.poll_events(output),
        }
    }

    fn send(&mut self, client: ClientId, payload: TransportPayload) -> Result<(), TransportError> {
        match self {
            ExternalTransport::Quic(quic) => quic.send(client, payload),
            ExternalTransport::Steam(steam) => steam.send(client, payload),
        }
    }

    fn broadcast_excluding(
        &mut self,
        exclude: &[ClientId],
        payload: TransportPayload,
    ) -> Result<(), TransportError> {
        match self {
            ExternalTransport::Quic(quic) => quic.broadcast_excluding(exclude, payload),
            ExternalTransport::Steam(steam) => steam.broadcast_excluding(exclude, payload),
        }
    }

    fn shutdown(&mut self) {
        match self {
            ExternalTransport::Quic(quic) => quic.shutdown(),
            ExternalTransport::Steam(_steam) => {}
        }
    }
}

fn warm_up_external(external: &mut ExternalTransport) {
    let mut discard = Vec::new();
    external.poll_events(&mut discard);
}

fn warm_up_quic(quic: &mut QuicServerTransport) {
    let mut discard = Vec::new();
    quic.poll_events(&mut discard);
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

/// Information about the server mode (without owning transports).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerModeInfo {
    /// Singleplayer (loopback only)
    Singleplayer,
    /// LAN/WAN multiplayer (loopback + external)
    Multiplayer,
    /// Dedicated server
    Dedicated,
}

/// Current lifecycle state of the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServerState {
    /// Server is initializing
    Starting = 0,
    /// Server is running and processing ticks
    Running = 1,
    /// Server is paused (embedded singleplayer only)
    Paused = 2,
    /// Server is shutting down gracefully
    ShuttingDown = 3,
    /// Server has stopped
    Stopped = 4,
}

/// Metrics for the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    pub receive_enter_time: SystemTime,
    pub receive_exit_time: SystemTime,
    pub simulation_enter_time: SystemTime,
    pub simulation_exit_time: SystemTime,
    pub replication_enter_time: SystemTime,
    pub replication_exit_time: SystemTime,
    pub transmit_enter_time: SystemTime,
    pub transmit_exit_time: SystemTime,
    pub control_enter_time: SystemTime,
    pub control_exit_time: SystemTime,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            receive_enter_time: SystemTime::now(),
            receive_exit_time: SystemTime::now(),
            simulation_enter_time: SystemTime::now(),
            simulation_exit_time: SystemTime::now(),
            replication_enter_time: SystemTime::now(),
            replication_exit_time: SystemTime::now(),
            transmit_enter_time: SystemTime::now(),
            transmit_exit_time: SystemTime::now(),
            control_enter_time: SystemTime::now(),
            control_exit_time: SystemTime::now(),
        }
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

// ==============================================================================
// Server Configuration & Resources
// ==============================================================================

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

    /// Creates a new GameServer from a ServerMode.
    ///
    /// This is an internal method used by ServerHandle to create the server instance
    /// in the server thread. Takes ownership of the transports.
    fn new_from_mode(mode: ServerMode) -> Result<Self, error::ServerError> {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        Self::initialize_server(&mut world, &mut schedule);

        // Start transports based on mode
        let mut mode = mode; // Make it mutable
        match &mut mode {
            ServerMode::Embedded {
                loopback: _,
                external,
            } => {
                tracing::info!("GameServer starting in Embedded mode");
                if let Some(ext) = external {
                    warm_up_external(ext);
                }
            }
            ServerMode::DedicatedQuic { external } => {
                tracing::info!("Dedicated QUIC server started successfully");
                warm_up_quic(external);
            }
        }

        Ok(Self {
            mode,
            world,
            schedule,
            state: ServerState::Starting,
            metrics: Metrics::default(),
        })
    }

    /// Adds an external transport (QUIC or Steam) to an embedded server.
    ///
    /// This allows the host to open their server to remote players.
    /// Only works on Embedded mode servers.
    pub fn add_external(mut self, external: ExternalTransport) -> Self {
        match self.mode {
            ServerMode::Embedded { loopback, .. } => {
                let mut transport = external;
                warm_up_external(&mut transport);
                self.mode = ServerMode::Embedded {
                    loopback,
                    external: Some(transport),
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

    /// Main server loop that runs in a separate thread.
    ///
    /// This function runs the server at a fixed tick rate (20 TPS) and processes commands
    /// from the command channel.
    ///
    /// # Arguments
    /// * `command_rx` - Receiver for server commands
    /// * `state` - Shared atomic state for thread-safe state updates
    pub fn run(
        &mut self,
        command_rx: crossbeam::channel::Receiver<ServerCommand>,
        state: Arc<AtomicServerState>,
    ) {
        tracing::info!("GameServer thread started");

        // Update state to Running
        self.state = ServerState::Running;
        state.store(ServerState::Running);

        // Target tick duration (20 TPS = 50ms per tick)
        let _tick_duration = Duration::from_millis(50);

        loop {
            let _tick_start = std::time::Instant::now();

            // Process commands from the control channel
            match command_rx.try_recv() {
                Ok(ServerCommand::Shutdown) => {
                    tracing::info!("Shutdown command received");
                    self.stop();
                    state.store(ServerState::Stopped);
                    break;
                }
                Ok(ServerCommand::Pause) => {
                    tracing::info!("Pause command received");
                    self.state = ServerState::Paused;
                    state.store(ServerState::Paused);
                }
                Ok(ServerCommand::Resume) => {
                    tracing::info!("Resume command received");
                    self.state = ServerState::Running;
                    state.store(ServerState::Running);
                }
                Ok(ServerCommand::AddExternal(mut transport)) => {
                    tracing::info!("AddExternal command received");
                    if let ServerMode::Embedded { external, .. } = &mut self.mode {
                        if let Some(mut existing) = external.take() {
                            tracing::warn!("Replacing existing external transport");
                            existing.shutdown();
                        }

                        warm_up_external(&mut transport);
                        *external = Some(transport);
                        tracing::info!("External transport attached");
                    } else {
                        tracing::error!("Cannot add external transport to dedicated server");
                    }
                }
                Ok(ServerCommand::RemoveExternal) => {
                    tracing::info!("RemoveExternal command received");
                    if let ServerMode::Embedded { external, .. } = &mut self.mode {
                        match external.take() {
                            Some(mut existing) => {
                                existing.shutdown();
                                tracing::info!("External transport removed");
                            }
                            None => {
                                tracing::warn!("No external transport to remove");
                            }
                        }
                    } else {
                        tracing::error!("Cannot remove external transport from dedicated server");
                    }
                }
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    // No command, continue
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    tracing::warn!("Command channel disconnected, shutting down");
                    self.stop();
                    state.store(ServerState::Stopped);
                    break;
                }
            }

            // Only tick if running
            if self.state == ServerState::Running {
                self.tick();
            }

            // Sleep for the remaining tick time
            // let tick_elapsed = tick_start.elapsed();
            // if tick_elapsed < tick_duration {
            //     std::thread::sleep(tick_duration - tick_elapsed);
            // } else {
            //     tracing::warn!(
            //         "Server tick took longer than target duration: {:?} > {:?}",
            //         tick_elapsed,
            //         tick_duration
            //     );
            // }
        }

        tracing::info!("GameServer thread stopped");
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
        self.poll_transports(&mut incoming_events);
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

        let connected_client_ids: HashSet<Uuid> = self
            .world
            .get_resource::<network::ConnectedClients>()
            .map(|cc| cc.clients.clone())
            .unwrap_or_default();

        // Group messages: broadcasts vs unicasts
        let mut broadcasts = Vec::new();
        let mut unicasts = Vec::new();

        for (target_client, message) in messages {
            match target_client {
                None => broadcasts.push(message),
                Some(client_id) => unicasts.push((client_id, message)),
            }
        }

        let serialize_message = |msg: &protocol::GameplayMessage| -> Option<bytes::Bytes> {
            match bincode::serde::encode_to_vec(msg, bincode::config::standard()) {
                Ok(bytes) => Some(bytes::Bytes::from(bytes)),
                Err(e) => {
                    tracing::error!(
                        "Failed to serialize gameplay message on channel {}: {:?}",
                        msg.channel(),
                        e
                    );
                    None
                }
            }
        };

        // Process broadcasts: use loopback fast-path when no external transport is attached
        for message in broadcasts {
            match &mut self.mode {
                ServerMode::Embedded { loopback, external } if external.is_none() => {
                    let channel = message.channel();

                    tracing::debug!(
                        "Broadcasting gameplay message via loopback fast-path on channel {}",
                        channel
                    );

                    if let Err(e) = loopback.send_direct(channel, message) {
                        tracing::warn!(
                            "Failed to broadcast to loopback client via fast-path: {:?}",
                            e
                        );
                    }
                }
                ServerMode::Embedded { loopback, external } => {
                    let channel = message.channel();
                    let Some(raw_bytes) = serialize_message(&message) else {
                        continue;
                    };

                    let payload_len = raw_bytes.len();
                    let payload = TransportPayload::message(channel, raw_bytes.clone());

                    tracing::debug!(
                        "Broadcasting message to {} clients on channel {}, {} bytes",
                        connected_client_ids.len(),
                        channel,
                        payload_len
                    );

                    if let Err(e) = loopback.send(HOST_CLIENT_ID, payload.clone()) {
                        tracing::warn!("Failed to broadcast to loopback client: {:?}", e);
                    }

                    if let Some(ext) = external.as_mut() {
                        if let Err(e) = ext.broadcast_excluding(&[HOST_CLIENT_ID], payload) {
                            tracing::warn!("Failed to broadcast to external clients: {:?}", e);
                        }
                    }
                }
                ServerMode::DedicatedQuic { external } => {
                    let channel = message.channel();
                    let Some(raw_bytes) = serialize_message(&message) else {
                        continue;
                    };

                    let payload_len = raw_bytes.len();
                    let payload = TransportPayload::message(channel, raw_bytes);

                    tracing::debug!(
                        "Broadcasting message to dedicated clients on channel {}, {} bytes",
                        channel,
                        payload_len
                    );

                    if let Err(e) = external.broadcast_excluding(&[], payload) {
                        tracing::warn!("Failed to broadcast to clients: {:?}", e);
                    }
                }
            }
        }

        // Process unicasts: use loopback fast-path when sending to host without external transport
        for (client_id, message) in unicasts {
            match &mut self.mode {
                ServerMode::Embedded { loopback, external } if external.is_none() => {
                    if client_id != HOST_CLIENT_ID {
                        tracing::warn!(
                            "Attempted to send to client {} without external transport",
                            client_id
                        );
                        continue;
                    }

                    let channel = message.channel();

                    tracing::debug!(
                        "Sending gameplay message to host via loopback fast-path on channel {}",
                        channel
                    );

                    if let Err(e) = loopback.send_direct(channel, message) {
                        tracing::warn!("Failed to send to loopback client via fast-path: {:?}", e);
                    }
                }
                ServerMode::Embedded { loopback, external } => {
                    let channel = message.channel();
                    let Some(raw_bytes) = serialize_message(&message) else {
                        continue;
                    };

                    let payload_len = raw_bytes.len();
                    let payload = TransportPayload::message(channel, raw_bytes);

                    tracing::debug!(
                        "Sending message to client {} on channel {}, {} bytes",
                        client_id,
                        channel,
                        payload_len
                    );

                    if client_id == HOST_CLIENT_ID {
                        if let Err(e) = loopback.send(client_id, payload) {
                            tracing::warn!(
                                "Failed to send to loopback client {}: {:?}",
                                client_id,
                                e
                            );
                        }
                    } else if let Some(ext) = external.as_mut() {
                        if let Err(e) = ext.send(client_id, payload) {
                            tracing::warn!(
                                "Failed to send to external client {}: {:?}",
                                client_id,
                                e
                            );
                        }
                    }
                }
                ServerMode::DedicatedQuic { external } => {
                    let channel = message.channel();
                    let Some(raw_bytes) = serialize_message(&message) else {
                        continue;
                    };

                    let payload_len = raw_bytes.len();
                    let payload = TransportPayload::message(channel, raw_bytes);

                    tracing::debug!(
                        "Sending message to client {} on channel {}, {} bytes",
                        client_id,
                        channel,
                        payload_len
                    );

                    if let Err(e) = external.send(client_id, payload) {
                        tracing::warn!("Failed to send to client {}: {:?}", client_id, e);
                    }
                }
            }
        }
    }

    fn poll_transports(&mut self, output: &mut Vec<TransportEvent>) {
        match &mut self.mode {
            ServerMode::Embedded { loopback, external } => {
                loopback.poll_events(output);
                if let Some(ext) = external {
                    ext.poll_events(output);
                }
            }
            ServerMode::DedicatedQuic { external } => {
                external.poll_events(output);
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

        match &mut self.mode {
            ServerMode::Embedded { external, .. } => {
                if let Some(ext) = external {
                    ext.shutdown();
                }
            }
            ServerMode::DedicatedQuic { external } => {
                external.shutdown();
            }
        }

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

    /// Pauses the server (internal, called by run loop).
    fn pause(&mut self) {
        self.state = ServerState::Paused;
        tracing::info!("Server paused");
    }

    /// Resumes the server (internal, called by run loop).
    fn resume(&mut self) {
        self.state = ServerState::Running;
        tracing::info!("Server resumed");
    }

    fn enter_receive(&mut self) {
        self.metrics.receive_enter_time = SystemTime::now();
    }
    fn enter_simulation(&mut self) {}
    fn enter_replication(&mut self) {}
    fn enter_transmit(&mut self) {}
    fn exit_receive(&mut self) {}
    fn exit_simulation(&mut self) {}
    fn exit_replication(&mut self) {}
    fn exit_transmit(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.tick_rate, 20.0);
        assert_eq!(config.max_clients, 16);
    }
}
