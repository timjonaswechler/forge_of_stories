//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - `aether` (dedicated server binary)
//! - `EmbeddedServer` (client-hosted server in `forge_of_stories`)
//!
//! The server runs in its own thread with a complete Bevy App using bevy_replicon
//! for automatic server-authoritative replication.

use app::LOG_SERVER;
use bevy::{prelude::*, state::app::StatesPlugin};
use bevy_replicon::shared::backend::ServerState as RepliconServerState;
use bevy_replicon::{prelude::*, server::ServerSystems};
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
};
use std::{
    net::{Ipv4Addr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    thread::JoinHandle,
    time::SystemTime,
};

pub mod components;
pub mod settings;
pub mod world;
pub mod world_setup;
use components::PlayerOwner;
use world::GroundPlaneSize;
use world::*;
use world_setup::spawn_world;

#[derive(Resource, Debug)]
pub struct Port(pub u16);

#[derive(Resource, Clone)]
struct ServerReadyFlag(Arc<AtomicBool>);

#[derive(Resource, Clone)]
struct PortStorage(Arc<std::sync::Mutex<u16>>);

#[derive(Resource, Default)]
struct WorldSpawned(bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameplayState {
    #[default]
    Unpaused,
    Paused, // Menü geöffnet, Singleplayer pausiert
}

#[derive(Resource)]
pub struct ServerHandle {
    thread_handle: Option<JoinHandle<()>>,
    ready_flag: Arc<AtomicBool>,
    port: Arc<std::sync::Mutex<u16>>,
}

impl ServerHandle {
    pub fn start_embedded(port: Port) -> Self {
        let ready_flag = Arc::new(AtomicBool::new(false));
        let server_ready = ready_flag.clone();

        let actual_port = Arc::new(std::sync::Mutex::new(port.0));
        let thread_port = actual_port.clone();

        let thread_handle = thread::spawn(move || {
            let thread_ready = ServerReadyFlag(server_ready);
            let port_storage = PortStorage(thread_port);

            App::new()
                .add_plugins((
                    MinimalPlugins,
                    StatesPlugin,
                    RepliconPlugins,
                    RepliconRenetPlugins,
                ))
                .insert_resource(Time::<Fixed>::from_hz(20.0))
                .insert_resource(port)
                .init_state::<GameplayState>()
                .insert_resource(thread_ready)
                .insert_resource(port_storage)
                .init_resource::<WorldSpawned>()
                .init_resource::<PlayerColorAssigner>()
                .replicate::<Player>()
                .replicate::<Position>()
                .replicate::<Velocity>()
                .replicate::<GroundPlane>()
                .replicate::<GroundPlaneSize>()
                // Server-Networking beim Start einrichten
                .add_systems(Startup, setup_networking)
                .add_systems(
                    PreUpdate,
                    handle_client_connections
                        .in_set(ServerSystems::Receive) // Läuft nachdem Replicon Nachrichten empfangen hat
                        .run_if(in_state(RepliconServerState::Running)),
                )
                .add_systems(
                    FixedUpdate,
                    simulate_physics
                        .run_if(in_state(RepliconServerState::Running))
                        .run_if(in_state(GameplayState::Unpaused)),
                )
                .add_systems(OnExit(RepliconServerState::Running), save_world)
                .run();
        });
        Self {
            thread_handle: Some(thread_handle),
            ready_flag,
            port: actual_port,
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("Failed to join server thread");
        }
    }

    /// Indicates whether the embedded server finished initializing networking
    /// and is ready to accept client connections.
    pub fn is_ready(&self) -> bool {
        self.ready_flag.load(Ordering::Acquire)
    }

    /// Returns the actual port the server is bound to.
    /// This may differ from the requested port if it was already in use.
    pub fn port(&self) -> u16 {
        *self.port.lock().unwrap()
    }
}

fn setup_networking(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    port: Res<Port>,
    ready_flag: Res<ServerReadyFlag>,
    port_storage: Res<PortStorage>,
) {
    const PROTOCOL_ID: u64 = 0;

    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get system time");

    // Try to find a free port starting from the requested port
    let (socket, actual_port) = find_free_port(port.0).expect("Failed to find a free port");

    if actual_port != port.0 {
        warn!(
            "Port {} was in use, using port {} instead",
            port.0, actual_port
        );
    }

    let server_config = ServerConfig {
        current_time,
        max_clients: 1,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create server transport");

    commands.insert_resource(server);
    commands.insert_resource(transport);
    // Update the port resource with the actual port we bound to
    commands.insert_resource(Port(actual_port));

    // Store the actual port so the client can read it
    *port_storage.0.lock().unwrap() = actual_port;

    info!(target: LOG_SERVER,"Server fully started on 127.0.0.1:{}", actual_port);
    ready_flag.0.store(true, Ordering::Release);
}

/// Tries to bind to a port, and if it fails, tries the next ports until it finds a free one.
/// Tries up to 10 ports starting from the given port.
fn find_free_port(start_port: u16) -> Result<(UdpSocket, u16)> {
    const MAX_ATTEMPTS: u16 = 10;

    for offset in 0..MAX_ATTEMPTS {
        let port = start_port + offset;
        match UdpSocket::bind((Ipv4Addr::LOCALHOST, port)) {
            Ok(socket) => {
                return Ok((socket, port));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                // Port is in use, try next one
                continue;
            }
            Err(e) => {
                // Other error, propagate it
                return Err(e.into());
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        format!(
            "Could not find a free port in range {}-{}",
            start_port,
            start_port + MAX_ATTEMPTS - 1
        ),
    )
    .into())
}

fn save_world(mut _commands: Commands) -> Result<()> {
    // Implement saving logic here
    Ok(())
}

fn simulate_physics(mut _commands: Commands) -> Result<()> {
    // Implement physics simulation logic here
    Ok(())
}

/// System that handles new client connections.
///
/// When a client connects (ConnectedClient entity is spawned by the backend),
/// this system spawns a player entity for them and initializes the world on first connection.
fn handle_client_connections(
    mut commands: Commands,
    new_clients: Query<Entity, Added<ConnectedClient>>,
    mut color_assigner: ResMut<world::PlayerColorAssigner>,
    mut world_spawned: ResMut<WorldSpawned>,
) {
    for client_entity in &new_clients {
        info!(target: LOG_SERVER,"New client entity detected: {:?}", client_entity);

        // Spawn world on first client connection
        if !world_spawned.0 {
            info!(target: LOG_SERVER,"First client connected, initializing game world");
            info!(target: LOG_SERVER,"Spawning world...");
            spawn_world(&mut commands);
            world_spawned.0 = true;
            info!(target: LOG_SERVER,"World spawned successfully");
        }

        let color = color_assigner.next_color();
        let _client_id = ClientId::from(client_entity);

        // Spawn player entity with replicated components
        let _player_entity = commands
            .spawn((
                Player { color },
                PlayerOwner { client_entity }, // Link player to client (not replicated)
                Position {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                },
                Velocity::default(),
                Replicated, // Mark for replication
            ))
            .id();
    }
}
