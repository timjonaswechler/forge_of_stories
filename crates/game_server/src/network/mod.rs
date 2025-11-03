//! Server networking setup and connection handling.
//!
//! This module handles:
//! - Server initialization with bevy_replicon_renet
//! - Port binding with automatic fallback
//! - Client connection/disconnection handling

use app::LOG_SERVER;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    RenetChannelsExt,
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
};
use std::{
    net::{Ipv4Addr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::SystemTime,
};

use crate::shared::*;
use crate::world::{PlayerColorAssigner, spawn_world};

/// Server port resource.
#[derive(Resource, Debug)]
pub struct Port(pub u16);

/// Internal resource for signaling server readiness to the main thread.
/// Public because lib.rs needs it for ServerHandle.
#[derive(Resource, Clone)]
pub struct ServerReadyFlag(pub Arc<AtomicBool>);

/// Internal resource for storing the actual bound port.
/// Public because lib.rs needs it for ServerHandle.
#[derive(Resource, Clone)]
pub struct PortStorage(pub Arc<std::sync::Mutex<u16>>);

/// Tracks whether the world has been spawned (only spawn on first client connection).
#[derive(Resource, Default)]
pub struct WorldSpawned(pub bool);

/// System that sets up the server networking (runs in Startup).
pub fn setup_networking(
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
        max_clients: 10,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create server transport");

    commands.insert_resource(server);
    commands.insert_resource(transport);
    commands.insert_resource(Port(actual_port));

    // Store the actual port so the client can read it
    *port_storage.0.lock().unwrap() = actual_port;

    info!(
        target: LOG_SERVER,
        "Server fully started on 127.0.0.1:{}", actual_port
    );
    ready_flag.0.store(true, Ordering::Release);
}

/// Tries to bind to a port, and if it fails, tries the next ports until it finds a free one.
/// Tries up to 10 ports starting from the given port.
fn find_free_port(start_port: u16) -> std::io::Result<(UdpSocket, u16)> {
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

/// System that handles new client connections.
///
/// When a client connects (ConnectedClient entity is spawned by the backend),
/// this system spawns a player entity for them and initializes the world on first connection.
pub fn handle_client_connections(
    mut commands: Commands,
    new_clients: Query<Entity, Added<ConnectedClient>>,
    client_ids: Query<&bevy_replicon::shared::backend::connected_client::NetworkId>,
    mut color_assigner: ResMut<PlayerColorAssigner>,
    mut world_spawned: ResMut<WorldSpawned>,
) {
    for client_entity in &new_clients {
        info!(
            target: LOG_SERVER,
            "New client entity detected: {:?}", client_entity
        );

        // Spawn world on first client connection
        if !world_spawned.0 {
            info!(
                target: LOG_SERVER,
                "First client connected, initializing game world"
            );
            spawn_world(&mut commands);
            world_spawned.0 = true;
            info!(target: LOG_SERVER, "World spawned successfully");
        }

        let color = color_assigner.next_color();
        let network_id = client_ids
            .get(client_entity)
            .expect("connected client is missing NetworkId");
        let client_id_value = network_id.get();

        // Spawn player entity with replicated components
        let _player_entity = commands
            .spawn((
                Player { color },
                PlayerIdentity {
                    client_id: client_id_value,
                },
                PlayerOwner { client_entity }, // Server-only, not replicated
                Position {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                },
                Velocity::default(),
                Replicated, // Mark for replication
            ))
            .id();

        info!(
            target: LOG_SERVER,
            "Spawned player for client {} (entity {:?})",
            client_id_value,
            client_entity
        );
    }
}

/// System that handles client disconnections.
///
/// When a client disconnects (ConnectedClient component is removed),
/// this system finds and despawns all entities owned by that client.
pub fn handle_client_disconnections(
    mut commands: Commands,
    mut disconnected_clients: RemovedComponents<ConnectedClient>,
    players: Query<(Entity, &PlayerOwner)>,
) {
    for disconnected_client in disconnected_clients.read() {
        info!(
            target: LOG_SERVER,
            "Client {:?} disconnected, cleaning up...",
            disconnected_client
        );

        // Find and despawn all entities owned by this client
        let mut despawned_count = 0;
        for (player_entity, owner) in &players {
            if owner.client_entity == disconnected_client {
                commands.entity(player_entity).despawn();
                despawned_count += 1;
                info!(
                    target: LOG_SERVER,
                    "Despawned player entity {:?} for disconnected client",
                    player_entity
                );
            }
        }

        info!(
            target: LOG_SERVER,
            "Client {:?} cleanup complete ({} entities despawned)",
            disconnected_client,
            despawned_count
        );
    }
}
