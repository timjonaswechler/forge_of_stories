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
use bevy_replicon::{prelude::*, server::ServerSystems};
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
};
use std::{
    net::{Ipv4Addr, UdpSocket},
    thread,
    thread::JoinHandle,
    time::SystemTime,
};

pub mod components;
pub mod world;
use components::PlayerOwner;
use world::*;

#[derive(Resource, Debug)]
pub struct Port(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameplayState {
    #[default]
    Unpaused,
    Paused, // Menü geöffnet, Singleplayer pausiert
}

#[derive(Resource)]
pub struct ServerHandle {
    thread_handle: Option<JoinHandle<()>>,
    // Kein eigener State mehr nötig - Replicon managed das
}

impl ServerHandle {
    pub fn start_embedded(port: Port) -> Self {
        let thread_handle = thread::spawn(move || {
            App::new()
                .add_plugins((MinimalPlugins, RepliconPlugins, RepliconRenetPlugins))
                .insert_resource(Time::<Fixed>::from_hz(20.0))
                .insert_resource(port)
                .init_state::<GameplayState>()
                // Beim Server-Start
                .add_systems(
                    OnEnter(ServerState::Running),
                    (setup_networking, initialize_world).chain(),
                )
                .add_systems(
                    PreUpdate,
                    handle_client_connections
                        .in_set(ServerSystems::Receive) // Läuft nachdem Replicon Nachrichten empfangen hat
                        .run_if(in_state(ServerState::Running)),
                )
                .add_systems(
                    FixedUpdate,
                    simulate_physics
                        .run_if(in_state(ServerState::Running))
                        .run_if(in_state(GameplayState::Unpaused)),
                )
                .add_systems(OnExit(ServerState::Running), save_world)
                .run();
        });
        Self {
            thread_handle: Some(thread_handle),
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("Failed to join server thread");
        }
    }
}

fn setup_networking(mut commands: Commands, channels: Res<RepliconChannels>, port: Res<Port>) {
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

    let socket =
        UdpSocket::bind((Ipv4Addr::LOCALHOST, port.0)).expect("Failed to bind server socket");

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

    info!("Server started on port {}", port.0);
}

fn save_world(mut commands: Commands) -> Result<()> {
    // Implement saving logic here
    Ok(())
}

fn simulate_physics(mut commands: Commands) -> Result<()> {
    // Implement physics simulation logic here
    Ok(())
}
fn initialize_world(mut commands: Commands) -> Result<()> {
    // Implement world initialization logic here
    Ok(())
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

        info!(
            "Client {} connected, spawning player with color {:?}",
            client_id, color
        );

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
