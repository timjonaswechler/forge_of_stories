//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - `aether` (dedicated server binary)
//! - `EmbeddedServer` (client-hosted server in `forge_of_stories`)
//!
//! The server runs in its own thread with a complete Bevy App using bevy_replicon
//! for automatic server-authoritative replication.

use bevy::prelude::*;
use loopback::{LoopbackBackendPlugins, LoopbackServer};
use networking::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::{self, JoinHandle};

pub mod components;
pub mod messages;
pub mod movement;
pub mod savegame;
pub mod world;
pub mod world_setup;

// Re-export commonly used types
pub use components::{Player, PlayerOwner, Position, Velocity};
pub use messages::PlayerInput;

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

    /// Current server state (shared with thread via Arc)
    state: Arc<AtomicServerState>,

    /// Server mode information
    mode: ServerMode,
}

/// Atomic version of GameServerState for thread-safe access.
pub struct AtomicServerState {
    state: AtomicU8,
}

impl AtomicServerState {
    fn new(state: GameServerState) -> Self {
        Self {
            state: AtomicU8::new(state as u8),
        }
    }

    fn load(&self) -> GameServerState {
        match self.state.load(Ordering::Relaxed) {
            0 => GameServerState::Starting,
            1 => GameServerState::Running,
            2 => GameServerState::Paused,
            3 => GameServerState::ShuttingDown,
            4 => GameServerState::Stopped,
            _ => GameServerState::Stopped,
        }
    }

    fn store(&self, state: GameServerState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }
}

/// Server operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    /// Embedded mode: Server runs in client process with loopback connection
    Embedded,
    /// Dedicated mode: Standalone server with QUIC transport
    Dedicated,
}

/// Current lifecycle state of the game server thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, States, Default, Hash)]
#[repr(u8)]
pub enum GameServerState {
    /// Server is initializing
    #[default]
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

impl ServerHandle {
    /// Starts a new embedded server in a separate thread.
    ///
    /// The server will run a complete Bevy App with bevy_replicon for replication.
    /// It uses the loopback backend for communication with the host client.
    ///
    /// # Arguments
    /// * `port` - TCP port for the loopback server
    ///
    /// # Returns
    /// A `ServerHandle` to control the server thread.
    pub fn start_embedded(port: u16) -> Self {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let state = Arc::new(AtomicServerState::new(GameServerState::Starting));
        let state_clone = state.clone();

        // Spawn the server thread
        let thread_handle = thread::spawn(move || {
            run_server_app(command_rx, state_clone, port);
        });

        Self {
            thread_handle: Some(thread_handle),
            command_tx,
            state,
            mode: ServerMode::Embedded,
        }
    }

    /// Returns the current server state.
    pub fn state(&self) -> GameServerState {
        self.state.load()
    }

    /// Returns the server mode.
    pub fn mode(&self) -> ServerMode {
        self.mode
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
// Server App Setup & Loop
// ==============================================================================

/// Runs the server Bevy App in the current thread.
///
/// This function creates a Bevy App with:
/// - MinimalPlugins (no rendering/audio)
/// - RepliconPlugins (server-authoritative replication)
/// - LoopbackBackendPlugins (TCP loopback transport)
/// - Replicated components (Player, Position, Velocity)
/// - Client messages (PlayerInput)
/// - Game systems (spawn_player, movement, etc.)
fn run_server_app(
    command_rx: crossbeam::channel::Receiver<ServerCommand>,
    state: Arc<AtomicServerState>,
    port: u16,
) {
    info!("Starting server Bevy App on port {}", port);

    let mut app = App::new();

    // Core plugins (minimal - no rendering/audio)
    app.add_plugins(MinimalPlugins);

    // Initialize game server state
    app.init_state::<GameServerState>();

    // Networking plugins
    app.add_plugins(RepliconPlugins)
        .add_plugins(LoopbackBackendPlugins);

    // Register replicated components
    app.replicate::<Player>()
        .replicate::<Position>()
        .replicate::<Velocity>();

    // Register client messages
    app.add_client_message::<PlayerInput>(Channel::Ordered);

    // Add game resources
    app.insert_resource(ServerCommandReceiver(command_rx))
        .insert_resource(world::PlayerColorAssigner::default());

    // Add game systems
    app.add_systems(
        PreUpdate,
        (
            process_server_commands,
            handle_client_connections.after(ServerSystems::Receive),
        ),
    )
    .add_systems(
        Update,
        (
            movement::process_player_input,
            movement::apply_velocity,
        )
            .run_if(in_state(GameServerState::Running)),
    );

    // Initialize world
    world_setup::spawn_world(&mut app.world_mut());

    // Start loopback server
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    match LoopbackServer::new(addr) {
        Ok(server) => {
            app.insert_resource(server);
            info!("Loopback server listening on {}", addr);
        }
        Err(e) => {
            error!("Failed to start loopback server: {}", e);
            state.store(GameServerState::Stopped);
            return;
        }
    }

    // Update state to Running
    state.store(GameServerState::Running);
    info!("Server running");

    // Main server loop
    loop {
        // Run one frame of the Bevy App
        // The process_server_commands system will handle shutdown
        app.update();

        // Check if we're shutting down
        if matches!(
            app.world().get_resource::<State<GameServerState>>().map(|s| **s),
            Some(GameServerState::ShuttingDown)
        ) {
            info!("Shutdown initiated");
            break;
        }
    }

    // Cleanup
    app.world_mut().remove_resource::<LoopbackServer>();
    state.store(GameServerState::Stopped);
    info!("Server stopped");
}

// ==============================================================================
// Server Systems
// ==============================================================================

/// Resource to receive commands from the main thread.
#[derive(Resource)]
struct ServerCommandReceiver(crossbeam::channel::Receiver<ServerCommand>);

/// System that processes server commands from the command channel.
fn process_server_commands(
    receiver: Option<Res<ServerCommandReceiver>>,
    mut next_state: ResMut<NextState<GameServerState>>,
) {
    let Some(receiver) = receiver else {
        return;
    };

    while let Ok(command) = receiver.0.try_recv() {
        match command {
            ServerCommand::Shutdown => {
                info!("Processing shutdown command");
                next_state.set(GameServerState::ShuttingDown);
            }
            ServerCommand::Pause => {
                info!("Processing pause command");
                next_state.set(GameServerState::Paused);
            }
            ServerCommand::Resume => {
                info!("Processing resume command");
                next_state.set(GameServerState::Running);
            }
        }
    }
}

/// System that handles new client connections.
///
/// When a client connects (ConnectedClient entity is spawned by the backend),
/// this system spawns a player entity for them.
fn handle_client_connections(
    mut commands: Commands,
    new_clients: Query<Entity, Added<ConnectedClient>>,
    mut color_assigner: ResMut<world::PlayerColorAssigner>,
) {
    for client_entity in &new_clients {
        let color = color_assigner.next_color();
        let client_id = ClientId::from(client_entity);

        info!("Client {} connected, spawning player with color {:?}", client_id, color);

        // Spawn player entity with replicated components
        commands.spawn((
            Player { color },
            PlayerOwner { client_entity }, // Link player to client (not replicated)
            Position {
                translation: Vec3::new(0.0, 1.0, 0.0),
            },
            Velocity::default(),
            Replicated, // Mark for replication
        ));
    }
}
